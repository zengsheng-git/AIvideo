use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

const BASE_URL: &str = "https://api.minimaxi.com";

fn friendly_error(code: i32, msg: &str) -> String {
    match code {
        1002 => "请求过于频繁（限流），请稍后重试".to_string(),
        1004 => "鉴权失败，请检查 API Key 是否正确".to_string(),
        1008 => "账户余额不足，请前往 MiniMax 平台充值".to_string(),
        1026 | 1027 => "内容涉及敏感信息，请修改描述后重试".to_string(),
        2013 => format!("参数异常：{msg}"),
        _ => format!("API 错误 {code}: {msg}"),
    }
}

fn check_base_resp(code: i32, msg: &str) -> Result<(), String> {
    if code == 0 {
        Ok(())
    } else {
        Err(friendly_error(code, msg))
    }
}

fn client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskReq<'a> {
    model: &'a str,
    prompt: &'a str,
    duration: i64,
    resolution: &'a str,
    prompt_optimizer: bool,
}

#[derive(Deserialize)]
struct BaseResp {
    status_code: i32,
    #[serde(default)]
    status_msg: String,
}

#[derive(Deserialize)]
struct CreateTaskResp {
    #[serde(default)]
    task_id: String,
    base_resp: BaseResp,
}

#[derive(Deserialize)]
struct QueryTaskResp {
    #[serde(default)]
    status: String,
    #[serde(default)]
    file_id: String,
    base_resp: BaseResp,
}

#[derive(Default, Deserialize)]
struct FileInfo {
    #[serde(default)]
    download_url: String,
}

#[derive(Deserialize)]
struct RetrieveResp {
    #[serde(default)]
    file: FileInfo,
    base_resp: BaseResp,
}

pub enum TaskStatus {
    Pending,
    Processing,
    Success { file_id: String },
    Fail { reason: String },
}

pub async fn create_task(
    api_key: &str,
    prompt: &str,
    model: &str,
    duration: i64,
    resolution: &str,
    prompt_optimizer: bool,
) -> Result<String, String> {
    let resp = client(Duration::from_secs(60))?
        .post(format!("{BASE_URL}/v1/video_generation"))
        .bearer_auth(api_key)
        .json(&CreateTaskReq {
            model,
            prompt,
            duration,
            resolution,
            prompt_optimizer,
        })
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{e}"))?;
    let body: CreateTaskResp = resp.json().await.map_err(|e| format!("解析响应失败：{e}"))?;
    check_base_resp(body.base_resp.status_code, &body.base_resp.status_msg)?;
    if body.task_id.is_empty() {
        return Err("API 未返回任务 ID".to_string());
    }
    Ok(body.task_id)
}

pub async fn query_task(api_key: &str, task_id: &str) -> Result<TaskStatus, String> {
    let resp = client(Duration::from_secs(30))?
        .get(format!("{BASE_URL}/v1/query/video_generation"))
        .bearer_auth(api_key)
        .query(&[("task_id", task_id)])
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{e}"))?;
    let body: QueryTaskResp = resp.json().await.map_err(|e| format!("解析响应失败：{e}"))?;
    check_base_resp(body.base_resp.status_code, &body.base_resp.status_msg)?;
    match body.status.as_str() {
        "Success" => Ok(TaskStatus::Success {
            file_id: body.file_id,
        }),
        "Fail" => Ok(TaskStatus::Fail {
            reason: if body.base_resp.status_msg.is_empty() {
                "生成失败".to_string()
            } else {
                body.base_resp.status_msg.clone()
            },
        }),
        "Processing" => Ok(TaskStatus::Processing),
        _ => Ok(TaskStatus::Pending),
    }
}

pub async fn get_download_url(api_key: &str, file_id: &str) -> Result<String, String> {
    let resp = client(Duration::from_secs(30))?
        .get(format!("{BASE_URL}/v1/files/retrieve"))
        .bearer_auth(api_key)
        .query(&[("file_id", file_id)])
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{e}"))?;
    let body: RetrieveResp = resp.json().await.map_err(|e| format!("解析响应失败：{e}"))?;
    check_base_resp(body.base_resp.status_code, &body.base_resp.status_msg)?;
    if body.file.download_url.is_empty() {
        return Err("未获取到下载地址（链接可能已过期，请重试）".to_string());
    }
    Ok(body.file.download_url)
}

pub async fn download_file(url: &str, dest: &Path) -> Result<u64, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("下载请求失败：{e}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载失败，HTTP 状态码 {}", resp.status()));
    }
    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("创建文件失败：{e}"))?;
    let mut stream = resp.bytes_stream();
    let mut total: u64 = 0;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("下载中断：{e}"))?;
        file.write_all(&bytes)
            .await
            .map_err(|e| format!("写入文件失败：{e}"))?;
        total += bytes.len() as u64;
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(total)
}

const TEXT_MODEL: &str = "MiniMax-M2.5-highspeed";

const POLISH_SYSTEM: &str = "你是一位顶尖的 AI 视频提示词专家，精通 MiniMax Hailuo 视频生成模型的提示词写法。请把用户的视频创意扩写成一段高质量中文视频生成提示词，要求：
1. 忠实保留用户的核心创意与主体内容，不得改变事实设定；
2. 补充画面细节：主体外观与动作、场景环境、光线氛围、色彩基调、镜头语言；运镜必须使用官方英文指令语法：[Push in]、[Pull out]、[Pan left]、[Pan right]、[Truck left]、[Truck right]、[Pedestal up]、[Pedestal down]、[Tilt up]、[Tilt down]、[Zoom in]、[Zoom out]、[Tracking shot]、[Shake]、[Static shot]；同一个方括号内最多组合 3 个指令同时生效，多个方括号指令按出现顺序执行，全段 1~3 处运镜即可，不要堆砌；
3. 语言精炼生动，直接描述画面，禁止输出任何解释、前言、总结或标题；
4. 全文控制在 80~200 字；
5. 只输出润色后的提示词本身。";

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ChatReq<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    top_p: f32,
    max_completion_tokens: i64,
}

#[derive(Deserialize)]
struct ChatMessageResp {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct ChatChoiceResp {
    message: ChatMessageResp,
}

#[derive(Deserialize)]
struct ChatResp {
    #[serde(default)]
    choices: Vec<ChatChoiceResp>,
    base_resp: BaseResp,
}

pub async fn optimize_prompt(api_key: &str, prompt: &str, style: &str) -> Result<String, String> {
    let mut user_content = format!("视频创意：{prompt}");
    if !style.trim().is_empty() {
        user_content.push_str(&format!("\n风格基调：{}", style.trim()));
    }
    let resp = client(Duration::from_secs(90))?
        .post(format!("{BASE_URL}/v1/text/chatcompletion_v2"))
        .bearer_auth(api_key)
        .json(&ChatReq {
            model: TEXT_MODEL,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: POLISH_SYSTEM,
                },
                ChatMessage {
                    role: "user",
                    content: &user_content,
                },
            ],
            temperature: 1.0,
            top_p: 0.95,
            max_completion_tokens: 4096,
        })
        .send()
        .await
        .map_err(|e| format!("网络请求失败：{e}"))?;
    let body: ChatResp = resp.json().await.map_err(|e| format!("解析响应失败：{e}"))?;
    check_base_resp(body.base_resp.status_code, &body.base_resp.status_msg)?;
    let content = body
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();
    if content.is_empty() {
        return Err("模型未返回内容，请重试".to_string());
    }
    Ok(content)
}
