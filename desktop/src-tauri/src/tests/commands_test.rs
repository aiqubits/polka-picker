// 命令模块测试

use crate::commands::users::{login, register, verify_email, get_user_profile, logout, check_login_status, get_current_user_info};
use crate::commands::pickers::{get_picker_marketplace, get_picker_detail, upload_picker};
use crate::commands::orders::{get_user_orders, create_order, get_order_detail};
use crate::commands::download::download_picker;
use crate::api::models::{LoginResponse, UserInfo, UserType, PickerListResponse, PickerDetail, OrderListResponse, OrderInfo};
use crate::utils::auth::AuthManager;
use mockito::{mock, Server};
use serde_json::json;
use tauri::{App, AppHandle, Manager, State};
use tauri_plugin_store::StoreBuilder;
use tokio::sync::Arc;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;

// 帮助函数：创建测试应用和 AuthManager
fn setup_test_app() -> (AppHandle, AuthManager) {
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));
    (app.handle().clone(), auth_manager)
}

// 测试登录命令
#[tokio::test]
async fn test_login_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("POST", "/api/users/login")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "token": "login_token",
            "user": {
                "user_id": "user_123",
                "email": "test@example.com",
                "user_name": "test_user",
                "user_type": "gen",
                "wallet_address": "0x123",
                "premium_balance": 100,
                "created_at": "2023-01-01T00:00:00Z"
            }
        })).unwrap())
        .create_async()
        .await;

    // 创建测试应用和 AuthManager
    let (_app, auth_manager) = setup_test_app();

    // 执行登录命令
    let result = login(
        "test@example.com".to_string(),
        "password123".to_string(),
        State::new(auth_manager.clone()),
    ).await;

    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.token, "login_token");
    assert_eq!(response.user.user_id, "user_123");
    
    // 验证 token 被正确保存
    assert_eq!(auth_manager.get_token().unwrap(), "login_token");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试登录失败情况
#[tokio::test]
async fn test_login_command_failure() {
    // 设置 mock 服务器返回错误
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("POST", "/api/users/login")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "error": "Unauthorized",
            "message": "Invalid credentials"
        })).unwrap())
        .create_async()
        .await;

    // 创建测试应用和 AuthManager
    let (_app, auth_manager) = setup_test_app();

    // 执行登录命令（使用错误的密码）
    let result = login(
        "test@example.com".to_string(),
        "wrong_password".to_string(),
        State::new(auth_manager.clone()),
    ).await;

    // 验证结果是错误
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("Unauthorized"));
    
    // 验证 token 没有被保存
    assert!(auth_manager.get_token().is_none());
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试注册命令
#[tokio::test]
async fn test_register_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("POST", "/api/users/register")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create_async()
        .await;

    // 执行注册命令
    let result = register(
        "newuser@example.com".to_string(),
        "password123".to_string(),
        "newuser".to_string(),
        "gen".to_string(),
        "0x456".to_string(),
    ).await;

    // 验证结果
    assert!(result.is_ok());
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试邮箱验证命令
#[tokio::test]
async fn test_verify_email_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("POST", "/api/users/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{}")
        .create_async()
        .await;

    // 执行邮箱验证命令
    let result = verify_email(
        "test@example.com".to_string(),
        "123456".to_string(),
    ).await;

    // 验证结果
    assert!(result.is_ok());
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试获取用户资料命令
#[tokio::test]
async fn test_get_user_profile_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("GET", "/api/users/profile")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "user_id": "user_123",
            "email": "test@example.com",
            "user_name": "test_user",
            "user_type": "gen",
            "wallet_address": "0x123",
            "premium_balance": 100,
            "created_at": "2023-01-01T00:00:00Z"
        })).unwrap())
        .match_header("Authorization", "Bearer test_token")
        .create_async()
        .await;

    // 创建测试应用和 AuthManager，并设置 token
    let (_app, auth_manager) = setup_test_app();
    auth_manager.set_token("test_token").unwrap();

    // 执行获取用户资料命令
    let result = get_user_profile(State::new(auth_manager.clone())).await;

    // 验证结果
    assert!(result.is_ok());
    let user_info = result.unwrap();
    assert_eq!(user_info.user_id, "user_123");
    assert_eq!(user_info.email, "test@example.com");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试登出命令
