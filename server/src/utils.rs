use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rand::RngCore;
use serde_json::json;

// 自定义错误类型
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    NotFound(String),
    UnprocessableEntity(String),
    InternalServerError,
    DatabaseError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

// 生成EVM钱包地址（简化版本）
pub fn generate_wallet() -> (String, String) {
    let mut rng = rand::thread_rng();
    
    // 生成32字节私钥
    let mut private_key: [u8; 32] = [0; 32];
    rng.fill_bytes(&mut private_key);
    let private_key_hex = hex::encode(private_key);
    
    // 生成20字节地址（简化版本，实际应该从私钥推导）
    let mut address: [u8; 20] = [0; 20];
    rng.fill_bytes(&mut address);
    let wallet_address = format!("0x{}", hex::encode(address));
    
    (private_key_hex, wallet_address)
}

// 生成随机token
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let mut token: [u8; 20] = [0; 20];
    rng.fill_bytes(&mut token);
    hex::encode(token)
}

#[cfg(test)]
pub mod test_utils {
    use crate::config::AppState;
    use crate::database::create_pool;
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    use serde_json::Value;

    pub async fn create_test_app_state() -> AppState {
        let pool = create_pool().await.expect("Failed to create test database pool");
        crate::database::init_database(&pool).await.expect("Failed to initialize test database");
        AppState::new(pool)
    }

    pub async fn send_request(
        app: axum::Router,
        request: Request<Body>,
    ) -> (StatusCode, Value) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let (_parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let body_json: Value = serde_json::from_str(&body_str).unwrap_or(serde_json::json!({}));
        (status, body_json)
    }
}

// 验证邮箱格式
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email) && !email.contains("..")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_wallet() {
        let (private_key, wallet_address) = generate_wallet();
        
        // 私钥应该是64个字符的十六进制字符串
        assert_eq!(private_key.len(), 64);
        assert!(private_key.chars().all(|c| c.is_ascii_hexdigit()));
        
        // 钱包地址应该以0x开头，后跟40个十六进制字符
        assert!(wallet_address.starts_with("0x"));
        assert_eq!(wallet_address.len(), 42);
        assert!(wallet_address[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_wallet_uniqueness() {
        let (private_key1, wallet_address1) = generate_wallet();
        let (private_key2, wallet_address2) = generate_wallet();
        
        // 每次生成的钱包应该不同
        assert_ne!(private_key1, private_key2);
        assert_ne!(wallet_address1, wallet_address2);
    }

    #[test]
    fn test_generate_token() {
        let token = generate_token();
        
        // Token应该是40个字符的十六进制字符串
        assert_eq!(token.len(), 40);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let token1 = generate_token();
        let token2 = generate_token();
        
        // 每次生成的token应该不同
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_is_valid_email() {
        // 有效邮箱
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(is_valid_email("test123@test-domain.org"));
        
        // 无效邮箱
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
        assert!(!is_valid_email("test@.com"));
        assert!(!is_valid_email("test..test@example.com"));
    }

    #[tokio::test]
    async fn test_create_test_app_state() {
        let state = test_utils::create_test_app_state().await;
        // 验证状态创建成功
        assert!(!state.db.is_closed());
    }

    #[tokio::test]
    async fn test_send_request() {
        use axum::{routing::get, Router};
        use axum::http::Request;
        
        let app = Router::new().route("/test", get(|| async { "Hello, World!" }));
        
        let request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
            
        let (status, _body) = test_utils::send_request(app, request).await;
        assert_eq!(status, StatusCode::OK);
    }
}