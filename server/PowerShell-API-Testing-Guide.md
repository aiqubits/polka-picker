# Pickers Server PowerShell API 测试指南

本指南提供了使用 PowerShell `Invoke-RestMethod` 命令测试 Pickers Server 所有 API 接口的完整方案。

## 📁 测试文件说明

### 1. `test-api.ps1` - 完整交互式测试脚本
- **功能**: 提供完整的 API 测试套件，支持交互式菜单
- **特点**: 详细的错误处理、彩色输出、测试结果统计
- **适用**: 开发调试、完整功能验证

### 2. `quick-test.ps1` - 快速验证脚本  
- **功能**: 快速验证核心 API 功能
- **特点**: 自动化流程、简洁输出、快速反馈
- **适用**: CI/CD 集成、快速健康检查

## 🚀 使用方法

### 前置条件
1. 确保 Pickers Server 正在运行:
   ```powershell
   cd server
   cargo run
   ```

2. 确保 PowerShell 执行策略允许运行脚本:
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

### 快速测试
```powershell
# 运行快速测试（推荐首次使用）
.\quick-test.ps1

# 指定服务器地址
.\quick-test.ps1 -BaseUrl "http://localhost:3000"
```

### 完整交互式测试
```powershell
# 启动交互式测试菜单
.\test-api.ps1

# 直接运行所有测试
.\test-api.ps1 all

# 启用详细输出模式
.\test-api.ps1 -Verbose

# 指定服务器地址和详细模式
.\test-api.ps1 -BaseUrl "http://localhost:3000" -Verbose
```

## 📋 API 测试覆盖

### ✅ 用户管理 API
```powershell
# 1. 用户注册
Invoke-RestMethod -Uri "http://localhost:3000/api/users/register" -Method POST -ContentType "application/json" -Body '{
    "email": "test@example.com",
    "user_name": "Test User", 
    "user_type": "gen"
}'

# 2. 邮箱验证
Invoke-RestMethod -Uri "http://localhost:3000/api/users/verify" -Method POST -ContentType "application/json" -Body '{
    "email": "test@example.com",
    "code": "123456"
}'

# 3. 用户登录
Invoke-RestMethod -Uri "http://localhost:3000/api/users/login" -Method POST -ContentType "application/json" -Body '{
    "email": "test@example.com"
}'

# 4. 获取用户信息（需要 JWT Token）
$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN" }
Invoke-RestMethod -Uri "http://localhost:3000/api/users/profile" -Method GET -Headers $headers
```

### ✅ Picker 市场 API
```powershell
# 1. 获取市场列表
Invoke-RestMethod -Uri "http://localhost:3000/api/pickers" -Method GET

# 2. 分页查询
Invoke-RestMethod -Uri "http://localhost:3000/api/pickers?page=1&size=10" -Method GET

# 3. 关键词搜索
Invoke-RestMethod -Uri "http://localhost:3000/api/pickers?keyword=test" -Method GET

# 4. 获取 Picker 详情
Invoke-RestMethod -Uri "http://localhost:3000/api/pickers/550e8400-e29b-41d4-a716-446655440000" -Method GET

# 5. 上传 Picker（需要开发者权限和 multipart 支持）
# 注意: PowerShell 的 multipart 上传需要特殊处理
```

### ✅ 订单管理 API
```powershell
# 1. 创建订单（积分支付）
$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN" }
Invoke-RestMethod -Uri "http://localhost:3000/api/orders" -Method POST -ContentType "application/json" -Headers $headers -Body '{
    "picker_id": "550e8400-e29b-41d4-a716-446655440000",
    "pay_type": "premium"
}'

# 2. 创建订单（钱包支付）
Invoke-RestMethod -Uri "http://localhost:3000/api/orders" -Method POST -ContentType "application/json" -Headers $headers -Body '{
    "picker_id": "550e8400-e29b-41d4-a716-446655440000",
    "pay_type": "wallet",
    "tx_hash": "0x1234567890abcdef1234567890abcdef12345678"
}'

# 3. 获取订单详情
Invoke-RestMethod -Uri "http://localhost:3000/api/orders/ORDER_ID" -Method GET -Headers $headers

# 4. 获取订单列表
Invoke-RestMethod -Uri "http://localhost:3000/api/orders" -Method GET -Headers $headers

# 5. 按状态筛选订单
Invoke-RestMethod -Uri "http://localhost:3000/api/orders?status=success" -Method GET -Headers $headers
```

### ✅ 文件下载 API
```powershell
# 下载文件（需要有效的下载 token）
Invoke-RestMethod -Uri "http://localhost:3000/download?token=DOWNLOAD_TOKEN" -Method GET -OutFile "downloaded_file.zip"
```

## 🔧 高级用法

