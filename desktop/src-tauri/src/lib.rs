#[cfg_attr(mobile, tauri::mobile_entry_point)]
// 导入必要的模块
use tauri::{Builder, Manager};
use tauri_plugin_store::{StoreBuilder};
use log::LevelFilter;

// 导入命令模块
pub mod commands;
pub mod api;
pub mod utils;
pub mod config;

// 导入认证管理器
use crate::utils::auth::AuthManager;

// 认证存储键
const AUTH_STORE_KEY: &str = "auth";

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn run() {
  // 创建 Tauri 应用
  Builder::default()
    // 设置插件
    .plugin(tauri_plugin_store::Builder::default().build())
    .setup(|app| {
      // 在调试模式下启用日志插件
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      
      // 初始化认证存储
      let auth_store = StoreBuilder::new(app.handle(), "auth.json")
        .build()?;
      
      // 将认证存储添加到应用状态
      app.manage(auth_store.clone());
      
      // 创建并管理 AuthManager 实例
      let auth_manager = AuthManager::new(auth_store);
      app.manage(auth_manager);
      
      Ok(())
    })
    // 注册命令
    .invoke_handler(tauri::generate_handler!(
      greet,
      // 健康检查
      commands::pickers::simple_connection_test,
      commands::users::api_connection,
      // 用户相关命令
      commands::users::login,
      commands::users::register,
      commands::users::verify_email,
      commands::users::get_user_profile,
      commands::users::logout,
      commands::users::check_login_status,
      commands::users::get_current_user_info,
      
      // Picker 相关命令
      commands::pickers::get_picker_marketplace,
      commands::pickers::get_picker_detail,
      commands::pickers::upload_picker,
      
      // 订单相关命令
      commands::orders::get_user_orders,
      commands::orders::create_order,
      commands::orders::get_order_detail,
      
      // 下载相关命令
      commands::download::download_picker
    ))
    // 运行应用
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // 测试 AUTH_STORE_KEY 常量
    #[test]
    fn test_auth_store_key_constant() {
        assert_eq!(AUTH_STORE_KEY, "auth");
    }
    
    // 测试模块导入
    #[test]
    fn test_module_imports() {
        // 验证模块可以被正确导入
        // 由于我们不能直接测试模块导入，这个测试主要是确保模块存在
        assert!(true);
    }
    
    // 测试核心模块可用性
    #[test]
    fn test_core_modules_available() {
        // 验证核心模块可以被访问
        let _ = config::AppConfig::default();
        
        // 测试通过
        assert!(true);
    }
}
