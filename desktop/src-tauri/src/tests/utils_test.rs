// 工具模块测试

use crate::api::models::UserInfo;
use crate::utils::auth::AuthManager;
use serde_json::json;
use tauri::{Manager, State};
use tauri_plugin_store::StoreBuilder;
use std::sync::Arc;
use std::time::SystemTime;

// 测试认证管理器的基本功能
#[test]
fn test_auth_manager_basic_functions() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 初始状态应该是未登录
    assert!(!auth_manager.is_logged_in());
    assert!(auth_manager.get_token().is_none());
    assert!(auth_manager.get_user_info().is_none());

    // 设置 token
    let token = "test_auth_token";
    auth_manager.set_token(token).unwrap();
    
    // 验证 token 被正确设置
    assert!(auth_manager.is_logged_in());
    assert_eq!(auth_manager.get_token().unwrap(), token);
    
    // 验证认证头被正确生成
    assert_eq!(auth_manager.get_auth_header().unwrap(), format!("Bearer {}", token));

    // 清除 token
    auth_manager.clear_token().unwrap();
    
    // 验证清除后的状态
    assert!(!auth_manager.is_logged_in());
    assert!(auth_manager.get_token().is_none());
    assert!(auth_manager.get_auth_header().is_none());
}

// 测试用户信息的存储和检索
#[test]
fn test_auth_manager_user_info() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 初始状态没有用户信息
    assert!(auth_manager.get_user_info().is_none());

    // 创建用户信息对象
    let user_info = UserInfo {
        user_id: "user_123".to_string(),
        email: "test@example.com".to_string(),
        user_name: "test_user".to_string(),
        user_type: crate::api::models::UserType::Gen,
        wallet_address: "0x123456789".to_string(),
        premium_balance: 100,
        created_at: "2023-01-01T00:00:00Z".to_string()
    };

    // 保存用户信息
    auth_manager.set_user_info(&user_info).unwrap();
    
    // 验证用户信息被正确保存
    let saved_user_info = auth_manager.get_user_info().unwrap();
    assert_eq!(saved_user_info.user_id, "user_123");
    assert_eq!(saved_user_info.email, "test@example.com");
    assert_eq!(saved_user_info.user_name, "test_user");
    assert_eq!(saved_user_info.wallet_address, "0x123456789");
    assert_eq!(saved_user_info.premium_balance, 100);

    // 清除 token 和用户信息
    auth_manager.clear_token().unwrap();
    
    // 验证清除后的状态
    assert!(auth_manager.get_user_info().is_none());
}

// 测试 JWT token 解析功能
#[test]
fn test_token_expiry_parsing() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 创建一个带有过期时间的 JWT token（使用 base64 编码的简单 claims）
    // 注意：这是一个简化的测试用例，真实的 JWT 解析需要更复杂的处理
    let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MDAwMDAwMDB9.test_signature";
    
    // 设置 token
    auth_manager.set_token(test_token).unwrap();
    
    // 验证过期时间被正确解析
    let expiry = auth_manager.get_token_expiry();
    assert!(expiry.is_some());
    assert_eq!(expiry.unwrap(), 1700000000);

    // 清理
    auth_manager.clear_token().unwrap();
}

// 测试 token 过期检查
#[test]
fn test_token_expired_check() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 创建一个已过期的 token（过期时间为 1970 年）
    let expired_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjB9.test_signature";
    
    // 设置过期的 token
    auth_manager.set_token(expired_token).unwrap();
    
    // 验证 token 被检测为已过期
    assert!(auth_manager.is_token_expired());

    // 清理
    auth_manager.clear_token().unwrap();

    // 创建一个未过期的 token（过期时间为未来）
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let future_expiry = current_time + 3600; // 1小时后过期
    
    // 构建包含未来过期时间的 token
    let future_token = format!("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOj{future_expiry}fQ.test_signature");
    
    // 设置未过期的 token
    auth_manager.set_token(&future_token).unwrap();
    
    // 验证 token 被检测为未过期
    assert!(!auth_manager.is_token_expired());

    // 清理
    auth_manager.clear_token().unwrap();
}

