# Debug Test Script for Pickers Server
# This script helps identify specific issues with API responses

param(
    [string]$BaseUrl = "http://localhost:3000"
)

Write-Host "=== Pickers Server Debug Test ===" -ForegroundColor Magenta
Write-Host "Server: $BaseUrl" -ForegroundColor Gray
Write-Host ""

# Function to make detailed API calls with full error information
function Debug-ApiCall {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Endpoint,
        [hashtable]$Body = @{},
        [hashtable]$Headers = @{}
    )
    
    Write-Host "--- Testing: $Name ---" -ForegroundColor Yellow
    Write-Host "Request: $Method $Endpoint" -ForegroundColor Gray
    
    try {
        $params = @{
            Uri = "$BaseUrl$Endpoint"
            Method = $Method
            ContentType = "application/json"
            Headers = $Headers
        }
        
        if ($Body.Count -gt 0) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
            Write-Host "Request Body:" -ForegroundColor Gray
            Write-Host $params.Body -ForegroundColor DarkGray
        }
        
        $response = Invoke-RestMethod @params
        
        Write-Host "SUCCESS" -ForegroundColor Green
        Write-Host "Response:" -ForegroundColor Gray
        Write-Host ($response | ConvertTo-Json -Depth 10) -ForegroundColor DarkGray
        Write-Host ""
        
        return $response
    }
    catch {
        Write-Host "ERROR" -ForegroundColor Red
        Write-Host "Status Code: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
        Write-Host "Status Description: $($_.Exception.Response.StatusDescription)" -ForegroundColor Red
        
        if ($_.ErrorDetails.Message) {
            Write-Host "Error Details: $($_.ErrorDetails.Message)" -ForegroundColor Red
        }
        
        # Try to get response content
        try {
            $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
            $responseBody = $reader.ReadToEnd()
            if ($responseBody) {
                Write-Host "Response Body: $responseBody" -ForegroundColor Red
            }
        }
        catch {
            # Ignore if we can't read response body
        }
        
        Write-Host ""
        return $null
    }
}

# 1. Test server connection
$marketResponse = Debug-ApiCall -Name "Server Connection" -Method "GET" -Endpoint "/api/pickers"

if (-not $marketResponse) {
    Write-Host "Cannot connect to server. Make sure it's running with 'cargo run'" -ForegroundColor Red
    exit 1
}

# 2. Test user registration with detailed error info
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$testEmail = "debug$timestamp@example.com"

Write-Host "Using test email: $testEmail" -ForegroundColor Cyan

$registerResponse = Debug-ApiCall -Name "User Registration" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = $testEmail
    user_name = "Debug Test User"
    user_type = "Gen"
}

if ($registerResponse -and $registerResponse.user_id) {
    $userId = $registerResponse.user_id
    Write-Host "User registered successfully with ID: $userId" -ForegroundColor Green
    
    # 3. Test email verification
    Write-Host "Checking server console for verification code..." -ForegroundColor Yellow
    Write-Host "Common verification codes to try: 123456, 000000, 111111" -ForegroundColor Yellow
    
    $verificationCode = Read-Host "Enter verification code from server console (or press Enter to try 123456)"
    if (-not $verificationCode) {
        $verificationCode = "123456"
    }
    
    $verifyResponse = Debug-ApiCall -Name "Email Verification" -Method "POST" -Endpoint "/api/users/verify" -Body @{
        email = $testEmail
        code = $verificationCode
    }
    
    if ($verifyResponse -and $verifyResponse.token) {
        $jwtToken = $verifyResponse.token
        Write-Host "JWT Token obtained: $($jwtToken.Substring(0, 20))..." -ForegroundColor Green
        
        # 4. Test authenticated endpoint
        $headers = @{ "Authorization" = "Bearer $jwtToken" }
        
        $profileResponse = Debug-ApiCall -Name "Get User Profile" -Method "GET" -Endpoint "/api/users/profile" -Headers $headers
        
        if ($profileResponse) {
            Write-Host "User profile retrieved successfully" -ForegroundColor Green
            
            # 5. Test order creation
            $orderResponse = Debug-ApiCall -Name "Create Order" -Method "POST" -Endpoint "/api/orders" -Headers $headers -Body @{
                picker_id = "550e8400-e29b-41d4-a716-446655440000"
                pay_type = "premium"
            }
            
            if ($orderResponse -and $orderResponse.order_id) {
                Write-Host "Order created successfully: $($orderResponse.order_id)" -ForegroundColor Green
                
                # 6. Test order list
                Debug-ApiCall -Name "Get Order List" -Method "GET" -Endpoint "/api/orders" -Headers $headers
            }
        }
    }
}

# 7. Test error handling
Write-Host "--- Testing Error Handling ---" -ForegroundColor Yellow
Debug-ApiCall -Name "Invalid Email Format" -Method "POST" -Endpoint "/api/users/register" -Body @{
    email = "not-an-email"
    user_name = "Test"
    user_type = "Gen"
}

# 8. Test unauthorized access
Debug-ApiCall -Name "Unauthorized Access" -Method "GET" -Endpoint "/api/users/profile"

Write-Host "=== Debug Test Complete ===" -ForegroundColor Magenta