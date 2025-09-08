pub mod users;
pub mod pickers;
pub mod orders;

pub use users::*;
pub use pickers::*;
pub use orders::*;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::config::AppState;
use crate::download::download;

/// 创建所有路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 健康检查
        .route("/", get(|| async { "Pickers Server is running!" }))
        // 用户相关路由（公开）
        .route("/api/users/register", post(register))
        .route("/api/users/verify", post(verify))
        .route("/api/users/login", post(login))
        // Picker相关路由（公开）
        .route("/api/pickers", get(get_market))
        .route("/api/pickers/{picker_id}", get(get_picker_detail))
        // 下载路由
        .route("/download", get(download))
        // 添加CORS支持
        .layer(CorsLayer::permissive())
}

/// 创建需要认证的路由
pub fn create_protected_routes() -> Router<AppState> {
    Router::new()
        .route("/api/users/profile", get(get_profile))
        .route("/api/pickers", post(upload_picker))
        .route("/api/orders", post(create_order))
        .route("/api/orders/{order_id}", get(get_order_detail))
        .route("/api/orders", get(get_user_orders))
}

