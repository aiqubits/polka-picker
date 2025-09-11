// API 模块测试

use crate::api::client::ApiClient;
use crate::api::models::{ApiError, LoginRequest, LoginResponse, UserInfo, UserType, PickerListResponse, PickerDetail, OrderListResponse, OrderInfo};
use crate::config::AppConfig;
use crate::utils::auth::AuthManager;
use mockito::{mock, Server};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;
use tauri_plugin_store::StoreBuilder;
use tokio::time::sleep;

// 用于测试的示例结构体
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    id: String,
    name: String,
    value: i32,
}

// 测试 API 客户端的基本 POST 请求功能
#[tokio::test]
async fn test_api_client_post() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("POST", "/api/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&TestData {
            id: "test-123".to_string(),
            name: "Test Data".to_string(),
            value: 42,
        }).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求
    let request = TestData {
        id: "test-123".to_string(),
        name: "Test Data".to_string(),
        value: 42,
    };
    let response: TestData = api_client.post("/api/test", &request).await.unwrap();

    // 验证响应
    assert_eq!(response, request);
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端的 GET 请求功能
#[tokio::test]
async fn test_api_client_get() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("GET", "/api/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&TestData {
            id: "test-123".to_string(),
            name: "Test Data".to_string(),
            value: 42,
        }).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求
    let response: TestData = api_client.get("/api/test", None).await.unwrap();

    // 验证响应
    assert_eq!(response.id, "test-123");
    assert_eq!(response.name, "Test Data");
    assert_eq!(response.value, 42);
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端的 PUT 请求功能
#[tokio::test]
async fn test_api_client_put() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("PUT", "/api/test/test-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({"updated": true})).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求
    let request = TestData {
        id: "test-123".to_string(),
        name: "Updated Data".to_string(),
        value: 100,
    };
    let response: HashMap<String, bool> = api_client.put("/api/test/test-123", &request).await.unwrap();

    // 验证响应
    assert_eq!(response.get("updated"), Some(&true));
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端的 DELETE 请求功能
#[tokio::test]
async fn test_api_client_delete() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("DELETE", "/api/test/test-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({"deleted": true})).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求
    let response: HashMap<String, bool> = api_client.delete("/api/test/test-123").await.unwrap();

    // 验证响应
    assert_eq!(response.get("deleted"), Some(&true));
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端处理查询参数
#[tokio::test]
async fn test_api_client_with_query_params() {
    // 设置 mock 服务器
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("GET", "/api/test?page=1&size=10&search=test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "page": 1,
            "size": 10,
            "search": "test",
            "total": 0,
            "items": []
        })).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 准备查询参数
    let mut query_params = HashMap::new();
    query_params.insert("page", "1".to_string());
    query_params.insert("size", "10".to_string());
    query_params.insert("search", "test".to_string());

    // 执行请求
    let response: HashMap<String, serde_json::Value> = api_client.get("/api/test", Some(&query_params)).await.unwrap();

    // 验证响应
    assert_eq!(response.get("page").unwrap().as_i64().unwrap(), 1);
    assert_eq!(response.get("size").unwrap().as_i64().unwrap(), 10);
    assert_eq!(response.get("search").unwrap().as_str().unwrap(), "test");
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端处理错误响应
#[tokio::test]
async fn test_api_client_error_response() {
    // 设置 mock 服务器返回错误
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("POST", "/api/auth/login")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "error": "Unauthorized", 
            "message": "Invalid credentials"
        })).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求并验证错误
    let request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "wrong_password".to_string(),
    };
    let result = api_client.post("/api/auth/login", &request).await;
    
    // 验证结果是错误
    assert!(result.is_err());
    match result.err().unwrap() {
        ApiError::AuthError(msg) => assert!(msg.contains("Unauthorized")),
        _ => panic!("Expected AuthError but got another error type"),
    }
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端带认证头的请求
#[tokio::test]
async fn test_api_client_with_auth() {
    // 设置 mock 服务器验证认证头
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("GET", "/api/users/profile")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "user_id": "123",
            "email": "test@example.com",
            "user_name": "test_user",
            "user_type": "gen",
            "wallet_address": "0x123",
            "premium_balance": 0,
            "created_at": "2023-01-01T00:00:00Z"
        })).unwrap())
        .create_async()
        .await;

    // 创建模拟的 AuthManager
    let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
    let store = StoreBuilder::new(app.handle(), "auth.json").build().unwrap();
    let auth_manager = AuthManager::new(State::new(store));
    
    // 设置测试 token
    auth_manager.set_token("test_token").unwrap();

    // 创建配置和带认证的 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 3,
    };
    let api_client = ApiClient::new(&config, Some(auth_manager));

    // 执行带认证的请求
    let result: UserInfo = api_client.get("/api/users/profile", None).await.unwrap();

    // 验证结果
    assert_eq!(result.user_id, "123");
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端处理服务器错误
#[tokio::test]
async fn test_api_client_server_error() {
    // 设置 mock 服务器返回 500 错误
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    let mock = mock_server
        .mock("GET", "/api/error")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&json!({
            "error": "Internal Server Error", 
            "message": "Something went wrong"
        })).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 1,  // 减少重试次数以加速测试
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求并验证错误
    let result: Result<TestData, ApiError> = api_client.get("/api/error", None).await;
    
    // 验证结果是错误
    assert!(result.is_err());
    match result.err().unwrap() {
        ApiError::ServerError(msg) => assert!(msg.contains("Internal Server Error")),
        _ => panic!("Expected ServerError but got another error type"),
    }
    
    // 验证 mock 被调用
    mock.assert_async().await;
}

// 测试 API 客户端的重试机制
#[tokio::test]
async fn test_api_client_retry_mechanism() {
    // 设置 mock 服务器，第一次返回 500 错误，第二次返回成功
    let mock_server = Server::new_async().await;
    let mock_url = mock_server.url();
    
    // 第一个 mock 返回错误
    let mock_error = mock_server
        .mock("GET", "/api/retry")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body("{\"error\": \"Internal Server Error\"}")
        .create_async()
        .await;
    
    // 第二个 mock 返回成功
    let mock_success = mock_server
        .mock("GET", "/api/retry")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&TestData {
            id: "retry-123".to_string(),
            name: "Retry Success".to_string(),
            value: 100,
        }).unwrap())
        .create_async()
        .await;

    // 创建配置和 API 客户端，设置重试次数为 1
    let config = AppConfig {
        api_base_url: mock_url,
        request_timeout_ms: 30000,
        max_retries: 1,
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求
    let response: TestData = api_client.get("/api/retry", None).await.unwrap();

    // 验证响应
    assert_eq!(response.id, "retry-123");
    assert_eq!(response.name, "Retry Success");
    assert_eq!(response.value, 100);
    
    // 验证两个 mock 都被调用
    mock_error.assert_async().await;
    mock_success.assert_async().await;
}

// 测试 API 客户端请求超时
#[tokio::test]
async fn test_api_client_timeout() {
    // 创建配置和 API 客户端，设置非常短的超时时间
    let config = AppConfig {
        api_base_url: "http://localhost:9999",  // 不存在的服务器
        request_timeout_ms: 50,  // 50ms 超时
        max_retries: 0,  // 不重试
    };
    let api_client = ApiClient::new(&config, None);

    // 执行请求并验证超时错误
    let result: Result<TestData, ApiError> = api_client.get("/api/timeout", None).await;
    
    // 验证结果是错误
    assert!(result.is_err());
    match result.err().unwrap() {
        ApiError::RequestError(_) => (),  // 请求错误，表示超时
        _ => panic!("Expected RequestError but got another error type"),
    }
}