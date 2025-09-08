use axum::{
    extract::{Query, State, Path, Multipart},
    response::Json,
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::config::AppState;
use crate::models::{Picker, UserType, User};
use crate::utils::AppError;

// 上传Picker请求
#[derive(Debug, Deserialize)]
pub struct UploadPickerRequest {
    pub alias: String,
    pub description: String,
    pub price: u64,
    pub version: String,
}

// 上传Picker响应
#[derive(Debug, Serialize)]
pub struct UploadPickerResponse {
    pub picker_id: Uuid,
    pub message: String,
}

// 市场查询参数
#[derive(Debug, Deserialize)]
pub struct MarketQuery {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub keyword: Option<String>,
}

// Picker信息
#[derive(Debug, Serialize)]
pub struct PickerInfo {
    pub picker_id: Uuid,
    pub alias: String,
    pub description: String,
    pub price: i64,
    pub image_path: String,
    pub version: String,
    pub download_count: i64,
    pub created_at: chrono::DateTime<Utc>,
}

// 市场响应
#[derive(Debug, Serialize)]
pub struct MarketResponse {
    pub pickers: Vec<PickerInfo>,
    pub total: u64,
}

// 上传Picker
pub async fn upload_picker(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadPickerResponse>, AppError> {
    // 验证用户是否为开发者
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    if user.user_type != UserType::Dev {
        return Err(AppError::BadRequest("Only developers can upload pickers".to_string()));
    }

    let mut alias = String::new();
    let mut description = String::new();
    let mut price = 0i64;
    let mut version = String::new();
    let mut image_path = String::new();
    let mut file_path = String::new();

    // 处理multipart数据
    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::BadRequest("Invalid multipart data".to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "alias" => {
                alias = field.text().await.map_err(|_| AppError::BadRequest("Invalid alias".to_string()))?;
            }
            "description" => {
                description = field.text().await.map_err(|_| AppError::BadRequest("Invalid description".to_string()))?;
            }
            "price" => {
                let price_str = field.text().await.map_err(|_| AppError::BadRequest("Invalid price".to_string()))?;
                price = price_str.parse().map_err(|_| AppError::BadRequest("Invalid price format".to_string()))?;
            }
            "version" => {
                version = field.text().await.map_err(|_| AppError::BadRequest("Invalid version".to_string()))?;
            }
            "image" => {
                let filename = field.file_name().unwrap_or("image.jpg").to_string();
                let data = field.bytes().await.map_err(|_| AppError::BadRequest("Invalid image data".to_string()))?;
                
                // 创建上传目录
                tokio::fs::create_dir_all("uploads/images").await.map_err(|_| AppError::InternalServerError)?;
                
                // 生成唯一文件名
                let unique_filename = format!("{}_{}", Uuid::new_v4(), filename);
                image_path = format!("uploads/images/{}", unique_filename);
                
                // 保存图片文件
                tokio::fs::write(&image_path, data).await.map_err(|_| AppError::InternalServerError)?;
            }
            "file" => {
                let filename = field.file_name().unwrap_or("picker.exe").to_string();
                let data = field.bytes().await.map_err(|_| AppError::BadRequest("Invalid file data".to_string()))?;
                
                // 创建上传目录
                tokio::fs::create_dir_all("uploads/files").await.map_err(|_| AppError::InternalServerError)?;
                
                // 生成唯一文件名
                let unique_filename = format!("{}_{}", Uuid::new_v4(), filename);
                file_path = format!("uploads/files/{}", unique_filename);
                
                // 保存文件
                tokio::fs::write(&file_path, data).await.map_err(|_| AppError::InternalServerError)?;
            }
            _ => {}
        }
    }

    // 验证必填字段
    if alias.is_empty() || description.is_empty() || version.is_empty() || image_path.is_empty() || file_path.is_empty() {
        return Err(AppError::BadRequest("Missing required fields".to_string()));
    }

    // 创建Picker记录
    let picker_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active', 0, ?, ?)
        "#,
    )
    .bind(picker_id)
    .bind(user_id)
    .bind(&alias)
    .bind(&description)
    .bind(price)
    .bind(&image_path)
    .bind(&file_path)
    .bind(&version)
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    Ok(Json(UploadPickerResponse {
        picker_id,
        message: "Picker uploaded successfully".to_string(),
    }))
}

