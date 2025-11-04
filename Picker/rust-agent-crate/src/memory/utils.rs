// 记忆系统工具函数模块
use std::path::{Path, PathBuf};
use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use log::warn;

/// 确保数据目录存在
pub async fn ensure_data_dir_exists(data_dir: &Path) -> Result<()> {
    if !data_dir.exists() {
        tokio::fs::create_dir_all(data_dir).await?;
    }
    Ok(())
}

/// 估算文本的 token 数量
/// 这是一个简化的实现，实际应用中可以使用更精确的 token 计算器
pub fn estimate_token_count(text: &str) -> usize {
    // 简化实现：假设平均每个 token 约 4 个字符
    // 这对于英文来说是一个合理的近似值，但对于中文可能不准确
    text.len() / 4
}

/// 估算文本的 token 数量（区分中文字符）
/// 对于中文字符，1字符≈1token；对于非中文字符，4字符≈1token
pub fn estimate_text_tokens(text: &str) -> usize {
    // 简化实现：假设平均每个 token 约 4 个字符
    // 对于英文，这个假设比较准确；对于中文，1个字符约等于1个token
    // 这里我们采用一个混合策略
    let chinese_chars = text.chars().filter(|c| {
        let c = *c as u32;
        // 中文字符的Unicode范围
        (0x4E00..=0x9FFF).contains(&c) || 
        (0x3400..=0x4DBF).contains(&c) || 
        (0x20000..=0x2A6DF).contains(&c) ||
        (0x2A700..=0x2B73F).contains(&c) ||
        (0x2B740..=0x2B81F).contains(&c) ||
        (0x2B820..=0x2CEAF).contains(&c) ||
        (0xF900..=0xFAFF).contains(&c) ||
        (0x2F800..=0x2FA1F).contains(&c)
    }).count();
    
    let non_chinese_chars = text.chars().count() - chinese_chars;
    
    // 中文字符：1字符≈1token，非中文字符：4字符≈1token
    chinese_chars + non_chinese_chars / 4
}

/// 估算 JSON 值的 token 数量
pub fn estimate_json_token_count(value: &Value) -> usize {
    match value {
        Value::String(s) => estimate_token_count(s),
        Value::Number(_) => 1, // 数字通常算作一个 token
        Value::Bool(_) => 1,   // 布尔值通常算作一个 token
        Value::Null => 1,      // null 通常算作一个 token
        Value::Array(arr) => {
            // 数组的 token 数量是所有元素之和加上方括号和逗号
            let mut count = 2; // 方括号
            for item in arr {
                count += estimate_json_token_count(item) + 1; // 加上逗号
            }
            count
        }
        Value::Object(obj) => {
            // 对象的 token 数量是所有键值对之和加上花括号和冒号
            let mut count = 2; // 花括号
            for (key, value) in obj {
                count += estimate_token_count(key) + 1; // 键和冒号
                count += estimate_json_token_count(value) + 1; // 值和逗号
            }
            count
        }
    }
}

/// 序列化 JSON 值到字符串，带错误处理
pub fn serialize_to_string(value: &Value) -> Result<String> {
    serde_json::to_string(value).map_err(|e| {
        warn!("Failed to serialize JSON value: {}", e);
        Error::from(e)
    })
}

/// 反序列化 JSON 字符串到值，带错误处理
pub fn deserialize_from_str(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str).map_err(|e| {
        warn!("Failed to deserialize JSON string: {}", e);
        Error::from(e)
    })
}

/// 生成当前时间戳 (ISO 8601 格式)
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// 解析时间戳字符串
pub fn parse_timestamp(timestamp: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    timestamp.parse::<chrono::DateTime<chrono::Utc>>().map_err(|e| {
        warn!("Failed to parse timestamp '{}': {}", timestamp, e);
        Error::from(e)
    })
}

/// 获取会话文件路径
pub fn get_session_file_path(data_dir: &Path, session_id: &str, suffix: &str) -> PathBuf {
    data_dir.join(format!("{}_{}", session_id, suffix))
}

/// 创建带时间戳的备份文件路径
pub fn create_backup_path(file_path: &Path) -> PathBuf {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
    let file_stem = file_path.file_stem().unwrap_or_else(|| std::ffi::OsStr::new("backup"));
    let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    if extension.is_empty() {
        parent.join(format!("{}_{}.backup", file_stem.to_string_lossy(), timestamp))
    } else {
        parent.join(format!("{}_{}_{}.backup", file_stem.to_string_lossy(), timestamp, extension))
    }
}

/// 异步读取文件内容，带错误处理
pub async fn read_file_content(file_path: &Path) -> Result<String> {
    tokio::fs::read_to_string(file_path).await.map_err(|e| {
        warn!("Failed to read file '{}': {}", file_path.display(), e);
        Error::from(e)
    })
}

