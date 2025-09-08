use axum::{
    extract::State,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::config::{AppState, Claims};
use crate::models::{User, UserType, VerificationCode};
use crate::utils::{generate_wallet, AppError};

// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub user_name: String,
    pub user_type: UserType,
}

// 注册响应
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub message: String,
}

// 验证请求
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub email: String,
    pub code: String,
}

// 验证响应
#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub token: String,
    pub user: UserInfo,
}

// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub email: String,
    pub user_name: String,
    pub user_type: UserType,
    pub wallet_address: String,
    pub premium_balance: i64,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id,
            email: user.email,
            user_name: user.user_name,
            user_type: user.user_type,
            wallet_address: user.wallet_address,
            premium_balance: user.premium_balance,
            created_at: user.created_at,
        }
    }
}

// 用户注册
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    // 验证邮箱格式
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Err(AppError::BadRequest("邮箱格式不正确".to_string()));
    }

    // 检查邮箱是否已存在
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    if existing_user.is_some() {
        return Err(AppError::UnprocessableEntity("邮箱已被注册".to_string()));
    }

    // 生成钱包地址和私钥
    let (private_key, wallet_address) = generate_wallet();
    
    // 创建用户
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
        VALUES (?, ?, ?, ?, ?, ?, 0, ?)
        "#,
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.user_name)
    .bind(match payload.user_type {
        UserType::Gen => "gen",
        UserType::Dev => "dev",
        UserType::Free => "free",
        UserType::Premium => "premium",
    })
    .bind(&private_key)
    .bind(&wallet_address)
    .bind(now.to_rfc3339())
    .execute(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    // 生成验证码
    let code = generate_verification_code();
    let verification_code = VerificationCode {
        code: code.clone(),
        expires_at: now + Duration::minutes(5),
        email: payload.email.clone(),
    };

    // 存储验证码
    state.verification_codes.lock().unwrap().insert(
        payload.email.clone(),
        verification_code,
    );

    // 模拟发送邮件（实际应用中应该调用邮件服务）
    info!("Verification code for {}: {}", payload.email, code);

    Ok(Json(RegisterResponse {
        user_id,
        message: "注册成功，验证码已发送到您的邮箱".to_string(),
    }))
}

// 验证邮箱
pub async fn verify(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    // 检查验证码
    let verification_code = {
        let codes = state.verification_codes.lock().unwrap();
        codes.get(&payload.email).cloned()
    };

    let verification_code = verification_code.ok_or_else(|| {
        AppError::BadRequest("验证码错误或已过期".to_string())
    })?;

    if verification_code.code != payload.code || verification_code.expires_at < Utc::now() {
        return Err(AppError::BadRequest("验证码错误或已过期".to_string()));
    }

    // 获取用户信息
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("用户不存在".to_string()))?;

    // 生成JWT token
    let claims = Claims::new(user.user_id);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::InternalServerError)?;

    // 移除验证码
    state.verification_codes.lock().unwrap().remove(&payload.email);

    Ok(Json(VerifyResponse {
        token,
        user: user.into(),
    }))
}

// 用户登录
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // 检查用户是否存在
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = ?",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    let user = user.ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    // 生成JWT token
    let claims = Claims::new(user.user_id);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::InternalServerError)?;

    Ok(Json(LoginResponse {
        token,
        user: user.into(),
    }))
}

// 获取用户信息
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

