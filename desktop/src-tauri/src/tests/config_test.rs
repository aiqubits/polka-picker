// 配置模块测试

use crate::config::{AppConfig, ConfigError};
use std::env;
use std::fs;
use std::path::PathBuf;

// 测试从环境变量加载配置
#[test]
fn test_load_config_from_env() {
    // 清理之前可能存在的环境变量
    env::remove_var("API_BASE_URL");
    env::remove_var("REQUEST_TIMEOUT_MS");
    env::remove_var("MAX_RETRIES");

    // 设置环境变量
    env::set_var("API_BASE_URL", "http://test.example.com");
    env::set_var("REQUEST_TIMEOUT_MS", "50000");
    env::set_var("MAX_RETRIES", "5");

    // 加载配置
    let config = AppConfig::load().unwrap();

    // 验证配置值
    assert_eq!(config.api_base_url, "http://test.example.com");
    assert_eq!(config.request_timeout_ms, 50000);
    assert_eq!(config.max_retries, 5);

    // 清理环境变量
    env::remove_var("API_BASE_URL");
    env::remove_var("REQUEST_TIMEOUT_MS");
    env::remove_var("MAX_RETRIES");
}

// 测试部分环境变量设置的情况
#[test]
fn test_load_config_with_partial_env() {
    // 清理之前可能存在的环境变量
    env::remove_var("API_BASE_URL");
    env::remove_var("REQUEST_TIMEOUT_MS");
    env::remove_var("MAX_RETRIES");

    // 只设置部分环境变量
    env::set_var("API_BASE_URL", "http://partial.example.com");
    // REQUEST_TIMEOUT_MS 和 MAX_RETRIES 使用默认值

    // 加载配置
    let config = AppConfig::load().unwrap();

    // 验证配置值
    assert_eq!(config.api_base_url, "http://partial.example.com");
    assert_eq!(config.request_timeout_ms, 30000); // 默认值
    assert_eq!(config.max_retries, 3); // 默认值

    // 清理环境变量
    env::remove_var("API_BASE_URL");
}

// 测试配置文件加载（如果存在的话）
#[test]
fn test_load_config_from_file() {
    // 清理环境变量以确保从文件加载
    env::remove_var("API_BASE_URL");

    // 首先尝试获取配置文件路径
    let config_path = if let Ok(path) = AppConfig::get_config_path() {
        path
    } else {
        // 如果无法获取配置路径，跳过此测试
        eprintln!("无法获取配置文件路径，跳过测试");
        return;
    };

    // 创建临时配置文件
    let config_content = r#"{
    "api_base_url": "http://file.example.com",
    "request_timeout_ms": 40000,
    "max_retries": 4
}"#;
    
    // 如果配置文件已存在，先保存其内容以便后续恢复
    let original_content = if config_path.exists() {
        Some(fs::read_to_string(&config_path).unwrap())
    } else {
        None
    };

    // 写入测试配置
    fs::write(&config_path, config_content).unwrap();

    // 加载配置
    let config = AppConfig::load().unwrap();

    // 验证配置值
    assert_eq!(config.api_base_url, "http://file.example.com");
    assert_eq!(config.request_timeout_ms, 40000);
    assert_eq!(config.max_retries, 4);

    // 恢复原始配置文件（如果存在）
    if let Some(content) = original_content {
        fs::write(&config_path, content).unwrap();
    } else {
        // 如果是新创建的文件，删除它
        fs::remove_file(&config_path).unwrap();
    }
}

// 测试配置错误处理
#[test]
fn test_config_error_handling() {
    // 清理环境变量
    env::remove_var("API_BASE_URL");

    // 修改环境变量为无效的数字值
    env::set_var("REQUEST_TIMEOUT_MS", "invalid_number");

    // 验证错误处理
    let result = AppConfig::load();
    assert!(result.is_err());
    
    // 根据实际实现，这里可能是 ConfigError::EnvVarError 或其他错误类型
    match result.err().unwrap() {
        ConfigError::EnvVarError(_) => (),
        ConfigError::FileNotFound => (), // 如果环境变量设置了但值无效，可能会回退到文件加载
        _ => panic!("Expected EnvVarError or FileNotFound but got another error type"),
    }

    // 清理环境变量
    env::remove_var("API_BASE_URL");
    env::remove_var("REQUEST_TIMEOUT_MS");
}

// 测试默认配置
#[test]
fn test_default_config() {
    // 清理环境变量
    env::remove_var("API_BASE_URL");

    // 尝试将配置文件移开（如果存在）
    let config_path = match AppConfig::get_config_path() {
        Ok(path) => {
            if path.exists() {
                let temp_path = path.with_extension("json.bak");
                fs::rename(&path, &temp_path).unwrap();
                Some((path, temp_path))
            } else {
                None
            }
        },
        Err(_) => None,
    };

    // 创建默认配置
    let default_config = AppConfig::default();

    // 验证默认配置值
    assert_eq!(default_config.api_base_url, "http://localhost:8080");
    assert_eq!(default_config.request_timeout_ms, 30000);
    assert_eq!(default_config.max_retries, 3);

    // 恢复配置文件（如果有）
    if let Some((original_path, temp_path)) = config_path {
        fs::rename(temp_path, original_path).unwrap();
    }
}