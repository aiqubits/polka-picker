use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;
use crate::database::DbPool;
use crate::models::{VerificationCode, DownloadToken};

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub verification_codes: Arc<Mutex<HashMap<String, VerificationCode>>>,
    pub download_tokens: Arc<Mutex<HashMap<String, DownloadToken>>>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self {
            db,
            jwt_secret: "your-secret-key".to_string(), // 在生产环境中应该从环境变量读取
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
            download_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // 清理过期的验证码
    pub fn cleanup_expired_codes(&self) {
        let mut codes = self.verification_codes.lock().unwrap();
        let now = Utc::now();
        codes.retain(|_, code| code.expires_at > now);
    }

    // 清理过期的下载token
    pub fn cleanup_expired_tokens(&self) {
        let mut tokens = self.download_tokens.lock().unwrap();
        let now = Utc::now();
        tokens.retain(|_, token| token.expires_at > now);
    }
}

// JWT Claims
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

impl Claims {
    pub fn new(user_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(24); // 24小时过期
        
        Self {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_app_state;
    use crate::models::{VerificationCode, DownloadToken};
    use chrono::{Duration, Utc};
    use serial_test::serial;
    use uuid::Uuid;

    #[tokio::test]
    #[serial]
    async fn test_app_state_new() {
        let state = create_test_app_state().await;
        
        assert_eq!(state.jwt_secret, "test_secret_key_for_testing_purposes_only_do_not_use_in_production");
        assert_eq!(state.verification_codes.lock().unwrap().len(), 0);
        assert_eq!(state.download_tokens.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_app_state_clone() {
        let state = create_test_app_state().await;
        let cloned_state = state.clone();
        
        assert_eq!(state.jwt_secret, cloned_state.jwt_secret);
        
        // 验证共享状态
        state.verification_codes.lock().unwrap().insert(
            "test_code".to_string(),
            VerificationCode {
                code: "123456".to_string(),
                expires_at: Utc::now() + Duration::minutes(10),
                email: "test@example.com".to_string(),
            }
        );
        
        assert_eq!(cloned_state.verification_codes.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_codes() {
        let state = create_test_app_state().await;
        let now = Utc::now();
        
        // 添加有效验证码
        let valid_code = VerificationCode {
            code: "123456".to_string(),
            expires_at: now + Duration::minutes(10),
            email: "valid@example.com".to_string(),
        };
        
        // 添加过期验证码
        let expired_code = VerificationCode {
            code: "654321".to_string(),
            expires_at: now - Duration::minutes(10),
            email: "expired@example.com".to_string(),
        };
        
        state.verification_codes.lock().unwrap().insert("valid".to_string(), valid_code);
        state.verification_codes.lock().unwrap().insert("expired".to_string(), expired_code);
        
        assert_eq!(state.verification_codes.lock().unwrap().len(), 2);
        
        // 清理过期验证码
        state.cleanup_expired_codes();
        
        assert_eq!(state.verification_codes.lock().unwrap().len(), 1);
        assert!(state.verification_codes.lock().unwrap().contains_key("valid"));
        assert!(!state.verification_codes.lock().unwrap().contains_key("expired"));
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_tokens() {
        let state = create_test_app_state().await;
        let now = Utc::now();
        let order_id1 = Uuid::new_v4();
        let order_id2 = Uuid::new_v4();
        
        // 添加有效下载token
        let valid_token = DownloadToken {
            token: "valid_token".to_string(),
            order_id: order_id1,
            expires_at: now + Duration::minutes(10),
        };
        
        // 添加过期下载token
        let expired_token = DownloadToken {
            token: "expired_token".to_string(),
            order_id: order_id2,
            expires_at: now - Duration::minutes(10),
        };
        
        state.download_tokens.lock().unwrap().insert("valid_token".to_string(), valid_token);
        state.download_tokens.lock().unwrap().insert("expired_token".to_string(), expired_token);
        
        assert_eq!(state.download_tokens.lock().unwrap().len(), 2);
        
        // 清理过期token
        state.cleanup_expired_tokens();
        
        assert_eq!(state.download_tokens.lock().unwrap().len(), 1);
        assert!(state.download_tokens.lock().unwrap().contains_key("valid_token"));
        assert!(!state.download_tokens.lock().unwrap().contains_key("expired_token"));
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_codes_empty() {
        let state = create_test_app_state().await;
        
        // 测试空的验证码集合
        state.cleanup_expired_codes();
        assert_eq!(state.verification_codes.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_tokens_empty() {
        let state = create_test_app_state().await;
        
        // 测试空的token集合
        state.cleanup_expired_tokens();
        assert_eq!(state.download_tokens.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_codes_all_valid() {
        let state = create_test_app_state().await;
        let now = Utc::now();
        
        // 添加多个有效验证码
        for i in 1..=3 {
            let code = VerificationCode {
                code: format!("12345{}", i),
                expires_at: now + Duration::minutes(10),
                email: format!("user{}@example.com", i),
            };
            state.verification_codes.lock().unwrap().insert(format!("code{}", i), code);
        }
        
        assert_eq!(state.verification_codes.lock().unwrap().len(), 3);
        
        // 清理过期验证码（应该没有过期的）
        state.cleanup_expired_codes();
        
        assert_eq!(state.verification_codes.lock().unwrap().len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_cleanup_expired_tokens_all_valid() {
        let state = create_test_app_state().await;
        let now = Utc::now();
        
        // 添加多个有效token
        for i in 1..=3 {
            let token = DownloadToken {
                token: format!("token{}", i),
                order_id: Uuid::new_v4(),
                expires_at: now + Duration::minutes(10),
            };
            state.download_tokens.lock().unwrap().insert(format!("token{}", i), token);
        }
        
        assert_eq!(state.download_tokens.lock().unwrap().len(), 3);
        
        // 清理过期token（应该没有过期的）
        state.cleanup_expired_tokens();
        
        assert_eq!(state.download_tokens.lock().unwrap().len(), 3);
    }

    #[test]
    fn test_claims_new() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(user_id);
        
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > claims.iat);
        
        // 验证过期时间大约是24小时后
        let expected_exp = chrono::Utc::now().timestamp() as usize + 24 * 3600;
        assert!((claims.exp as i64 - expected_exp as i64).abs() < 60); // 允许1分钟误差
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(user_id);
        
        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("sub"));
        assert!(json.contains("exp"));
        assert!(json.contains("iat"));
        assert!(json.contains(&user_id.to_string()));
    }

    #[test]
    fn test_claims_deserialization() {
        let user_id = Uuid::new_v4();
        let original_claims = Claims::new(user_id);
        
        let json = serde_json::to_string(&original_claims).unwrap();
        let deserialized_claims: Claims = serde_json::from_str(&json).unwrap();
        
        assert_eq!(original_claims.sub, deserialized_claims.sub);
        assert_eq!(original_claims.exp, deserialized_claims.exp);
        assert_eq!(original_claims.iat, deserialized_claims.iat);
    }

    #[test]
    fn test_claims_debug() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(user_id);
        
        let debug_str = format!("{:?}", claims);
        assert!(debug_str.contains("Claims"));
        assert!(debug_str.contains("sub"));
        assert!(debug_str.contains("exp"));
        assert!(debug_str.contains("iat"));
    }

    #[tokio::test]
    #[serial]
    async fn test_concurrent_access_verification_codes() {
        let state = create_test_app_state().await;
        let state_clone1 = state.clone();
        let state_clone2 = state.clone();
        
        // 模拟并发访问
        let handle1 = tokio::spawn(async move {
            for i in 0..10 {
                let code = VerificationCode {
                    code: format!("code{}", i),
                    expires_at: Utc::now() + Duration::minutes(10),
                    email: format!("user{}@example.com", i),
                };
                state_clone1.verification_codes.lock().unwrap().insert(format!("key{}", i), code);
            }
        });
        
        let handle2 = tokio::spawn(async move {
            for i in 10..20 {
                let code = VerificationCode {
                    email: format!("user{}@example.com", i),
                    code: format!("code{}", i),
                    expires_at: Utc::now() + Duration::minutes(10),
                };
                state_clone2.verification_codes.lock().unwrap().insert(format!("key{}", i), code);
            }
        });
        
        handle1.await.unwrap();
        handle2.await.unwrap();
        
        assert_eq!(state.verification_codes.lock().unwrap().len(), 20);
    }

    #[tokio::test]
    #[serial]
    async fn test_concurrent_access_download_tokens() {
        let state = create_test_app_state().await;
        let state_clone1 = state.clone();
        let state_clone2 = state.clone();
        
        // 模拟并发访问
        let handle1 = tokio::spawn(async move {
            for i in 0..10 {
                let token = DownloadToken {
                    token: format!("token{}", i),
                    order_id: Uuid::new_v4(),
                    expires_at: Utc::now() + Duration::minutes(10),
                };
                state_clone1.download_tokens.lock().unwrap().insert(format!("token{}", i), token);
            }
        });
        
        let handle2 = tokio::spawn(async move {
            for i in 10..20 {
                let token = DownloadToken {
                    token: format!("token{}", i),
                    order_id: Uuid::new_v4(),
                    expires_at: Utc::now() + Duration::minutes(10),
                };
                state_clone2.download_tokens.lock().unwrap().insert(format!("token{}", i), token);
            }
        });
        
        handle1.await.unwrap();
        handle2.await.unwrap();
        
        assert_eq!(state.download_tokens.lock().unwrap().len(), 20);
    }
}