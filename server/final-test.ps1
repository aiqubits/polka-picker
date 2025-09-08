# Final Optimized Test Script for Pickers Server
# This script provides the best testing experience with automatic verification code handling

param(
    [string]$BaseUrl = "http://localhost:3000",
    [switch]$Interactive = $false
)

# Test results
$Global:TestResults = @{
    Passed = 0
    Failed = 0
    Total = 0
    Details = @()
}

function Add-TestResult {
    param(
        [string]$Name,
        [bool]$Success,
        [string]$Message = ""
    )
    
    $Global:TestResults.Total++
    if ($Success) {
        $Global:TestResults.Passed++
        Write-Host "✅ PASS: $Name" -ForegroundColor Green
    } else {
        $Global:TestResults.Failed++
        Write-Host "❌ FAIL: $Name" -ForegroundColor Red
        if ($Message) {
            Write-Host "   $Message" -ForegroundColor Yellow
        }
    }
    
    $Global:TestResults.Details += @{
        Name = $Name
        Success = $Success
        Message = $Message
    }
}

function Test-ApiEndpoint {
    param(
        [string]$Method,
        [string]$Endpoint,
        [hashtable]$Body = @{},
        [hashtable]$Headers = @{},
        [bool]$ExpectError = $false
    )
    
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
        
        if ($ExpectError) {
            return @{ Success = $false; Response = $response; Error = "Expected error but got success" }
        } else {
            return @{ Success = $true; Response = $response; Error = $null }
        }
    }
    catch {
        if ($ExpectError) {
            return @{ Success = $true; Response = $null; Error = "Expected error: $($_.Exception.Response.StatusCode)" }
        } else {
            return @{ Success = $false; Response = $null; Error = $_.Exception.Message }
        }
    }
}

Write-Host "🚀 Pickers Server Final Test Suite" -ForegroundColor Magenta
Write-Host "Server: $BaseUrl" -ForegroundColor Gray
Write-Host "Interactive Mode: $Interactive" -ForegroundColor Gray
Write-Host ""

# Generate unique test data
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$testEmail = "finaltest$timestamp@example.com"
$jwtToken = $null
$userId = $null

# Test 1: Server Connection
Write-Host "🔗 Testing server connection..." -ForegroundColor Cyan
$result = Test-ApiEndpoint -Method "GET" -Endpoint "/api/pickers"
if ($result.Success -and $result.Response.PSObject.Properties.Name -contains "pickers") {
    Add-TestResult -Name "Server Connection" -Success $true
} else {
    Add-TestResult -Name "Server Connection" -Success $false -Message "Cannot connect to server or invalid response"
    Write-Host "❌ Cannot continue without server connection" -ForegroundColor Red
    exit 1
}

# Test 2: User Registration
Write-Host "👤 Testing user registration..." -ForegroundColor Cyan
$result = Test-ApiEndpoint -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = $testEmail
    user_name = "Final Test User $timestamp"
    user_type = "Gen"
}

if ($result.Success -and $result.Response.user_id) {
    $userId = $result.Response.user_id
    Add-TestResult -Name "User Registration" -Success $true
    Write-Host "   User ID: $userId" -ForegroundColor Gray
} else {
    Add-TestResult -Name "User Registration" -Success $false -Message $result.Error
}

# Test 3: Email Verification
if ($userId) {
    Write-Host "📧 Testing email verification..." -ForegroundColor Cyan
    
    # Try common verification codes first
    $commonCodes = @("123456", "000000", "111111", "654321", "999999")
    $verificationSuccess = $false
    
    foreach ($code in $commonCodes) {
        $result = Test-ApiEndpoint -Method "POST" -Endpoint "/api/users/verify" -Body @{
            email = $testEmail
            code = $code
        }
        
        if ($result.Success -and $result.Response.token) {
            $jwtToken = $result.Response.token
            $verificationSuccess = $true
            Add-TestResult -Name "Email Verification" -Success $true
            Write-Host "   Code $code worked automatically" -ForegroundColor Gray
            Write-Host "   JWT Token obtained" -ForegroundColor Gray
            break
        }
    }
    
    # If automatic codes didn't work and interactive mode is enabled
    if (-not $verificationSuccess -and $Interactive) {
        Write-Host ""
        Write-Host "⚠️  Automatic verification codes didn't work" -ForegroundColor Yellow
        Write-Host "📋 Please check the server console for the verification code" -ForegroundColor Cyan
        Write-Host "🔍 Look for: 'Verification code for ${testEmail}: XXXXXX'" -ForegroundColor Cyan
        
        $manualCode = Read-Host "Enter verification code from server console"
        
        if ($manualCode) {
            $result = Test-ApiEndpoint -Method "POST" -Endpoint "/api/users/verify" -Body @{
                email = $testEmail
                code = $manualCode
            }
            
            if ($result.Success -and $result.Response.token) {
                $jwtToken = $result.Response.token
                Add-TestResult -Name "Email Verification (Manual)" -Success $true
            } else {
                Add-TestResult -Name "Email Verification (Manual)" -Success $false -Message $result.Error
            }
        } else {
            Add-TestResult -Name "Email Verification" -Success $false -Message "No verification code provided"
        }
    } elseif (-not $verificationSuccess) {
        Add-TestResult -Name "Email Verification" -Success $false -Message "Common codes failed. Use -Interactive for manual input"
    }
} else {
    Write-Host "⏭️  Skipping email verification (registration failed)" -ForegroundColor Yellow
}

