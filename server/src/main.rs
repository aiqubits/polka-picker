use pickers_server::{
    config::{AppState, Claims},
    database::{create_pool, init_database},
    handlers::{create_protected_routes, create_routes},
    utils::AppError,
};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tokio;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建数据库连接池
    let pool = create_pool().await.map_err(|e| {
        error!("Failed to create database pool: {}", e);
        AppError::InternalServerError
    })?;
    
    // 初始化数据库
    init_database(&pool).await.map_err(|e| {
        error!("Failed to initialize database: {}", e);
        AppError::InternalServerError
    })?;
    
    // 创建应用状态
    let app_state = AppState {
        db: pool,
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
        verification_codes: Arc::new(Mutex::new(HashMap::new())),
        download_tokens: Arc::new(Mutex::new(HashMap::new())),
    };
    
    // 创建定时任务来清理过期的验证码和下载令牌
    let cleanup_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await; // 每5分钟
            cleanup_state.cleanup_expired_codes();
            cleanup_state.cleanup_expired_tokens();
        }
    });
    
    // 创建路由
    let app = create_routes()
        .merge(create_protected_routes())
        .with_state(app_state);
    
    // 打印API文档
    info!("API Documentation:");
    info!("- GET  /                     - Health check");
    info!("- POST /api/users/register   - Register new user");
    info!("- POST /api/users/verify     - Verify user registration");
    info!("- POST /api/users/login      - User login");
    info!("- GET  /api/pickers          - Get list of pickers");
    info!("- GET  /api/pickers/:id      - Get picker details");
    info!("- GET  /api/users/profile    - Get user profile (protected)");
    info!("- POST /api/pickers          - Upload new picker (protected)");
    info!("- POST /api/orders           - Create new order (protected)");
    info!("- GET  /api/orders           - Get user orders (protected)");
    info!("- GET  /api/orders/:id       - Get order details (protected)");
    info!("- GET  /api/download/:token  - Download picker (protected)");
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn test_database_initialization() {
        // 创建数据库连接池
        let pool = create_pool().await.expect("Failed to create database pool");
        
        // 初始化数据库
        let result = init_database(&pool).await;
        assert!(result.is_ok(), "Database initialization should succeed");
        
        // 验证表是否创建
        let tables: Vec<String> = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
            .fetch_all(&pool)
            .await
            .expect("Failed to query tables")
            .into_iter()
            .map(|row| row.get(0))
            .collect();
        
        assert!(tables.contains(&"users".to_string()), "Users table should be created");
        assert!(tables.contains(&"pickers".to_string()), "Pickers table should be created");
        assert!(tables.contains(&"orders".to_string()), "Orders table should be created");
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        // 创建数据库连接池
        let pool = create_pool().await.expect("Failed to create database pool");
        
        // 创建应用状态
        let app_state = AppState {
            db: pool,
            jwt_secret: "test_secret".to_string(),
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
            download_tokens: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 验证应用状态字段
        assert_eq!(app_state.jwt_secret, "test_secret");
        assert!(app_state.verification_codes.lock().unwrap().is_empty());
        assert!(app_state.download_tokens.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_routes_creation() {
        // 创建数据库连接池
        let pool = create_pool().await.expect("Failed to create database pool");
        
        // 创建应用状态
        let app_state = AppState {
            db: pool,
            jwt_secret: "test_secret".to_string(),
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
            download_tokens: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 创建路由
        let app: axum::Router = create_routes()
            .merge(create_protected_routes())
            .with_state(app_state);
        
        // 这里我们只验证路由创建不 panic
        // 实际的路由测试在集成测试中完成
        assert!(true);
    }

    #[tokio::test]
    async fn test_cleanup_task_spawn() {
        // 创建数据库连接池
        let pool = create_pool().await.expect("Failed to create database pool");
        
        // 创建应用状态
        let app_state = AppState {
            db: pool,
            jwt_secret: "test_secret".to_string(),
            verification_codes: Arc::new(Mutex::new(HashMap::new())),
            download_tokens: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 创建定时任务来清理过期的验证码和下载令牌
        let cleanup_state = app_state.clone();
        let handle = tokio::spawn(async move {
            // 只运行一次清理而不是循环
            cleanup_state.cleanup_expired_codes();
            cleanup_state.cleanup_expired_tokens();
        });
        
        // 等待任务完成
        let result = handle.await;
        assert!(result.is_ok(), "Cleanup task should complete without error");
    }
}