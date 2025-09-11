// 下载相关命令
use crate::api::client::ApiClient;
use std::time::UNIX_EPOCH;
use std::{fs::File, time::SystemTime};
use std::io::Write;
use crate::utils::auth::AuthManager;
use crate::config::AppConfig;
use tauri::{AppHandle, Manager, State};

// 下载 Picker 文件命令
#[tauri::command]
pub async fn download_picker(
    token: String,
    app: AppHandle,
    auth_manager: State<'_, AuthManager>,
) -> Result<String, String> {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let api_client = ApiClient::new(&config, Some(auth_manager.inner().clone()));
    
    // 下载文件内容
    let file_content = api_client.download("/download", &token).await.map_err(|e| e.to_string())?;
    
    // 获取下载目录
    let downloads_dir = app.path().download_dir()
        .map_err(|_| "无法获取下载目录".to_string())?;
    
    // 获取当前时间
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取当前时间失败: {}", e))?;

    // 生成随机文件名（在实际应用中应该从API获取文件名）
    let file_name = format!("picker_{}_{}.zip", token.chars().take(8).collect::<String>(), current_time.as_secs());
    let file_path = downloads_dir.join(file_name);
    
    // 写入文件
    let mut file = File::create(&file_path)
        .map_err(|e| format!("无法创建文件: {}", e))?;
    
    file.write_all(&file_content)
        .map_err(|e| format!("无法写入文件: {}", e))?;
    
    // 返回文件路径
    file_path.to_str()
        .ok_or_else(|| "无法转换文件路径".to_string())
        .map(String::from)
}