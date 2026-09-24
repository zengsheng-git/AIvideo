mod db;
mod minimax;

use db::{Settings, Video};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
}

fn with_db<T>(
    state: &State<AppState>,
    f: impl FnOnce(&rusqlite::Connection) -> Result<T, String>,
) -> Result<T, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    f(&conn)
}

// ---------- 设置 ----------

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    with_db(&state, |conn| db::get_settings(conn))
}

#[tauri::command]
fn save_settings(state: State<AppState>, api_key: String, download_dir: String) -> Result<(), String> {
    with_db(&state, |conn| db::save_settings(conn, &api_key, &download_dir))
}

#[tauri::command]
async fn pick_download_dir(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.map(|p| p.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}

// ---------- 视频生成 ----------

#[tauri::command]
async fn create_video(
    state: State<'_, AppState>,
    prompt: String,
    model: String,
    duration: i64,
    resolution: String,
    prompt_optimizer: bool,
) -> Result<Video, String> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err("请输入视频描述".to_string());
    }
    if prompt.chars().count() > 2000 {
        return Err("视频描述不能超过 2000 字符".to_string());
    }
    if resolution == "1080P" && duration != 6 {
        return Err("1080P 分辨率仅支持 6 秒时长".to_string());
    }
    let api_key = with_db(&state, |conn| db::get_settings(conn))?.api_key;
    if api_key.trim().is_empty() {
        return Err("请先在设置页配置 MiniMax API Key".to_string());
    }
    let task_id = minimax::create_task(&api_key, &prompt, &model, duration, &resolution, prompt_optimizer).await?;
    let id = with_db(&state, |conn| {
        db::insert_video(conn, &prompt, &model, duration, &resolution, prompt_optimizer, &task_id)
    })?;
    with_db(&state, |conn| db::get_video(conn, id))
}

#[tauri::command]
async fn poll_video(state: State<'_, AppState>, id: i64) -> Result<Video, String> {
    let video = with_db(&state, |conn| db::get_video(conn, id))?;
    if matches!(video.status.as_str(), "success" | "fail") {
        return Ok(video);
    }
    let task_id = video
        .task_id
        .clone()
        .ok_or_else(|| "该记录没有关联的生成任务".to_string())?;
    let api_key = with_db(&state, |conn| db::get_settings(conn))?.api_key;
    match minimax::query_task(&api_key, &task_id).await? {
        minimax::TaskStatus::Success { file_id } => {
            with_db(&state, |conn| {
                db::update_video_status(conn, id, "success", Some(&file_id), None)
            })?;
        }
        minimax::TaskStatus::Fail { reason } => {
            with_db(&state, |conn| {
                db::update_video_status(conn, id, "fail", None, Some(&reason))
            })?;
        }
        minimax::TaskStatus::Processing => {
            with_db(&state, |conn| {
                db::update_video_status(conn, id, "processing", None, None)
            })?;
        }
        minimax::TaskStatus::Pending => {}
    }
    with_db(&state, |conn| db::get_video(conn, id))
}

#[tauri::command]
async fn retry_video(state: State<'_, AppState>, id: i64) -> Result<Video, String> {
    let video = with_db(&state, |conn| db::get_video(conn, id))?;
    if matches!(video.status.as_str(), "submitted" | "processing") {
        return Err("该视频正在生成中，无需重试".to_string());
    }
    let api_key = with_db(&state, |conn| db::get_settings(conn))?.api_key;
    if api_key.trim().is_empty() {
        return Err("请先在设置页配置 MiniMax API Key".to_string());
    }
    let task_id = minimax::create_task(
        &api_key,
        &video.prompt,
        &video.model,
        video.duration,
        &video.resolution,
        video.prompt_optimizer,
    )
    .await?;
    with_db(&state, |conn| db::reset_for_retry(conn, id, &task_id))?;
    with_db(&state, |conn| db::get_video(conn, id))
}

// ---------- 文件 ----------

#[tauri::command]
async fn download_video(state: State<'_, AppState>, id: i64) -> Result<Video, String> {
    let video = with_db(&state, |conn| db::get_video(conn, id))?;
    if let Some(path) = &video.local_path {
        if std::path::Path::new(path).exists() {
            return Ok(video);
        }
    }
    if video.status != "success" {
        return Err("视频尚未生成完成，无法下载".to_string());
    }
    let file_id = video
        .file_id
        .clone()
        .ok_or_else(|| "缺少文件 ID，请重新查询任务状态".to_string())?;
    let settings = with_db(&state, |conn| db::get_settings(conn))?;
    std::fs::create_dir_all(&settings.download_dir).map_err(|e| e.to_string())?;
    let url = minimax::get_download_url(&settings.api_key, &file_id).await?;
    let dest = std::path::Path::new(&settings.download_dir)
        .join(format!("video_{}_{}.mp4", id, chrono::Local::now().format("%Y%m%d_%H%M%S")));
    let size = minimax::download_file(&url, &dest).await?;
    with_db(&state, |conn| {
        db::set_local_file(conn, id, &dest.to_string_lossy(), size as i64)
    })?;
    with_db(&state, |conn| db::get_video(conn, id))
}

#[tauri::command]
fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

// ---------- 增删改查 ----------

#[tauri::command]
fn list_videos(state: State<AppState>) -> Result<Vec<Video>, String> {
    with_db(&state, |conn| db::list_videos(conn))
}

#[tauri::command]
fn update_video(state: State<AppState>, id: i64, prompt: String) -> Result<Video, String> {
    let prompt = prompt.trim().to_string();
    if prompt.is_empty() {
        return Err("视频描述不能为空".to_string());
    }
    if prompt.chars().count() > 2000 {
        return Err("视频描述不能超过 2000 字符".to_string());
    }
    with_db(&state, |conn| db::update_video_prompt(conn, id, &prompt))?;
    with_db(&state, |conn| db::get_video(conn, id))
}

#[tauri::command]
fn delete_video(state: State<AppState>, id: i64, delete_file: bool) -> Result<(), String> {
    let video = with_db(&state, |conn| db::get_video(conn, id))?;
    if delete_file {
        if let Some(path) = &video.local_path {
            let _ = std::fs::remove_file(path);
        }
    }
    with_db(&state, |conn| db::delete_video(conn, id))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            std::fs::create_dir_all(&data_dir).expect("无法创建应用数据目录");
            let videos_dir = data_dir.join("videos");
            std::fs::create_dir_all(&videos_dir).expect("无法创建视频目录");
            let conn =
                rusqlite::Connection::open(data_dir.join("aivideo.db")).expect("无法打开数据库");
            db::init(&conn, &videos_dir.to_string_lossy()).expect("数据库初始化失败");
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            pick_download_dir,
            create_video,
            poll_video,
            retry_video,
            download_video,
            open_path,
            list_videos,
            update_video,
            delete_video
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