// 测试无效的 JWT token 处理
#[test]
fn test_invalid_token_handling() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 测试无效格式的 token
    let invalid_tokens = vec![
        "invalid_token",                         // 格式完全错误
        "only_one_part",                        // 只有一个部分
        "two.parts",                            // 只有两个部分
        "invalid.base64.payload.signature",     // 无效的 base64 编码
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid_json_payload.signature" // 无效的 JSON payload
    ];

    for token in invalid_tokens {
        // 设置无效 token
        auth_manager.set_token(token).unwrap();
        
        // 验证 token 状态
        assert!(auth_manager.is_logged_in()); // 只要有 token 就认为已登录
        assert_eq!(auth_manager.get_token().unwrap(), token);
        
        // 验证过期时间解析失败
        assert!(auth_manager.get_token_expiry().is_none());
        assert!(!auth_manager.is_token_expired()); // 对于无法解析的 token，认为未过期
        
        // 清理
        auth_manager.clear_token().unwrap();
    }
}

// 测试存储操作失败情况
#[test]
fn test_store_operations_error_handling() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 正常情况下的操作应该成功
    assert!(auth_manager.set_token("test_token").is_ok());
    assert!(auth_manager.clear_token().is_ok());
    
    // 测试保存用户信息
    let user_info = UserInfo {
        user_id: "user_123".to_string(),
        email: "test@example.com".to_string(),
        user_name: "test_user".to_string(),
        user_type: crate::api::models::UserType::Gen,
        wallet_address: "0x123456789".to_string(),
        premium_balance: 100,
        created_at: "2023-01-01T00:00:00Z".to_string()
    };
    assert!(auth_manager.set_user_info(&user_info).is_ok());
    
    // 清理
    auth_manager.clear_token().unwrap();
}

// 测试 AuthManager 在并发环境下的行为
#[test]
fn test_auth_manager_concurrent_access() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = Arc::new(AuthManager::new(State::new(store)));

    // 创建多个线程同时访问 AuthManager
    let handles: Vec<_> = (0..5).map(|i| {
        let auth_manager_clone = Arc::clone(&auth_manager);
        std::thread::spawn(move || {
            // 线程特定的 token
            let token = format!("thread_token_{}", i);
            
            // 设置 token
            auth_manager_clone.set_token(&token).unwrap();
            
            // 验证 token 设置正确
            assert_eq!(auth_manager_clone.get_token().unwrap(), token);
            
            // 清理
            if i == 4 {  // 最后一个线程负责清理
                auth_manager_clone.clear_token().unwrap();
            }
        })
    }).collect();

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    // 验证最后一个线程正确清理了 token
    assert!(!auth_manager.is_logged_in());
    assert!(auth_manager.get_token().is_none());
}

// 测试不同类型的用户信息存储和检索
#[test]
fn test_different_user_types() {
    // 创建测试应用和 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth_test.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));

    // 测试不同类型的用户
    let user_types = vec![
        crate::api::models::UserType::Gen,
        crate::api::models::UserType::Dev,
        crate::api::models::UserType::Admin
    ];

    for user_type in user_types {
        let user_info = UserInfo {
            user_id: format!("user_{:?}", user_type),
            email: format!("{:?}@example.com", user_type),
            user_name: format!("{:?}_user", user_type),
            user_type: user_type.clone(),
            wallet_address: "0x123456789".to_string(),
            premium_balance: 100,
            created_at: "2023-01-01T00:00:00Z".to_string()
        };

        // 保存用户信息
        auth_manager.set_user_info(&user_info).unwrap();
        
        // 验证用户信息被正确保存
        let saved_user_info = auth_manager.get_user_info().unwrap();
        assert_eq!(saved_user_info.user_id, user_info.user_id);
        assert_eq!(saved_user_info.email, user_info.email);
        assert_eq!(saved_user_info.user_type, user_type);
        
        // 清理
        auth_manager.clear_token().unwrap();
    }
}