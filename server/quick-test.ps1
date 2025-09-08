# Pickers Server 快速 API 测试脚本
# 简化版本，用于快速验证核心功能

param(
    [string]$BaseUrl = "http://localhost:3000"
)

# 测试结果统计
$Global:TestResults = @{
    Passed = 0
    Failed = 0
    Total = 0
}

function Test-Api {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Endpoint,
        [hashtable]$Body = @{},
        [hashtable]$Headers = @{},
        [scriptblock]$Validator = { $true }
    )
    
    $Global:TestResults.Total++
    
    try {
        $params = @{
            Uri = "$BaseUrl$Endpoint"
            Method = $Method
            ContentType = "application/json"
            Headers = $Headers
        }
        
        if ($Body.Count -gt 0) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
        }
        
        $response = Invoke-RestMethod @params
        
        if (& $Validator $response) {
            Write-Host "✅ $Name" -ForegroundColor Green
            $Global:TestResults.Passed++
            return $response
        } else {
            Write-Host "❌ $Name - 验证失败" -ForegroundColor Red
            $Global:TestResults.Failed++
            return $null
        }
    }
    catch {
        Write-Host "❌ $Name - $($_.Exception.Message)" -ForegroundColor Red
        $Global:TestResults.Failed++
        return $null
    }
}

# 快速测试流程
Write-Host "🚀 Pickers Server 快速测试开始" -ForegroundColor Cyan
Write-Host "服务器: $BaseUrl" -ForegroundColor Gray

# 1. 测试服务器连接
$marketResponse = Test-Api -Name "服务器连接" -Method "GET" -Endpoint "/api/pickers" -Validator {
    param($response)
    return $response -and $response.PSObject.Properties.Name -contains "pickers"
}

if (-not $marketResponse) {
    Write-Host "❌ 服务器连接失败，停止测试" -ForegroundColor Red
    exit 1
}

# 2. 用户注册
$registerResponse = Test-Api -Name "用户注册" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = "quicktest@example.com"
    user_name = "Quick Test User"
    user_type = "gen"
} -Validator {
    param($response)
    return $response -and $response.user_id
}

$userId = $registerResponse.user_id

# 3. 邮箱验证（使用默认验证码）
Write-Host "ℹ️  使用默认验证码 123456 进行测试" -ForegroundColor Yellow

$verifyResponse = Test-Api -Name "邮箱验证" -Method "POST" -Endpoint "/api/users/verify" -Body @{
    email = "quicktest@example.com"
    code = "123456"
} -Validator {
    param($response)
    return $response -and $response.token
}

$jwtToken = $verifyResponse.token

# 4. 获取用户信息
if ($jwtToken) {
    $headers = @{ "Authorization" = "Bearer $jwtToken" }
    
    Test-Api -Name "获取用户信息" -Method "GET" -Endpoint "/api/users/profile" -Headers $headers -Validator {
        param($response)
        return $response -and $response.user_id -eq $userId
    }
    
    # 5. 创建订单
    $orderResponse = Test-Api -Name "创建订单" -Method "POST" -Endpoint "/api/orders" -Headers $headers -Body @{
        picker_id = "550e8400-e29b-41d4-a716-446655440000"
        pay_type = "premium"
    } -Validator {
        param($response)
        return $response -and $response.order_id
    }
    
    # 6. 获取订单列表
    Test-Api -Name "获取订单列表" -Method "GET" -Endpoint "/api/orders" -Headers $headers -Validator {
        param($response)
        return $response -and $response.PSObject.Properties.Name -contains "orders"
    }
}

# 7. 测试错误处理
Test-Api -Name "错误处理测试" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = "invalid-email"
    user_name = "Test"
    user_type = "gen"
} -Validator {
    param($response)
    # 这个测试期望失败，所以如果到这里说明没有正确处理错误
    return $false
}

# 输出测试结果
Write-Host "`n📊 测试结果统计:" -ForegroundColor Cyan
Write-Host "总计: $($Global:TestResults.Total)" -ForegroundColor Gray
Write-Host "通过: $($Global:TestResults.Passed)" -ForegroundColor Green
Write-Host "失败: $($Global:TestResults.Failed)" -ForegroundColor Red

$successRate = [math]::Round(($Global:TestResults.Passed / $Global:TestResults.Total) * 100, 1)
Write-Host "成功率: $successRate%" -ForegroundColor $(if ($successRate -ge 80) { "Green" } else { "Yellow" })

if ($Global:TestResults.Failed -eq 0) {
    Write-Host "`n🎉 所有测试通过！" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  有 $($Global:TestResults.Failed) 个测试失败" -ForegroundColor Yellow
}

Write-Host "`n💡 提示: 使用 .\test-api.ps1 运行完整的交互式测试" -ForegroundColor Cyan