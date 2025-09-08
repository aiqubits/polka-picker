# Pickers Server Quick API Test Script (English Version)
# Optimized version for reliable testing

param(
    [string]$BaseUrl = "http://localhost:3000"
)

# Test results statistics
$Global:TestResults = @{
    Passed = 0
    Failed = 0
    Total = 0
}

# Generate unique test data
$Global:TestTimestamp = Get-Date -Format "yyyyMMddHHmmss"
$Global:TestEmail = "test$Global:TestTimestamp@example.com"

function Test-Api {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Endpoint,
        [hashtable]$Body = @{},
        [hashtable]$Headers = @{},
        [scriptblock]$Validator = { $true },
        [switch]$ExpectError
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
        
        if ($ExpectError) {
            Write-Host "FAIL $Name - Expected error but got success" -ForegroundColor Red
            $Global:TestResults.Failed++
            return $null
        }
        
        if (& $Validator $response) {
            Write-Host "PASS $Name" -ForegroundColor Green
            $Global:TestResults.Passed++
            return $response
        } else {
            Write-Host "FAIL $Name - Validation failed" -ForegroundColor Red
            $Global:TestResults.Failed++
            return $null
        }
    }
    catch {
        if ($ExpectError) {
            Write-Host "PASS $Name - Expected error caught: $($_.Exception.Response.StatusCode)" -ForegroundColor Green
            $Global:TestResults.Passed++
            return $null
        } else {
            Write-Host "FAIL $Name - $($_.Exception.Message)" -ForegroundColor Red
            $Global:TestResults.Failed++
            return $null
        }
    }
}

# Quick test flow
Write-Host "Pickers Server Quick Test Started" -ForegroundColor Cyan
Write-Host "Server: $BaseUrl" -ForegroundColor Gray

# 1. Test server connection
$marketResponse = Test-Api -Name "Server Connection" -Method "GET" -Endpoint "/api/pickers" -Validator {
    param($response)
    return $response -and $response.PSObject.Properties.Name -contains "pickers"
}

if (-not $marketResponse) {
    Write-Host "Server connection failed, stopping tests" -ForegroundColor Red
    exit 1
}

# 2. User registration with unique email
Write-Host "INFO: Using unique email: $Global:TestEmail" -ForegroundColor Yellow

$registerResponse = Test-Api -Name "User Registration" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = $Global:TestEmail
    user_name = "Quick Test User $Global:TestTimestamp"
    user_type = "Gen"
} -Validator {
    param($response)
    return $response -and $response.user_id
}

$userId = $null
if ($registerResponse) {
    $userId = $registerResponse.user_id
    Write-Host "INFO: Registered user ID: $userId" -ForegroundColor Cyan
}

# 3. Email verification (interactive mode for better success rate)
$jwtToken = $null
if ($userId) {
    Write-Host "INFO: Email verification needed for user: $Global:TestEmail" -ForegroundColor Yellow
    Write-Host "INFO: Check the server console (cargo run terminal) for verification code" -ForegroundColor Cyan
    Write-Host "INFO: Look for a line like: 'Verification code for $Global:TestEmail: XXXXXX'" -ForegroundColor Cyan
    
    # First try common codes automatically
    $commonCodes = @("123456", "000000", "111111", "654321")
    $codeWorked = $false
    
    foreach ($code in $commonCodes) {
        try {
            $verifyResponse = Invoke-RestMethod -Uri "$BaseUrl/api/users/verify" -Method POST -ContentType "application/json" -Body (@{
                email = $Global:TestEmail
                code = $code
            } | ConvertTo-Json)
            
            if ($verifyResponse -and $verifyResponse.token) {
                Write-Host "PASS Email Verification - Code $code worked automatically" -ForegroundColor Green
                $Global:TestResults.Total++
                $Global:TestResults.Passed++
                $jwtToken = $verifyResponse.token
                $codeWorked = $true
                break
            }
        }
        catch {
            continue
        }
    }
    
    # If common codes didn't work, ask user for the actual code
    if (-not $codeWorked) {
        Write-Host ""
        Write-Host "Common codes didn't work. Please check server console for the actual verification code." -ForegroundColor Yellow
        $actualCode = Read-Host "Enter verification code from server console (or press Enter to skip)"
        
        if ($actualCode) {
            try {
                $verifyResponse = Invoke-RestMethod -Uri "$BaseUrl/api/users/verify" -Method POST -ContentType "application/json" -Body (@{
                    email = $Global:TestEmail
                    code = $actualCode
                } | ConvertTo-Json)
                
                if ($verifyResponse -and $verifyResponse.token) {
                    Write-Host "PASS Email Verification - Manual code worked" -ForegroundColor Green
                    $Global:TestResults.Total++
                    $Global:TestResults.Passed++
                    $jwtToken = $verifyResponse.token
                } else {
                    Write-Host "FAIL Email Verification - Manual code didn't work" -ForegroundColor Red
                    $Global:TestResults.Total++
                    $Global:TestResults.Failed++
                }
            }
            catch {
                Write-Host "FAIL Email Verification - Error with manual code: $($_.Exception.Message)" -ForegroundColor Red
                $Global:TestResults.Total++
                $Global:TestResults.Failed++
            }
        } else {
            Write-Host "SKIP Email Verification - No code provided" -ForegroundColor Yellow
            $Global:TestResults.Total++
            $Global:TestResults.Failed++
        }
    }
} else {
    Write-Host "SKIP Email Verification - User registration failed" -ForegroundColor Yellow
}

# 4. Get user profile
if ($jwtToken) {
    $headers = @{ "Authorization" = "Bearer $jwtToken" }
    
    Test-Api -Name "Get User Profile" -Method "GET" -Endpoint "/api/users/profile" -Headers $headers -Validator {
        param($response)
        return $response -and $response.user_id -eq $userId
    }
    
    # 5. Create order
    $orderResponse = Test-Api -Name "Create Order" -Method "POST" -Endpoint "/api/orders" -Headers $headers -Body @{
        picker_id = "550e8400-e29b-41d4-a716-446655440000"
        pay_type = "premium"
    } -Validator {
        param($response)
        return $response -and $response.order_id
    }
    
    # 6. Get order list
    Test-Api -Name "Get Order List" -Method "GET" -Endpoint "/api/orders" -Headers $headers -Validator {
        param($response)
        return $response -and $response.PSObject.Properties.Name -contains "orders"
    }
}

# 7. Test error handling (expect this to fail)
Test-Api -Name "Error Handling Test" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = "invalid-email-format"
    user_name = "Test"
    user_type = "Gen"
} -ExpectError

# Output test results
Write-Host ""
Write-Host "Test Results Summary:" -ForegroundColor Cyan
Write-Host "Total: $($Global:TestResults.Total)" -ForegroundColor Gray
Write-Host "Passed: $($Global:TestResults.Passed)" -ForegroundColor Green
Write-Host "Failed: $($Global:TestResults.Failed)" -ForegroundColor Red

$successRate = [math]::Round(($Global:TestResults.Passed / $Global:TestResults.Total) * 100, 1)
Write-Host "Success Rate: $successRate%" -ForegroundColor $(if ($successRate -ge 80) { "Green" } else { "Yellow" })

if ($Global:TestResults.Failed -eq 0) {
    Write-Host ""
    Write-Host "All tests passed!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "$($Global:TestResults.Failed) test(s) failed" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "TIP: Use .\test-api.ps1 for full interactive testing" -ForegroundColor Cyan