# Pickers Server 测试指南

## 🧪 测试概览

本项目包含全面的测试套件，确保 Pickers Server 的质量和稳定性。

### 📋 测试类型

1. **单元测试** - 测试单个函数和模块
2. **集成测试** - 测试完整的 API 工作流程
3. **中间件测试** - 测试认证和错误处理
4. **数据库测试** - 测试数据库操作和约束

### 📊 测试覆盖范围

| 模块 | 测试类型 | 覆盖内容 |
|------|----------|----------|
| `models.rs` | 单元测试 | 数据模型序列化、枚举验证 |
| `utils.rs` | 单元测试 | 邮箱验证、钱包生成、错误处理 |
| `database.rs` | 单元测试 | 数据库连接、表创建、约束验证 |
| `middleware.rs` | 单元测试 | JWT 认证、错误处理 |
| `handlers/users.rs` | 单元测试 | 用户注册、验证、登录逻辑 |
| `integration_tests.rs` | 集成测试 | 完整 API 工作流程 |

## 🚀 运行测试

### 快速开始

```powershell
# 运行所有测试
.\run-tests.ps1

# 仅运行单元测试
.\run-tests.ps1 -UnitOnly

# 仅运行集成测试
.\run-tests.ps1 -IntegrationOnly

# 生成覆盖率报告
.\run-tests.ps1 -Coverage
```

### 使用 Cargo 直接运行

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test models::tests

# 运行特定测试
cargo test test_user_registration

# 显示详细输出
cargo test -- --nocapture

# 运行集成测试
cargo test --test integration_tests
```

## 📝 测试详情

### 1. 数据模型测试 (`models.rs`)

```rust
#[test]
fn test_user_type_serialization() // 测试用户类型序列化
fn test_order_status_enum()       // 测试订单状态枚举
fn test_picker_model_validation() // 测试 Picker 模型验证
```

### 2. 工具函数测试 (`utils.rs`)

```rust
#[test]
fn test_email_validation()        // 测试邮箱格式验证
fn test_wallet_generation()       // 测试钱包地址生成
fn test_error_response_format()   // 测试错误响应格式
```

### 3. 数据库测试 (`database.rs`)

```rust
#[tokio::test]
async fn test_create_pool()           // 测试数据库连接池创建
async fn test_init_database()         // 测试数据库初始化
async fn test_database_tables_created() // 测试表创建
async fn test_user_table_constraints() // 测试用户表约束
```

### 4. 中间件测试 (`middleware.rs`)

```rust
#[tokio::test]
async fn test_auth_middleware_with_valid_token()   // 测试有效 Token
async fn test_auth_middleware_without_token()      // 测试缺少 Token
async fn test_auth_middleware_with_invalid_token() // 测试无效 Token
async fn test_auth_middleware_with_expired_token() // 测试过期 Token
```

### 5. 用户处理器测试 (`handlers/users.rs`)

```rust
#[tokio::test]
async fn test_register_valid_user()      // 测试用户注册
async fn test_register_invalid_email()   // 测试无效邮箱
async fn test_register_duplicate_email() // 测试重复邮箱
async fn test_verify_valid_code()        // 测试邮箱验证
async fn test_login_existing_user()      // 测试用户登录
```

### 6. 集成测试 (`integration_tests.rs`)

```rust
#[tokio::test]
async fn test_user_registration_flow()    // 测试完整注册流程
async fn test_unauthorized_access()       // 测试未授权访问
async fn test_cors_headers()              // 测试 CORS 头
async fn test_malformed_json_request()    // 测试格式错误的请求
```

## 📊 测试结果解读

### 成功输出示例

```
running 25 tests
test models::tests::test_user_type_serialization ... ok
test utils::tests::test_email_validation ... ok
test database::tests::test_create_pool ... ok
test handlers::users::tests::test_register_valid_user ... ok
test integration_tests::test_user_registration_flow ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 失败输出示例

```
test handlers::users::tests::test_register_invalid_email ... FAILED

failures:

---- handlers::users::tests::test_register_invalid_email stdout ----
thread 'handlers::users::tests::test_register_invalid_email' panicked at 'assertion failed: result.is_err()'
```

## 🔧 测试配置

### 环境变量

- `RUST_TEST_THREADS=1` - 避免数据库并发问题
- `RUST_BACKTRACE=1` - 显示详细错误信息

### 测试数据库

- 使用内存 SQLite 数据库 (`sqlite::memory:`)
- 每个测试都有独立的数据库实例
- 测试结束后自动清理

### 测试工具

- `tokio-test` - 异步测试支持
- `axum-test` - HTTP 处理器测试
- `tower` - 中间件测试
- `serde_json` - JSON 序列化测试

## 🎯 测试最佳实践

### 1. 测试命名

```rust
// ✅ 好的命名
#[test]
fn test_email_validation_with_valid_email()

// ❌ 不好的命名
#[test]
fn test1()
```

### 2. 测试结构

```rust
#[tokio::test]
async fn test_function_name() {
    // Arrange - 准备测试数据
    let state = create_test_state().await;
    let request = TestRequest { ... };
    
    // Act - 执行被测试的功能
    let result = function_under_test(state, request).await;
    
    // Assert - 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap().field, expected_value);
}
```

### 3. 错误测试

```rust
#[tokio::test]
async fn test_error_case() {
    let result = function_that_should_fail().await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AppError::BadRequest(msg) => assert!(msg.contains("expected error")),
        _ => panic!("Expected BadRequest error"),
    }
}
```

## 🐛 故障排除

### 常见问题

1. **数据库连接失败**
   ```
   Error: Failed to create test database pool
   ```
   解决方案: 确保 SQLite 支持已启用

2. **测试超时**
   ```
   Error: test timed out
   ```
   解决方案: 增加 `RUST_TEST_THREADS=1` 环境变量

3. **JWT 测试失败**
   ```
   Error: Invalid JWT token
   ```
   解决方案: 检查测试中的密钥是否一致

### 调试技巧

1. **使用 `println!` 调试**
   ```rust
   println!("Debug: {:?}", variable);
   ```

2. **运行单个测试**
   ```bash
   cargo test test_specific_function -- --nocapture
   ```

3. **查看详细错误**
   ```bash
   RUST_BACKTRACE=full cargo test
   ```

## 📈 持续集成

### GitHub Actions 配置示例

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
```

## 🎉 总结

这个测试套件提供了：

- ✅ **全面覆盖** - 覆盖所有核心功能
- ✅ **快速反馈** - 快速发现问题
- ✅ **易于维护** - 清晰的测试结构
- ✅ **自动化** - 支持 CI/CD 集成
- ✅ **文档化** - 测试即文档

通过运行这些测试，你可以确信 Pickers Server 的功能正常工作，并且在未来的修改中不会破坏现有功能。