/// 异步写入文件内容，带错误处理
pub async fn write_file_content(file_path: &Path, content: &str) -> Result<()> {
    tokio::fs::write(file_path, content).await.map_err(|e| {
        warn!("Failed to write file '{}': {}", file_path.display(), e);
        Error::from(e)
    })
}

/// 原子写入文件内容（先写入临时文件，然后重命名）
pub async fn atomic_write_file(file_path: &Path, content: &str) -> Result<()> {
    // 创建临时文件路径
    let temp_path = file_path.with_extension("tmp");
    
    // 确保父目录存在
    if let Some(parent) = file_path.parent() {
        ensure_data_dir_exists(parent).await?;
    }
    
    // 写入临时文件
    write_file_content(&temp_path, content).await?;
    
    // 原子重命名
    tokio::fs::rename(&temp_path, file_path).await.map_err(|e| {
        warn!("Failed to rename temporary file to '{}': {}", file_path.display(), e);
        Error::from(e)
    })?;
    
    Ok(())
}

/// 追加内容到文件，带错误处理
pub async fn append_to_file(file_path: &Path, content: &str) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    
    // 确保父目录存在
    if let Some(parent) = file_path.parent() {
        ensure_data_dir_exists(parent).await?;
    }
    
    // 打开文件并追加内容
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .await?;
    
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    
    Ok(())
}

/// 检查文件是否存在
pub async fn file_exists(file_path: &Path) -> bool {
    tokio::fs::metadata(file_path).await.is_ok()
}

/// 删除文件，带错误处理
pub async fn delete_file(file_path: &Path) -> Result<()> {
    if file_exists(file_path).await {
        tokio::fs::remove_file(file_path).await.map_err(|e| {
            warn!("Failed to delete file '{}': {}", file_path.display(), e);
            Error::from(e)
        })?;
    }
    Ok(())
}

/// 创建目录（如果不存在）
pub async fn ensure_dir_exists(dir_path: &Path) -> Result<()> {
    if !dir_path.exists() {
        tokio::fs::create_dir_all(dir_path).await.map_err(|e| {
            warn!("Failed to create directory '{}': {}", dir_path.display(), e);
            Error::from(e)
        })?;
    }
    Ok(())
}

/// 获取环境变量值，如果不存在则返回默认值
pub fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 从环境变量获取数据目录路径
pub fn get_data_dir_from_env() -> PathBuf {
    let default_dir = "./data/memory";
    let dir_str = get_env_var("MEMORY_DATA_DIR", default_dir);
    PathBuf::from(dir_str)
}

/// 从环境变量获取摘要阈值
pub fn get_summary_threshold_from_env() -> usize {
    let default_threshold = 3500;
    let threshold_str = get_env_var("MEMORY_SUMMARY_THRESHOLD", &default_threshold.to_string());
    threshold_str.parse().unwrap_or(default_threshold)
}

/// 从环境变量获取最近消息数量
pub fn get_recent_messages_count_from_env() -> usize {
    let default_count = 10;
    let count_str = get_env_var("MEMORY_RECENT_MESSAGES_COUNT", &default_count.to_string());
    count_str.parse().unwrap_or(default_count)
}

/// 生成随机会话 ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_ensure_data_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        
        assert!(!dir_path.exists());
        ensure_data_dir_exists(&dir_path).await.unwrap();
        assert!(dir_path.exists());
    }
    
    #[test]
    fn test_estimate_token_count() {
        assert_eq!(estimate_token_count(""), 0);
        assert_eq!(estimate_token_count("hello world"), 2); // 11 chars / 4 = 2 (floor)
        assert_eq!(estimate_token_count("a".repeat(20).as_str()), 5); // 20 chars / 4 = 5
    }
    
    #[test]
    fn test_current_timestamp() {
        let timestamp = current_timestamp();
        assert!(parse_timestamp(&timestamp).is_ok());
    }
    
    #[test]
    fn test_get_session_file_path() {
        let data_dir = Path::new("/tmp");
        let session_id = "test_session";
        let suffix = "json";
        
        let path = get_session_file_path(data_dir, session_id, suffix);
        assert_eq!(path, PathBuf::from("/tmp/test_session_json"));
    }
    
    #[test]
    fn test_get_env_var() {
        let key = "MEMORY_TEST_VAR";
        let default = "default_value";
        
        // 确保环境变量未设置
        std::env::remove_var(key);
        assert_eq!(get_env_var(key, default), default);
        
        // 设置环境变量
        std::env::set_var(key, "test_value");
        assert_eq!(get_env_var(key, default), "test_value");
        
        // 清理
        std::env::remove_var(key);
    }
    
    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID length
        assert_eq!(id2.len(), 36); // UUID length
    }
}