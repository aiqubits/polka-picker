use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    Router,
};
use pickers_server::{
    config::AppState,
    database::{create_pool, init_database},
    handlers::create_routes,
    utils::AppError,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_app() -> Router {
    let pool = create_pool().await.expect("Failed to create test database pool");
    init_database(&pool).await.expect("Failed to initialize test database");

    let state = AppState {
        db: pool,
        jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
        verification_codes: Arc::new(Mutex::new(HashMap::new())),
        download_tokens: Arc::new(Mutex::new(HashMap::new())),
    };

    create_routes().with_state(state)
}

#[tokio::test]
async fn test_user_registration_flow() {
    let app = create_test_app().await;
    
    let timestamp = chrono::Utc::now().timestamp();
    let test_email = format!("test{}@example.com", timestamp);
    
    // 测试用户注册
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": test_email,
                "user_name": "Test User",
                "user_type": "Gen"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_user_registration_duplicate_email() {
    let app = create_test_app().await;
    
    let test_email = "duplicate@example.com";
    
    // 第一次注册
    let register_request1 = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": test_email,
                "user_name": "Test User 1",
                "user_type": "Gen"
            })
            .to_string(),
        ))
        .unwrap();

    let response1 = app.clone().oneshot(register_request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::OK);

    // 第二次注册相同邮箱
    let register_request2 = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": test_email,
                "user_name": "Test User 2",
                "user_type": "Gen"
            })
            .to_string(),
        ))
        .unwrap();

    let response2 = app.oneshot(register_request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_user_registration_invalid_user_type() {
    let app = create_test_app().await;
    
    let timestamp = chrono::Utc::now().timestamp();
    let test_email = format!("invalid{}@example.com", timestamp);
    
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": test_email,
                "user_name": "Test User",
                "user_type": "InvalidType"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(register_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_user_registration_invalid_email() {
    let app = create_test_app().await;
    
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "invalid-email",
                "user_name": "Test User",
                "user_type": "Gen"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(register_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_pickers_list() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/pickers")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_pickers_with_pagination() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/pickers?page=1&limit=10")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_pickers_with_search() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/pickers?search=test")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_unauthorized_access() {
    let app = create_test_app().await;
    
    // 测试不存在的端点
    let request = Request::builder()
        .method("GET")
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_endpoint() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/pickers")
        .header("Origin", "http://localhost:3000")
        .header("Access-Control-Request-Method", "GET")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
    assert!(headers.contains_key("access-control-allow-methods"));
}

#[tokio::test]
async fn test_malformed_json_request() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_required_fields() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/users/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "test@example.com"
                // 缺少 user_name �?user_type
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;
    
    let request = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