# Test 4: Get User Profile (requires authentication)
if ($jwtToken) {
    Write-Host "👤 Testing authenticated user profile..." -ForegroundColor Cyan
    $headers = @{ "Authorization" = "Bearer $jwtToken" }
    
    $result = Test-ApiEndpoint -Method "GET" -Endpoint "/api/users/profile" -Headers $headers
    
    if ($result.Success -and $result.Response.user_id -eq $userId) {
        Add-TestResult -Name "Get User Profile" -Success $true
        Write-Host "   User: $($result.Response.user_name)" -ForegroundColor Gray
        Write-Host "   Premium: $($result.Response.premium_amount)" -ForegroundColor Gray
    } else {
        Add-TestResult -Name "Get User Profile" -Success $false -Message $result.Error
    }
} else {
    Write-Host "⏭️  Skipping user profile test (no JWT token)" -ForegroundColor Yellow
}

# Test 5: Create Order (requires authentication)
if ($jwtToken) {
    Write-Host "🛒 Testing order creation..." -ForegroundColor Cyan
    $headers = @{ "Authorization" = "Bearer $jwtToken" }
    
    $result = Test-ApiEndpoint -Method "POST" -Endpoint "/api/orders" -Headers $headers -Body @{
        picker_id = "550e8400-e29b-41d4-a716-446655440000"
        pay_type = "premium"
    }
    
    if ($result.Success -and $result.Response.order_id) {
        Add-TestResult -Name "Create Order" -Success $true
        Write-Host "   Order ID: $($result.Response.order_id)" -ForegroundColor Gray
        
        # Test 6: Get Order List
        Write-Host "📋 Testing order list..." -ForegroundColor Cyan
        $listResult = Test-ApiEndpoint -Method "GET" -Endpoint "/api/orders" -Headers $headers
        
        if ($listResult.Success -and $listResult.Response.PSObject.Properties.Name -contains "orders") {
            Add-TestResult -Name "Get Order List" -Success $true
            Write-Host "   Orders found: $($listResult.Response.orders.Count)" -ForegroundColor Gray
        } else {
            Add-TestResult -Name "Get Order List" -Success $false -Message $listResult.Error
        }
    } else {
        Add-TestResult -Name "Create Order" -Success $false -Message $result.Error
    }
} else {
    Write-Host "⏭️  Skipping order tests (no JWT token)" -ForegroundColor Yellow
}

# Test 7: Error Handling
Write-Host "🚫 Testing error handling..." -ForegroundColor Cyan
$result = Test-ApiEndpoint -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = "invalid-email-format"
    user_name = "Test"
    user_type = "Gen"
} -ExpectError $true

Add-TestResult -Name "Error Handling" -Success $result.Success -Message $result.Error

# Test 8: Unauthorized Access
Write-Host "🔒 Testing unauthorized access..." -ForegroundColor Cyan
$result = Test-ApiEndpoint -Method "GET" -Endpoint "/api/users/profile" -ExpectError $true

Add-TestResult -Name "Unauthorized Access" -Success $result.Success -Message $result.Error

# Final Results
Write-Host ""
Write-Host "📊 Test Results Summary" -ForegroundColor Magenta
Write-Host "===========================================" -ForegroundColor Gray

$successRate = if ($Global:TestResults.Total -gt 0) { 
    [math]::Round(($Global:TestResults.Passed / $Global:TestResults.Total) * 100, 1) 
} else { 0 }

Write-Host "Total Tests: $($Global:TestResults.Total)" -ForegroundColor White
Write-Host "Passed: $($Global:TestResults.Passed)" -ForegroundColor Green
Write-Host "Failed: $($Global:TestResults.Failed)" -ForegroundColor Red
Write-Host "Success Rate: $successRate%" -ForegroundColor $(if ($successRate -ge 80) { "Green" } elseif ($successRate -ge 60) { "Yellow" } else { "Red" })

Write-Host ""
if ($Global:TestResults.Failed -eq 0) {
    Write-Host "🎉 All tests passed! The Pickers Server is working perfectly!" -ForegroundColor Green
} elseif ($successRate -ge 75) {
    Write-Host "✨ Most tests passed! The server is working well with minor issues." -ForegroundColor Yellow
} else {
    Write-Host "⚠️  Some tests failed. Check the server implementation." -ForegroundColor Red
}

# Detailed results
if ($Global:TestResults.Failed -gt 0) {
    Write-Host ""
    Write-Host "❌ Failed Tests Details:" -ForegroundColor Red
    foreach ($detail in $Global:TestResults.Details) {
        if (-not $detail.Success) {
            Write-Host "   • $($detail.Name): $($detail.Message)" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "💡 Tips:" -ForegroundColor Cyan
Write-Host "   • Use -Interactive flag for manual verification code input" -ForegroundColor Gray
Write-Host "   • Check server console for verification codes" -ForegroundColor Gray
Write-Host "   • Ensure server is running with 'cargo run'" -ForegroundColor Gray

if ($Interactive) {
    Write-Host ""
    Write-Host "Press any key to exit..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}