#[tokio::test]
async fn test_logout_command() {
    // 创建测试应用和 AuthManager
    let (_app, auth_manager) = setup_test_app();
    
    // 先设置一个 token，模拟已登录状态
    let test_token = "test_token_value";
    auth_manager.set_token(test_token).unwrap();
    
    // 验证初始状态是已登录
    assert!(auth_manager.get_token().is_some());
    
    // 调用 logout 命令
    let result = logout(State::new(auth_manager.clone())).await;
    
    // 验证命令执行成功
    assert!(result.is_ok());
    
    // 验证登录状态已清除
    assert!(auth_manager.get_token().is_none());
}

// 测试检查登录状态命令
#[tokio::test]
async fn test_check_login_status_command() {
    // 创建测试应用和 AuthManager
    let (_app, auth_manager) = setup_test_app();
    
    // 创建 AuthManager 的 State 实例
    let auth_manager_state = State::new(auth_manager.clone());
    
    // 测试未登录状态
    let result = check_login_status(auth_manager_state.clone()).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // 设置 token，模拟已登录状态
    let test_token = "test_token_value";
    auth_manager.set_token(test_token).unwrap();
    
    // 测试已登录状态
    let result = check_login_status(auth_manager_state).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

// 测试获取当前用户信息命令
#[tokio::test]
async fn test_get_current_user_info_command() {
    // 创建测试应用和 AuthManager
    let (_app, auth_manager) = setup_test_app();
    
    // 创建 AuthManager 的 State 实例
    let auth_manager_state = State::new(auth_manager.clone());
    
    // 测试未登录状态
    let result = get_current_user_info(auth_manager_state.clone()).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // 设置用户信息，模拟已登录状态
    let test_token = "test_token_value";
    let test_user_info = UserInfo {
        user_id: "user123".to_string(),
        email: "test@example.com".to_string(),
        user_name: "Test User".to_string(),
        user_type: UserType::Gen,
        wallet_address: "0x123".to_string(),
        premium_balance: 100,
        created_at: "2023-01-01T00:00:00Z".to_string()
    };
    auth_manager.set_token(test_token).unwrap();
    auth_manager.set_user_info(&test_user_info).unwrap();
    
    // 测试已登录状态
    let result = get_current_user_info(auth_manager_state).await;
    assert!(result.is_ok());
    let user_info = result.unwrap().unwrap();
    assert_eq!(user_info.user_id, "user123");
    assert_eq!(user_info.email, "test@example.com");
}

// 测试获取 Picker 市场列表命令
#[tokio::test]
async fn test_get_picker_marketplace_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("GET", "/api/pickers?page=1&size=10")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "items": [
                {
                    "picker_id": "picker_1",
                    "name": "Test Picker 1",
                    "description": "A test picker",
                    "price": 50,
                    "creator": "creator_1",
                    "rating": 4.5,
                    "downloads": 100
                }
            ],
            "total": 1,
            "page": 1,
            "size": 10
        })).unwrap())
        .create_async()
        .await;

    // 执行获取 Picker 市场列表命令
    let result = get_picker_marketplace(Some(1), Some(10), None).await;

    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].picker_id, "picker_1");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试获取 Picker 详情命令
#[tokio::test]
async fn test_get_picker_detail_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("GET", "/api/pickers/picker_1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "picker_id": "picker_1",
            "name": "Test Picker 1",
            "description": "A test picker",
            "price": 50,
            "creator": "creator_1",
            "rating": 4.5,
            "downloads": 100
        })).unwrap())
        .create_async()
        .await;

    // 执行获取 Picker 详情命令
    let result = get_picker_detail("picker_1".to_string()).await;

    // 验证结果
    assert!(result.is_ok());
    let picker = result.unwrap();
    assert_eq!(picker.picker_id, "picker_1");
    assert_eq!(picker.name, "Test Picker 1");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试获取用户订单列表命令