// 生成6位数字验证码
fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.random_range(100000..999999))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{create_pool, init_database};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    async fn create_test_state() -> AppState {
        let pool = create_pool().await.expect("Failed to create test pool");
        init_database(&pool).await.expect("Failed to init test database");
        
        AppState {
            db: pool,
            jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
            download_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[test]
    fn test_generate_verification_code() {
        let code = generate_verification_code();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        
        let code_num: u32 = code.parse().unwrap();
        assert!(code_num >= 100000 && code_num <= 999999);
    }

    #[test]
    fn test_generate_verification_code_uniqueness() {
        let mut codes = std::collections::HashSet::new();
        
        // 生成100个验证码，检查是否有重复
        for _ in 0..100 {
            let code = generate_verification_code();
            codes.insert(code);
        }
        
        // 虽然理论上可能有重复，但概率很低
        assert!(codes.len() > 90); // 允许少量重复
    }

    #[tokio::test]
    async fn test_register_valid_user() {
        let state = create_test_state().await;
        
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            user_name: "Test User".to_string(),
            user_type: UserType::Gen,
        };

        let result = register(State(state), Json(request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.user_id.to_string().is_empty());
        assert!(response.message.contains("验证码已发送"));
    }

    #[tokio::test]
    async fn test_register_invalid_email() {
        let state = create_test_state().await;
        
        let request = RegisterRequest {
            email: "invalid-email".to_string(),
            user_name: "Test User".to_string(),
            user_type: UserType::Gen,
        };

        let result = register(State(state), Json(request)).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::BadRequest(msg) => assert!(msg.contains("邮箱格式不正确")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let state = create_test_state().await;
        
        let request1 = RegisterRequest {
            email: "duplicate@example.com".to_string(),
            user_name: "Test User 1".to_string(),
            user_type: UserType::Gen,
        };

        let request2 = RegisterRequest {
            email: "duplicate@example.com".to_string(),
            user_name: "Test User 2".to_string(),
            user_type: UserType::Dev,
        };

        // 第一次注册应该成功
        let result1 = register(State(state.clone()), Json(request1)).await;
        assert!(result1.is_ok());

        // 第二次注册相同邮箱应该失败
        let result2 = register(State(state), Json(request2)).await;
        assert!(result2.is_err());
        
        match result2.unwrap_err() {
            AppError::UnprocessableEntity(msg) => assert!(msg.contains("邮箱已被注册")),
            _ => panic!("Expected UnprocessableEntity error"),
        }
    }

    #[tokio::test]
    async fn test_verify_valid_code() {
        let state = create_test_state().await;
        let email = "verify@example.com";
        
        // 先注册用户
        let register_request = RegisterRequest {
            email: email.to_string(),
            user_name: "Verify User".to_string(),
            user_type: UserType::Gen,
        };
        
        let register_result = register(State(state.clone()), Json(register_request)).await;
        if let Err(ref e) = register_result {
            info!("Register error: {:?}", e);
        }
        assert!(register_result.is_ok());

        // 检查用户是否真的被插入到数据库中
        let user_check = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&state.db)
            .await;
        info!("User check result: {:?}", user_check);

        // 获取生成的验证码
        let verification_codes = state.verification_codes.lock().unwrap();
        let verification_code = verification_codes.get(email).unwrap();
        let code = verification_code.code.clone();
        drop(verification_codes);

        // 验证邮箱
        let verify_request = VerifyRequest {
            email: email.to_string(),
            code,
        };

        let result = verify(State(state), Json(verify_request)).await;
        if let Err(ref e) = result {
            info!("Verify error: {:?}", e);
        }
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, email);
    }

    #[tokio::test]
    async fn test_verify_invalid_code() {
        let state = create_test_state().await;
        let email = "invalid@example.com";
        
        // 先注册用户
        let register_request = RegisterRequest {
            email: email.to_string(),
            user_name: "Invalid User".to_string(),
            user_type: UserType::Gen,
        };
        
        let register_result = register(State(state.clone()), Json(register_request)).await;
        assert!(register_result.is_ok());

        // 使用错误的验证码
        let verify_request = VerifyRequest {
            email: email.to_string(),
            code: "000000".to_string(),
        };

        let result = verify(State(state), Json(verify_request)).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::BadRequest(msg) => assert!(msg.contains("验证码错误或已过期")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[tokio::test]
    async fn test_verify_expired_code() {
        let state = create_test_state().await;
        let email = "expired@example.com";
        
        // 先注册用户
        let register_request = RegisterRequest {
            email: email.to_string(),
            user_name: "Expired User".to_string(),
            user_type: UserType::Gen,
        };
        
        let register_result = register(State(state.clone()), Json(register_request)).await;
        assert!(register_result.is_ok());

        // 手动设置过期的验证码
        {
            let mut verification_codes = state.verification_codes.lock().unwrap();
            let expired_code = VerificationCode {
                email: email.to_string(),
                code: "123456".to_string(),
                expires_at: Utc::now() - Duration::minutes(10), // 10分钟前过期
            };
            verification_codes.insert(email.to_string(), expired_code);
        }

        // 使用过期的验证码
        let verify_request = VerifyRequest {
            email: email.to_string(),
            code: "123456".to_string(),
        };

        let result = verify(State(state), Json(verify_request)).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::BadRequest(msg) => assert!(msg.contains("验证码错误或已过期")),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[tokio::test]
    async fn test_login_existing_user() {
        let state = create_test_state().await;
        let email = "login@example.com";
        
        // 先注册并验证用户
        let register_request = RegisterRequest {
            email: email.to_string(),
            user_name: "Login User".to_string(),
            user_type: UserType::Gen,
        };
        
        register(State(state.clone()), Json(register_request)).await.unwrap();
        
        // 获取验证码并验证
        let verification_codes = state.verification_codes.lock().unwrap();
        let code = verification_codes.get(email).unwrap().code.clone();
        drop(verification_codes);
        
        let verify_request = VerifyRequest {
            email: email.to_string(),
            code,
        };
        
        verify(State(state.clone()), Json(verify_request)).await.unwrap();

        // 现在测试登录
        let login_request = LoginRequest {
            email: email.to_string(),
        };

        let result = login(State(state), Json(login_request)).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, email);
    }

    #[tokio::test]
    async fn test_login_nonexistent_user() {
        let state = create_test_state().await;
        
        let login_request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
        };

        let result = login(State(state), Json(login_request)).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            AppError::NotFound(msg) => assert!(msg.contains("用户不存在")),
            _ => panic!("Expected NotFound error"),
        }
    }
}