// 获取市场列表
pub async fn get_market(
    State(state): State<AppState>,
    Query(query): Query<MarketQuery>,
) -> Result<Json<MarketResponse>, AppError> {
    let page = query.page.unwrap_or(1);
    let size = query.size.unwrap_or(10);
    let offset = (page - 1) * size;

    // 构建查询条件和获取数据
    let (pickers, total) = if let Some(keyword) = &query.keyword {
        let search_pattern = format!("%{}%", keyword);
        
        // 获取总数
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM pickers WHERE status = 'active' AND (alias LIKE ? OR description LIKE ?)"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        // 获取Picker列表
        let pickers: Vec<Picker> = sqlx::query_as(
            "SELECT * FROM pickers WHERE status = 'active' AND (alias LIKE ? OR description LIKE ?) ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(size as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        (pickers, total.0)
    } else {
        // 获取总数
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) as count FROM pickers WHERE status = 'active'"
        )
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        // 获取Picker列表
        let pickers: Vec<Picker> = sqlx::query_as(
            "SELECT * FROM pickers WHERE status = 'active' ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(size as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        (pickers, total.0)
    };

    let picker_infos: Vec<PickerInfo> = pickers.into_iter().map(|p| PickerInfo {
        picker_id: p.picker_id,
        alias: p.alias,
        description: p.description,
        price: p.price,
        image_path: p.image_path,
        version: p.version,
        download_count: p.download_count as i64,
        created_at: p.created_at,
    }).collect();

    Ok(Json(MarketResponse {
        pickers: picker_infos,
        total: total as u64,
    }))
}

// 获取Picker详情
pub async fn get_picker_detail(
    State(state): State<AppState>,
    Path(picker_id): Path<Uuid>,
) -> Result<Json<PickerInfo>, AppError> {
    let picker = sqlx::query_as::<_, Picker>(
        "SELECT * FROM pickers WHERE picker_id = ? AND status = 'active'",
    )
    .bind(picker_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?
    .ok_or_else(|| AppError::NotFound("Picker not found".to_string()))?;

    Ok(Json(PickerInfo {
        picker_id: picker.picker_id,
        alias: picker.alias,
        description: picker.description,
        price: picker.price,
        image_path: picker.image_path,
        version: picker.version,
        download_count: picker.download_count as i64,
        created_at: picker.created_at,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_app_state;
    use crate::models::{OrderStatus};
    use axum::extract::{Query, State, Path};
    use chrono::Utc;
    use serial_test::serial;
    use uuid::Uuid;

    #[tokio::test]
    #[serial]
    async fn test_get_market_success() {
        let state = create_test_app_state().await;
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_123', 'devwallet123', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建测试Picker
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let query = MarketQuery {
            page: Some(1),
            size: Some(10),
            keyword: None,
        };

        let result = get_market(State(state), Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.pickers.len(), 1);
        assert_eq!(response.total, 1);
        assert_eq!(response.pickers[0].alias, "Test Picker");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_market_with_keyword_search() {
        let state = create_test_app_state().await;
        let dev_user_id = Uuid::new_v4();
        let picker_id1 = Uuid::new_v4();
        let picker_id2 = Uuid::new_v4();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_123', 'devwallet123', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建两个测试Picker
        let now = Utc::now();
        // 创建第一个Picker
        let picker_id1 = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Game Picker', 'Game Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id1)
        .bind(dev_user_id)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();
        
        // 创建第二个Picker
        let picker_id2 = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Other Picker', 'Other Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id2)
        .bind(dev_user_id)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let query = MarketQuery {
            page: Some(1),
            size: Some(10),
            keyword: Some("game".to_string()),
        };

        let result = get_market(State(state), Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.pickers.len(), 1);
        assert_eq!(response.total, 1);
        assert_eq!(response.pickers[0].alias, "Game Picker");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_market_pagination() {
        let state = create_test_app_state().await;
        let dev_user_id = Uuid::new_v4();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_123', 'devwallet123', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建多个测试Picker
        let now = Utc::now();
        for i in 1..=15 {
            let picker_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
                VALUES (?, ?, ?, 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
                "#,
            )
            .bind(picker_id)
            .bind(dev_user_id)
            .bind(format!("Test Picker {}", i))
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&state.db)
            .await
            .unwrap();
        }

        let query = MarketQuery {
            page: Some(2),
            size: Some(10),
            keyword: None,
        };

        let result = get_market(State(state), Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.pickers.len(), 5); // 第二页应该有5个
        assert_eq!(response.total, 15);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_market_default_pagination() {
        let state = create_test_app_state().await;
        let dev_user_id = Uuid::new_v4();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_123', 'devwallet123', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建一个测试Picker
        let picker_id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let query = MarketQuery {
            page: None, // 使用默认值
            size: None, // 使用默认值
            keyword: None,
        };

        let result = get_market(State(state), Query(query)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.pickers.len(), 1);
        assert_eq!(response.total, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_picker_detail_success() {
        let state = create_test_app_state().await;
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_123', 'devwallet123', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建测试Picker
        sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 10, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let result = get_picker_detail(State(state), Path(picker_id)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.picker_id, picker_id);
        assert_eq!(response.alias, "Test Picker");
        assert_eq!(response.description, "Test Description");
        assert_eq!(response.price, 500);
        assert_eq!(response.download_count, 10);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_picker_detail_not_found() {
        let state = create_test_app_state().await;
        let picker_id = Uuid::new_v4();

        let result = get_picker_detail(State(state), Path(picker_id)).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::NotFound(msg) => assert_eq!(msg, "Picker not found"),
            _ => panic!("Expected NotFound error"),
        }
    }
}