#[tokio::test]
async fn test_get_user_orders_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("GET", "/api/orders?page=1&size=10")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "items": [
                {
                    "order_id": "order_123",
                    "user_id": "user_123",
                    "picker_id": "picker_123",
                    "picker_name": "Test Picker",
                    "price": 50,
                    "status": "completed",
                    "created_at": "2023-01-01T00:00:00Z"
                }
            ],
            "total": 1,
            "page": 1,
            "size": 10
        })).unwrap())
        .match_header("Authorization", "Bearer test_token")
        .create_async()
        .await;

    // 创建测试应用和 AuthManager，并设置 token
    let (_app, auth_manager) = setup_test_app();
    auth_manager.set_token("test_token").unwrap();

    // 执行获取用户订单列表命令
    let result = get_user_orders(Some(1), Some(10), None, State::new(auth_manager.clone())).await;

    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].order_id, "order_123");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试创建订单命令
#[tokio::test]
async fn test_create_order_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("POST", "/api/orders")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "order_id": "order_123",
            "user_id": "user_123",
            "picker_id": "picker_123",
            "picker_name": "Test Picker",
            "price": 50,
            "status": "pending",
            "created_at": "2023-01-01T00:00:00Z"
        })).unwrap())
        .match_header("Authorization", "Bearer test_token")
        .create_async()
        .await;

    // 创建测试应用和 AuthManager，并设置 token
    let (_app, auth_manager) = setup_test_app();
    auth_manager.set_token("test_token").unwrap();

    // 执行创建订单命令
    let result = create_order("picker_123".to_string(), State::new(auth_manager.clone())).await;

    // 验证结果
    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.order_id, "order_123");
    assert_eq!(order.picker_id, "picker_123");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试获取订单详情命令
#[tokio::test]
async fn test_get_order_detail_command() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    let mock = mock_server
        .mock("GET", "/api/orders/order_123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "order_id": "order_123",
            "user_id": "user_123",
            "picker_id": "picker_123",
            "picker_name": "Test Picker",
            "price": 50,
            "status": "completed",
            "created_at": "2023-01-01T00:00:00Z"
        })).unwrap())
        .match_header("Authorization", "Bearer test_token")
        .create_async()
        .await;

    // 创建测试应用和 AuthManager，并设置 token
    let (_app, auth_manager) = setup_test_app();
    auth_manager.set_token("test_token").unwrap();

    // 执行获取订单详情命令
    let result = get_order_detail("order_123".to_string(), State::new(auth_manager.clone())).await;

    // 验证结果
    assert!(result.is_ok());
    let order = result.unwrap();
    assert_eq!(order.order_id, "order_123");
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
}

// 测试下载 Picker 文件命令
#[tokio::test]
async fn test_download_picker_command() {
    // 创建临时目录作为下载目录
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_str().unwrap().to_string();
    
    // 设置环境变量以使用临时目录作为下载目录
    std::env::set_var("DOWNLOAD_DIR", &temp_dir_path);
    
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 设置环境变量以使用 mock 服务器
    std::env::set_var("API_BASE_URL", &mock_url);
    
    // 模拟文件内容
    let mock_file_content = b"This is a test file content";
    
    let mock = mock_server
        .mock("GET", "/download?token=download_token")
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body(mock_file_content)
        .match_header("Authorization", "Bearer test_token")
        .create_async()
        .await;

    // 创建测试应用和 AuthManager，并设置 token
    let (_app, auth_manager) = setup_test_app();
    auth_manager.set_token("test_token").unwrap();

    // 执行下载 Picker 文件命令
    let result = download_picker("download_token".to_string(), _app, State::new(auth_manager.clone())).await;

    // 验证结果
    assert!(result.is_ok());
    let file_path = result.unwrap();
    
    // 验证文件存在
    assert!(std::path::Path::new(&file_path).exists());
    
    // 验证文件内容
    let content = std::fs::read(&file_path).unwrap();
    assert_eq!(&content, mock_file_content);
    
    // 验证 mock 被调用
    mock.assert_async().await;
    
    // 清理环境变量
    std::env::remove_var("API_BASE_URL");
    std::env::remove_var("DOWNLOAD_DIR");
}