### 自定义测试函数
```powershell
function Test-CustomApi {
    param(
        [string]$Endpoint,
        [hashtable]$Body = @{},
        [string]$Token = ""
    )
    
    $headers = @{}
    if ($Token) {
        $headers["Authorization"] = "Bearer $Token"
    }
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3000$Endpoint" -Method POST -ContentType "application/json" -Headers $headers -Body ($Body | ConvertTo-Json)
        Write-Host "✅ 请求成功" -ForegroundColor Green
        return $response
    }
    catch {
        Write-Host "❌ 请求失败: $($_.Exception.Message)" -ForegroundColor Red
        return $null
    }
}

# 使用示例
$token = "your_jwt_token_here"
Test-CustomApi -Endpoint "/api/users/profile" -Token $token
```

### 批量测试脚本
```powershell
# 批量用户注册测试
1..5 | ForEach-Object {
    $userData = @{
        email = "user$_@example.com"
        user_name = "Test User $_"
        user_type = "gen"
    }
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/users/register" -Method POST -ContentType "application/json" -Body ($userData | ConvertTo-Json)
        Write-Host "✅ 用户 $_ 注册成功: $($response.user_id)" -ForegroundColor Green
    }
    catch {
        Write-Host "❌ 用户 $_ 注册失败: $($_.Exception.Message)" -ForegroundColor Red
    }
}
```

### 性能测试
```powershell
# 简单的性能测试
$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

1..10 | ForEach-Object {
    Invoke-RestMethod -Uri "http://localhost:3000/api/pickers" -Method GET | Out-Null
}

$stopwatch.Stop()
$avgTime = $stopwatch.ElapsedMilliseconds / 10
Write-Host "平均响应时间: $avgTime ms" -ForegroundColor Cyan
```

## 📊 测试结果解读

### 成功响应示例
```json
// 用户注册成功
{
    "user_id": "550e8400-e29b-41d4-a716-446655440000",
    "message": "Registration successful. Please check your email for verification code."
}

// 邮箱验证成功
{
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "user": {
        "user_id": "550e8400-e29b-41d4-a716-446655440000",
        "email": "test@example.com",
        "user_name": "Test User",
        "user_type": "gen",
        "wallet_address": "0x742d35Cc6634C0532925a3b8D4C0C8b3C2e1e1e1",
        "premium_amount": 1000,
        "created_at": "2024-01-01T00:00:00Z"
    }
}
```

### 错误响应示例
```json
// 邮箱格式错误
{
    "error": "Invalid email format"
}

// 未授权访问
{
    "error": "Unauthorized: Missing or invalid token"
}

// 资源不存在
{
    "error": "Picker not found"
}
```

## 🛠️ 故障排除

### 常见问题

1. **服务器连接失败**
   ```
   错误: 无法连接到服务器
   解决: 确保服务器正在运行 (cargo run)
   ```

2. **PowerShell 执行策略限制**
   ```powershell
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   ```

3. **JWT Token 过期**
   ```
   错误: Unauthorized
   解决: 重新进行邮箱验证获取新的 token
   ```

4. **验证码错误**
   ```
   错误: Invalid verification code
   解决: 查看服务器控制台输出的验证码
   ```

### 调试技巧

1. **启用详细输出**
   ```powershell
   .\test-api.ps1 -Verbose
   ```

2. **查看完整错误信息**
   ```powershell
   try {
       Invoke-RestMethod -Uri "http://localhost:3000/api/endpoint" -Method POST
   }
   catch {
       Write-Host $_.Exception.Response.StatusCode
       Write-Host $_.Exception.Response.StatusDescription
       Write-Host $_.ErrorDetails.Message
   }
   ```

3. **保存响应到文件**
   ```powershell
   $response = Invoke-RestMethod -Uri "http://localhost:3000/api/pickers" -Method GET
   $response | ConvertTo-Json -Depth 10 | Out-File "response.json"
   ```

## 📈 扩展和定制

### 添加新的测试用例
1. 在 `test-api.ps1` 中添加新的测试函数
2. 在菜单中添加对应选项
3. 在 `Start-ApiTests` 函数中调用新测试

### 集成到 CI/CD
```yaml
# GitHub Actions 示例
- name: Test API
  run: |
    Start-Process -FilePath "cargo" -ArgumentList "run" -WorkingDirectory "server" -NoNewWindow
    Start-Sleep -Seconds 10
    powershell -File "server/quick-test.ps1"
```

### 自定义配置
```powershell
# 创建配置文件 config.json
{
    "baseUrl": "http://localhost:3000",
    "testUsers": [
        {
            "email": "test1@example.com",
            "name": "Test User 1",
            "type": "gen"
        }
    ]
}

# 在脚本中读取配置
$config = Get-Content "config.json" | ConvertFrom-Json
```

## 📚 参考资料

- [PowerShell Invoke-RestMethod 文档](https://docs.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/invoke-restmethod)
- [Pickers Server API 规范](./spec.md)
- [Rust Axum 框架文档](https://docs.rs/axum/latest/axum/)

---

**提示**: 建议先运行 `quick-test.ps1` 进行快速验证，然后使用 `test-api.ps1` 进行详细测试和调试。