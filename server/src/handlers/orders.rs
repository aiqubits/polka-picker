use axum::{
    extract::{Query, State, Path},
    response::Json,
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::config::AppState;
use crate::models::{Order, OrderStatus, PayType, User, UserType, Picker};
use crate::utils::AppError;

// 创建订单请求
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub picker_id: Uuid,
    pub pay_type: PayType,
}

// 创建订单响应
#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub order_id: Uuid,
    pub message: String,
}

// 订单查询参数
#[derive(Debug, Deserialize)]
pub struct OrderQuery {
    pub page: Option<u32>,
    pub size: Option<u32>,
    pub status: Option<OrderStatus>,
}

// 订单信息
#[derive(Debug, Serialize)]
pub struct OrderInfo {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub picker_id: Uuid,
    pub picker_alias: String,
    pub amount: i64,
    pub pay_type: PayType,
    pub status: OrderStatus,
    pub created_at: chrono::DateTime<Utc>,
}

// 订单列表响应
#[derive(Debug, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderInfo>,
    pub total: u64,
    pub page: u32,
    pub size: u32,
    pub has_next: bool,
}

// 创建订单
pub async fn create_order(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderResponse>, AppError> {
    info!("create_order called with user_id: {}, picker_id: {}, pay_type: {:?}", user_id, payload.picker_id, payload.pay_type);
    // 获取用户信息
    info!("Fetching user information...");
    let user_result = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await;
    
    match &user_result {
        Ok(user) => info!("User found: {:?}", user),
        Err(e) => info!("Failed to fetch user: {:?}", e),
    }
    
    let user = user_result.map_err(|_| AppError::NotFound("User not found".to_string()))?;

    // 获取Picker信息
    info!("Fetching picker information...");
    let picker_result = sqlx::query_as::<_, Picker>(
        "SELECT * FROM pickers WHERE picker_id = ? AND status = 'active'",
    )
    .bind(payload.picker_id)
    .fetch_optional(&state.db)
    .await;
    
    match &picker_result {
        Ok(Some(picker)) => info!("Picker found: {:?}", picker),
        Ok(None) => info!("Picker not found"),
        Err(e) => info!("Failed to fetch picker: {:?}", e),
    }
    
    let picker = picker_result
        .map_err(|_| AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Picker not found".to_string()))?;

    // 检查支付方式和余额
    match payload.pay_type {
        PayType::Premium => {
            if user.premium_balance < picker.price {
                return Err(AppError::BadRequest("Insufficient premium balance".to_string()));
            }
        }
        PayType::Wallet => {
            // 这里应该检查钱包余额，暂时跳过
        }
    }

    // 创建订单
    let order_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::hours(1); // 订单1小时后过期
    let tx_hash = if matches!(payload.pay_type, PayType::Wallet) {
        Some(Uuid::new_v4().to_string()) // 为钱包支付生成交易哈希
    } else {
        None
    };
    
    info!("Creating order with ID: {}", order_id);
    info!("User ID: {}", user_id);
    info!("Picker ID: {}", payload.picker_id);
    info!("Picker price: {}", picker.price);
    info!("Pay type: {:?}", payload.pay_type);
    info!("Order status: {:?}", OrderStatus::Pending);
    info!("TX hash: {:?}", tx_hash);
    info!("Created at: {}", now.to_rfc3339());
    info!("Expires at: {}", expires_at.to_rfc3339());

    // 开始事务
    info!("Starting transaction...");
    let mut tx = state.db.begin().await.map_err(|e| {
        info!("Failed to start transaction: {:?}", e);
        AppError::DatabaseError
    })?;

    // 插入订单记录
    if matches!(payload.pay_type, PayType::Wallet) {
        info!("Inserting wallet order...");
        let result = sqlx::query(
            r#"
            INSERT INTO orders (order_id, user_id, picker_id, amount, pay_type, status, tx_hash, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(payload.picker_id)
        .bind(picker.price)
        .bind(&payload.pay_type)
        .bind(&OrderStatus::Pending)
        .bind(&tx_hash)
        .bind(now.to_rfc3339())
        .bind(expires_at.to_rfc3339())
        .execute(&mut *tx)
        .await;
        
        match &result {
            Ok(_) => info!("Wallet order inserted successfully"),
            Err(e) => info!("Failed to insert wallet order: {:?}", e),
        }
        
        result.map_err(|_| AppError::DatabaseError)?;
    } else {
        info!("Inserting premium order...");
        let result = sqlx::query(
            r#"
            INSERT INTO orders (order_id, user_id, picker_id, amount, pay_type, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(payload.picker_id)
        .bind(picker.price)
        .bind(&payload.pay_type)
        .bind(&OrderStatus::Pending)
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await;
        
        match &result {
            Ok(_) => info!("Premium order inserted successfully"),
            Err(e) => info!("Failed to insert premium order: {:?}", e),
        }
        
        result.map_err(|_| AppError::DatabaseError)?;
    }

    // 扣除用户余额（如果是Premium支付）
    if matches!(payload.pay_type, PayType::Premium) {
        info!("Processing premium payment...");
        let result = sqlx::query(
            "UPDATE users SET premium_balance = premium_balance - ? WHERE user_id = ?",
        )
        .bind(picker.price)
        .bind(user_id)
        .execute(&mut *tx)
        .await;
        
        match &result {
            Ok(_) => info!("User balance updated successfully"),
            Err(e) => info!("Failed to update user balance: {:?}", e),
        }
        
        result.map_err(|_| AppError::DatabaseError)?;
        
        // 更新订单状态为成功
        info!("Updating order status to success...");
        let result = sqlx::query(
            "UPDATE orders SET status = ? WHERE order_id = ?",
        )
        .bind(&OrderStatus::Success)
        .bind(order_id)
        .execute(&mut *tx)
        .await;
        
        match &result {
            Ok(_) => info!("Order status updated successfully"),
            Err(e) => info!("Failed to update order status: {:?}", e),
        }
        
        result.map_err(|_| AppError::DatabaseError)?;

        // 增加Picker下载次数
        info!("Increasing picker download count...");
        let result = sqlx::query(
            "UPDATE pickers SET download_count = download_count + 1 WHERE picker_id = ?",
        )
        .bind(payload.picker_id)
        .execute(&mut *tx)
        .await;
        
        match &result {
            Ok(_) => info!("Picker download count updated successfully"),
            Err(e) => info!("Failed to update picker download count: {:?}", e),
        }
        
        result.map_err(|_| AppError::DatabaseError)?;
    }

    // 提交事务
    info!("Committing transaction...");
    let result = tx.commit().await;
    
    match &result {
        Ok(_) => info!("Transaction committed successfully"),
        Err(e) => info!("Failed to commit transaction: {:?}", e),
    }
    
    result.map_err(|_| AppError::DatabaseError)?;

    Ok(Json(CreateOrderResponse {
        order_id,
        message: "Order created successfully".to_string(),
    }))
}

// 获取用户订单列表
pub async fn get_user_orders(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Query(query): Query<OrderQuery>,
) -> Result<Json<OrderListResponse>, AppError> {
    let page = query.page.unwrap_or(1);
    let size = query.size.unwrap_or(10);
    let offset = (page - 1) * size;

    // 构建查询条件
    let (where_clause, count_where_clause) = if let Some(_status) = &query.status {
        (
            format!("WHERE o.user_id = ? AND o.status = ? ORDER BY o.created_at DESC LIMIT ? OFFSET ?"),
            "WHERE o.user_id = ? AND o.status = ?".to_string()
        )
    } else {
        (
            "WHERE o.user_id = ? ORDER BY o.created_at DESC LIMIT ? OFFSET ?".to_string(),
            "WHERE o.user_id = ?".to_string()
        )
    };

    // 获取总数
    let total = if let Some(status) = &query.status {
        let result: (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) as count FROM orders o {}", count_where_clause))
            .bind(user_id)
            .bind(status)
            .fetch_one(&state.db)
            .await
            .map_err(|_| AppError::DatabaseError)?;
        result.0
    } else {
        let result: (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) as count FROM orders o {}", count_where_clause))
            .bind(user_id)
            .fetch_one(&state.db)
            .await
            .map_err(|_| AppError::DatabaseError)?;
        result.0
    };

    // 获取订单列表
    let orders = if let Some(status) = &query.status {
        sqlx::query_as::<_, Order>(&format!(
            "SELECT o.* FROM orders o {}",
            where_clause
        ))
        .bind(user_id)
        .bind(status)
        .bind(size as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?
    } else {
        sqlx::query_as::<_, Order>(&format!(
            "SELECT o.* FROM orders o {}",
            where_clause
        ))
        .bind(user_id)
        .bind(size as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?
    };

    // 获取Picker信息
    let mut order_infos = Vec::new();
    for order in orders {
        let picker = sqlx::query_as::<_, Picker>(
            "SELECT * FROM pickers WHERE picker_id = ?",
        )
        .bind(order.picker_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        order_infos.push(OrderInfo {
            order_id: order.order_id,
            user_id: order.user_id,
            picker_id: order.picker_id,
            picker_alias: picker.alias,
            amount: order.amount,
            pay_type: order.pay_type,
            status: order.status,
            created_at: order.created_at,
        });
    }

    let has_next = (page * size) < total as u32;

    Ok(Json(OrderListResponse {
        orders: order_infos,
        total: total as u64,
        page,
        size,
        has_next,
    }))
}

// 获取订单详情
pub async fn get_order_detail(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderInfo>, AppError> {
    info!("get_order_detail called with order_id: {}, user_id: {}", order_id, user_id);
    
    // 获取订单信息
    let order_query = "SELECT * FROM orders WHERE order_id = ? AND user_id = ?";
    info!("Executing query: {}", order_query);
    let order_result = sqlx::query_as::<_, Order>(order_query)
        .bind(order_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await;
    
    match &order_result {
        Ok(Some(order)) => info!("Order found: {:?}", order),
        Ok(None) => info!("No order found"),
        Err(e) => info!("Error fetching order: {}", e),
    }
    
    let order = order_result
        .map_err(|_| AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

    // 获取Picker信息
    let picker_query = "SELECT * FROM pickers WHERE picker_id = ?";
    info!("Executing picker query: {}", picker_query);
    let picker_result = sqlx::query_as::<_, Picker>(picker_query)
        .bind(order.picker_id)
        .fetch_one(&state.db)
        .await;
    
    match &picker_result {
        Ok(picker) => info!("Picker found: {:?}", picker),
        Err(e) => info!("Error fetching picker: {}", e),
    }
    
    let picker = picker_result
        .map_err(|_| AppError::DatabaseError)?;

    let order_info = OrderInfo {
        order_id: order.order_id,
        user_id: order.user_id,
        picker_id: order.picker_id,
        picker_alias: picker.alias,
        amount: order.amount,
        pay_type: order.pay_type,
        status: order.status,
        created_at: order.created_at,
    };
    
    info!("Returning order info: {:?}", order_info);
    
    Ok(Json(order_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_app_state;
    use crate::models::{PayType, OrderStatus};
    use axum::extract::{State, Path};
    use axum::Extension;
    use chrono::Utc;
    use serial_test::serial;
    use uuid::Uuid;

    #[tokio::test]
    #[serial]
    async fn test_create_order_premium_success() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();

        // 创建测试用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 1000, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_456', 'devwallet456', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建测试Picker
        let result = sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();
        
        let request = CreateOrderRequest {
            picker_id,
            pay_type: PayType::Premium,
        };

        let result = create_order(
            State(state.clone()),
            Extension(user_id),
            Json(request),
        ).await;

        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.order_id.is_nil());
        assert_eq!(response.message, "Order created successfully");

        // 验证用户余额被扣除
        let user: User = sqlx::query_as("SELECT * FROM users WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(user.premium_balance, 500); // 1000 - 500

        // 验证Picker下载次数增加
        let picker: Picker = sqlx::query_as("SELECT * FROM pickers WHERE picker_id = ?")
            .bind(picker_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(picker.download_count, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_order_premium_insufficient_balance() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();

        // 创建测试用户（余额不足）
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
        VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 100, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        // 创建测试开发者用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_456', 'devwallet456', 0, ?)
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
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let request = CreateOrderRequest {
            picker_id,
            pay_type: PayType::Premium,
        };

        let result = create_order(
            State(state),
            Extension(user_id),
            Json(request),
        ).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::BadRequest(msg) => assert_eq!(msg, "Insufficient premium balance"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_create_order_picker_not_found() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4(); // 不存在的picker

        // 创建测试用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 1000, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let request = CreateOrderRequest {
            picker_id,
            pay_type: PayType::Premium,
        };

        let result = create_order(
            State(state),
            Extension(user_id),
            Json(request),
        ).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => assert_eq!(msg, "Picker not found"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_create_order_wallet_success() {
        info!("Starting test_create_order_wallet_success");
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();
        
        info!("User ID: {}", user_id);
        info!("Dev User ID: {}", dev_user_id);
        info!("Picker ID: {}", picker_id);

        // 创建测试用户
        info!("Creating test user...");
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 0, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        match &result {
            Ok(_) => info!("Test user created successfully"),
            Err(e) => info!("Failed to create test user: {:?}", e),
        }
        
        result.unwrap();

        // 创建测试开发者用户
        info!("Creating test dev user...");
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_456', 'devwallet456', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        match &result {
            Ok(_) => info!("Test dev user created successfully"),
            Err(e) => info!("Failed to create test dev user: {:?}", e),
        }
        
        result.unwrap();

        // 创建测试Picker
        info!("Creating test picker...");
        let now = Utc::now();
        let result = sqlx::query(
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
        .await;
        
        match &result {
            Ok(_) => info!("Test picker created successfully"),
            Err(e) => info!("Failed to create test picker: {:?}", e),
        }
        
        result.unwrap();

        let request = CreateOrderRequest {
            picker_id,
            pay_type: PayType::Wallet,
        };
        
        info!("Calling create_order...");

        let result = create_order(
            State(state.clone()),
            Extension(user_id),
            Json(request),
        ).await;
        
        info!("create_order result: {:?}", result);

        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.order_id.is_nil());
        assert_eq!(response.message, "Order created successfully");

        // 验证订单状态为Pending
        let order: Order = sqlx::query_as("SELECT * FROM orders WHERE order_id = ?")
            .bind(response.order_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        info!("Order status: {:?}", order.status);
        assert_eq!(order.status, OrderStatus::Pending);
        info!("Order tx_hash: {:?}", order.tx_hash);
        assert!(order.tx_hash.is_some());
        info!("Order expires_at: {:?}", order.expires_at);
        assert!(order.expires_at.is_some());

        // 验证Picker下载次数没有增加
        let picker: Picker = sqlx::query_as("SELECT * FROM pickers WHERE picker_id = ?")
            .bind(picker_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        info!("Picker download count: {}", picker.download_count);
        assert_eq!(picker.download_count, 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_order_detail_success() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        info!("Creating test users and picker...");

        // 创建测试用户
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 1000, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert user result: {:?}", result);
        result.unwrap();

        // 创建测试开发者用户
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, 'dev@test.com', 'Dev User', 'dev', 'private_key_456', 'devwallet456', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert dev user result: {:?}", result);
        result.unwrap();

        // 创建测试Picker
        let result = sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert picker result: {:?}", result);
        result.unwrap();

        info!("Creating test order...");

        // 创建测试订单
        let result = sqlx::query(
            r#"
            INSERT INTO orders (order_id, user_id, picker_id, amount, pay_type, status, tx_hash, created_at, expires_at)
            VALUES (?, ?, ?, 500, ?, ?, NULL, ?, NULL)
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(picker_id)
        .bind(&PayType::Premium)
        .bind(&OrderStatus::Success)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert order result: {:?}", result);
        result.unwrap();

        info!("Calling get_order_detail...");

        let result = get_order_detail(
            State(state),
            Extension(user_id),
            Path(order_id),
        ).await;

        info!("Get order detail result: {:?}", result);
    
        assert!(result.is_ok());
    
        let response = result.unwrap();
        assert_eq!(response.order_id, order_id);
        assert_eq!(response.user_id, user_id);
        assert_eq!(response.picker_id, picker_id);
        assert_eq!(response.picker_alias, "Test Picker");
        assert_eq!(response.amount, 500);
        assert_eq!(response.pay_type, PayType::Premium);
        assert_eq!(response.status, OrderStatus::Success, "Expected status to be success, but got {:?}", response.status);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_order_detail_not_found() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let order_id = Uuid::new_v4(); // 不存在的订单

        // 创建测试用户
        sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
        VALUES (?, 'user@test.com', 'Test User', 'gen', 'private_key_123', 'wallet123', 1000, ?)
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let result = get_order_detail(
            State(state),
            Extension(user_id),
            Path(order_id),
        ).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(msg) => assert_eq!(msg, "Order not found"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_order_detail_simple() {
        let state = create_test_app_state().await;
        let user_id = Uuid::new_v4();
        let dev_user_id = Uuid::new_v4();
        let picker_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
    
        // 创建测试用户
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, ?, 'Test User', 'gen', 'private_key_123', 'wallet123', 1000, ?)
            "#,
        )
        .bind(user_id)
        .bind("user@test.com")
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert user result: {:?}", result);
        result.unwrap();
    
        // 创建测试开发者用户
        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, email, user_name, user_type, private_key, wallet_address, premium_balance, created_at)
            VALUES (?, ?, 'Dev User', 'dev', 'private_key_456', 'devwallet456', 0, ?)
            "#,
        )
        .bind(dev_user_id)
        .bind("dev@test.com")
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert dev user result: {:?}", result);
        result.unwrap();
    
        // 创建测试Picker
        let result = sqlx::query(
            r#"
            INSERT INTO pickers (picker_id, dev_user_id, alias, description, price, image_path, file_path, version, status, download_count, created_at, updated_at)
            VALUES (?, ?, 'Test Picker', 'Test Description', 500, 'test.jpg', 'test.exe', '1.0', 'active', 0, ?, ?)
            "#,
        )
        .bind(picker_id)
        .bind(dev_user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert picker result: {:?}", result);
        result.unwrap();
    
        // 创建测试订单
        let result = sqlx::query(
            r#"
            INSERT INTO orders (order_id, user_id, picker_id, amount, pay_type, status, tx_hash, created_at, expires_at)
            VALUES (?, ?, ?, 500, ?, ?, NULL, ?, NULL)
            "#,
        )
        .bind(order_id)
        .bind(user_id)
        .bind(picker_id)
        .bind(&PayType::Premium)
        .bind(&OrderStatus::Success)
        .bind(Utc::now().to_rfc3339())
        .execute(&state.db)
        .await;
        
        info!("Insert order result: {:?}", result);
        result.unwrap();
    
        info!("Calling get_order_detail...");
    
        let result = get_order_detail(
            State(state),
            Extension(user_id),
            Path(order_id),
        ).await;
    
        info!("Get order detail result: {:?}", result);
    
        assert!(result.is_ok());
    
        let response = result.unwrap();
        assert_eq!(response.order_id, order_id);
        assert_eq!(response.user_id, user_id);
        assert_eq!(response.picker_id, picker_id);
        assert_eq!(response.picker_alias, "Test Picker");
        assert_eq!(response.amount, 500);
        assert_eq!(response.pay_type, PayType::Premium);
        assert_eq!(response.status, OrderStatus::Success);
    }
}