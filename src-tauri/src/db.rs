use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: i64,
    pub prompt: String,
    pub model: String,
    pub duration: i64,
    pub resolution: String,
    pub prompt_optimizer: bool,
    pub status: String,
    pub task_id: Option<String>,
    pub file_id: Option<String>,
    pub fail_reason: Option<String>,
    pub local_path: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub api_key: String,
    pub download_dir: String,
}

pub fn init(conn: &Connection, default_dir: &str) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS videos (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            prompt           TEXT NOT NULL,
            model            TEXT NOT NULL DEFAULT 'MiniMax-Hailuo-2.3',
            duration         INTEGER NOT NULL DEFAULT 6,
            resolution       TEXT NOT NULL DEFAULT '768P',
            prompt_optimizer INTEGER NOT NULL DEFAULT 1,
            status           TEXT NOT NULL DEFAULT 'submitted',
            task_id          TEXT,
            file_id          TEXT,
            fail_reason      TEXT,
            local_path       TEXT,
            file_size        INTEGER,
            created_at       TEXT NOT NULL,
            updated_at       TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES ('api_key', ''), ('download_dir', ?1)",
            params![default_dir],
        )?;
    }
    Ok(())
}

fn row_to_video(row: &Row) -> rusqlite::Result<Video> {
    Ok(Video {
        id: row.get("id")?,
        prompt: row.get("prompt")?,
        model: row.get("model")?,
        duration: row.get("duration")?,
        resolution: row.get("resolution")?,
        prompt_optimizer: row.get::<_, i64>("prompt_optimizer")? != 0,
        status: row.get("status")?,
        task_id: row.get("task_id")?,
        file_id: row.get("file_id")?,
        fail_reason: row.get("fail_reason")?,
        local_path: row.get("local_path")?,
        file_size: row.get("file_size")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn get_settings(conn: &Connection) -> Result<Settings, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let mut api_key = String::new();
    let mut download_dir = String::new();
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (k, v) = row.map_err(|e| e.to_string())?;
        match k.as_str() {
            "api_key" => api_key = v,
            "download_dir" => download_dir = v,
            _ => {}
        }
    }
    Ok(Settings {
        api_key,
        download_dir,
    })
}

pub fn save_settings(conn: &Connection, api_key: &str, download_dir: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('api_key', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![api_key],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('download_dir', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![download_dir],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn insert_video(
    conn: &Connection,
    prompt: &str,
    model: &str,
    duration: i64,
    resolution: &str,
    prompt_optimizer: bool,
    task_id: &str,
) -> Result<i64, String> {
    let ts = now();
    conn.execute(
        "INSERT INTO videos (prompt, model, duration, resolution, prompt_optimizer, status, task_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'submitted', ?6, ?7, ?7)",
        params![
            prompt,
            model,
            duration,
            resolution,
            prompt_optimizer as i64,
            task_id,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn get_video(conn: &Connection, id: i64) -> Result<Video, String> {
    conn.query_row("SELECT * FROM videos WHERE id = ?1", params![id], row_to_video)
        .optional()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("记录 #{id} 不存在"))
}

pub fn list_videos(conn: &Connection) -> Result<Vec<Video>, String> {
    let mut stmt = conn
        .prepare("SELECT * FROM videos ORDER BY id DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_video)
        .map_err(|e| e.to_string())?;
    let mut videos = Vec::new();
    for row in rows {
        videos.push(row.map_err(|e| e.to_string())?);
    }
    Ok(videos)
}

pub fn update_video_status(
    conn: &Connection,
    id: i64,
    status: &str,
    file_id: Option<&str>,
    fail_reason: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "UPDATE videos SET status = ?1, file_id = COALESCE(?2, file_id), fail_reason = ?3, updated_at = ?4 WHERE id = ?5",
        params![status, file_id, fail_reason, now(), id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_video_prompt(conn: &Connection, id: i64, prompt: &str) -> Result<(), String> {
    let n = conn
        .execute(
            "UPDATE videos SET prompt = ?1, updated_at = ?2 WHERE id = ?3",
            params![prompt, now(), id],
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!("记录 #{id} 不存在"));
    }
    Ok(())
}

pub fn set_local_file(
    conn: &Connection,
    id: i64,
    path: &str,
    file_size: i64,
) -> Result<(), String> {
    conn.execute(
        "UPDATE videos SET local_path = ?1, file_size = ?2, updated_at = ?3 WHERE id = ?4",
        params![path, file_size, now(), id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reset_for_retry(conn: &Connection, id: i64, task_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE videos SET status = 'submitted', task_id = ?1, file_id = NULL, fail_reason = NULL, local_path = NULL, file_size = NULL, updated_at = ?2 WHERE id = ?3",
        params![task_id, now(), id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_video(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM videos WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err(format!("记录 #{id} 不存在"));
    }
    Ok(())
}
