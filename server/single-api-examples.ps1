# Pickers Server Individual API Test Examples
# Individual Invoke-RestMethod commands for each API endpoint

# Configuration
$BaseUrl = "http://localhost:3000"
$ContentType = "application/json"

Write-Host "=== Pickers Server API Test Examples ===" -ForegroundColor Magenta
Write-Host "Server: $BaseUrl" -ForegroundColor Gray
Write-Host ""

# Helper function for formatted output
function Show-ApiExample {
    param(
        [string]$Title,
        [string]$Command,
        [string]$Description = ""
    )
    
    Write-Host "## $Title" -ForegroundColor Yellow
    if ($Description) {
        Write-Host "   $Description" -ForegroundColor Gray
    }
    Write-Host $Command -ForegroundColor Cyan
    Write-Host ""
}

# 1. User Registration Examples
Show-ApiExample -Title "User Registration (General User)" -Description "Register a new general user" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/register" -Method POST -ContentType "$ContentType" -Body '{
    "email": "user@example.com",
    "user_name": "Test User",
    "user_type": "Gen"
}'
Write-Host "User ID: `$(`$response.user_id)"
"@

Show-ApiExample -Title "User Registration (Developer)" -Description "Register a new developer user" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/register" -Method POST -ContentType "$ContentType" -Body '{
    "email": "dev@example.com", 
    "user_name": "Developer User",
    "user_type": "Dev"
}'
Write-Host "User ID: `$(`$response.user_id)"
"@

# 2. Email Verification Examples
Show-ApiExample -Title "Email Verification" -Description "Verify email with code (check server console for code)" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/verify" -Method POST -ContentType "$ContentType" -Body '{
    "email": "user@example.com",
    "code": "123456"
}'
`$token = `$response.token
Write-Host "JWT Token: `$token"
Write-Host "User Info: `$(`$response.user | ConvertTo-Json)"
"@

# 3. User Login Examples
Show-ApiExample -Title "User Login" -Description "Login with email (generates new verification code)" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/login" -Method POST -ContentType "$ContentType" -Body '{
    "email": "user@example.com"
}'
Write-Host "Login response: `$(`$response | ConvertTo-Json)"
"@

# 4. Get User Profile Examples
Show-ApiExample -Title "Get User Profile" -Description "Get current user profile (requires JWT token)" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/profile" -Method GET -Headers `$headers
Write-Host "User Profile: `$(`$response | ConvertTo-Json)"
"@

# 5. Market List Examples
Show-ApiExample -Title "Get Market List (Basic)" -Description "Get all pickers in the market" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/pickers" -Method GET
Write-Host "Total Pickers: `$(`$response.total)"
Write-Host "Pickers: `$(`$response.pickers | ConvertTo-Json)"
"@

Show-ApiExample -Title "Get Market List (Paginated)" -Description "Get pickers with pagination" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/pickers?page=1&size=10" -Method GET
Write-Host "Page 1 Results: `$(`$response | ConvertTo-Json)"
"@

Show-ApiExample -Title "Search Market" -Description "Search pickers by keyword" -Command @"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/pickers?keyword=test&sort=price_asc" -Method GET
Write-Host "Search Results: `$(`$response | ConvertTo-Json)"
"@

# 6. Picker Detail Examples
Show-ApiExample -Title "Get Picker Detail" -Description "Get detailed information about a specific picker" -Command @"
`$pickerId = "550e8400-e29b-41d4-a716-446655440000"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/pickers/`$pickerId" -Method GET
Write-Host "Picker Detail: `$(`$response | ConvertTo-Json)"
"@

# 7. Order Creation Examples
Show-ApiExample -Title "Create Order (Premium Payment)" -Description "Create order using premium points" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/orders" -Method POST -ContentType "$ContentType" -Headers `$headers -Body '{
    "picker_id": "550e8400-e29b-41d4-a716-446655440000",
    "pay_type": "premium"
}'
Write-Host "Order ID: `$(`$response.order_id)"
Write-Host "Order Status: `$(`$response.status)"
"@

Show-ApiExample -Title "Create Order (Wallet Payment)" -Description "Create order using wallet payment" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/orders" -Method POST -ContentType "$ContentType" -Headers `$headers -Body '{
    "picker_id": "550e8400-e29b-41d4-a716-446655440000",
    "pay_type": "wallet",
    "tx_hash": "0x1234567890abcdef1234567890abcdef12345678"
}'
Write-Host "Order ID: `$(`$response.order_id)"
Write-Host "Payment Status: `$(`$response.status)"
"@

# 8. Order Management Examples
Show-ApiExample -Title "Get Order Detail" -Description "Get detailed information about a specific order" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$orderId = "YOUR_ORDER_ID_HERE"
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/orders/`$orderId" -Method GET -Headers `$headers
Write-Host "Order Detail: `$(`$response | ConvertTo-Json)"
"@

Show-ApiExample -Title "Get Order List (All)" -Description "Get all orders for current user" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/orders" -Method GET -Headers `$headers
Write-Host "Total Orders: `$(`$response.pagination.total)"
Write-Host "Orders: `$(`$response.orders | ConvertTo-Json)"
"@

Show-ApiExample -Title "Get Order List (Filtered)" -Description "Get orders filtered by status" -Command @"
`$headers = @{ "Authorization" = "Bearer YOUR_JWT_TOKEN_HERE" }
`$response = Invoke-RestMethod -Uri "$BaseUrl/api/orders?status=success&page=1&limit=10" -Method GET -Headers `$headers
Write-Host "Successful Orders: `$(`$response | ConvertTo-Json)"
"@

# 9. File Download Examples
Show-ApiExample -Title "Download File" -Description "Download purchased picker file using download token" -Command @"
`$downloadToken = "YOUR_DOWNLOAD_TOKEN_HERE"
Invoke-RestMethod -Uri "$BaseUrl/download?token=`$downloadToken" -Method GET -OutFile "downloaded_picker.zip"
Write-Host "File downloaded as: downloaded_picker.zip"
"@

# 10. Error Handling Examples
Show-ApiExample -Title "Test Invalid Email" -Description "Test error handling with invalid email format" -Command @"
try {
    `$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/register" -Method POST -ContentType "$ContentType" -Body '{
        "email": "invalid-email-format",
        "user_name": "Test User",
        "user_type": "Gen"
    }'
}
catch {
    Write-Host "Error caught: `$(`$_.Exception.Message)" -ForegroundColor Red
    Write-Host "Status Code: `$(`$_.Exception.Response.StatusCode)" -ForegroundColor Red
}
"@

Show-ApiExample -Title "Test Unauthorized Access" -Description "Test accessing protected endpoint without token" -Command @"
try {
    `$response = Invoke-RestMethod -Uri "$BaseUrl/api/users/profile" -Method GET
}
catch {
    Write-Host "Error: `$(`$_.Exception.Message)" -ForegroundColor Red
    Write-Host "This is expected - endpoint requires authentication"
}
"@

# 11. Complete Workflow Example
Write-Host "## Complete Workflow Example" -ForegroundColor Yellow
Write-Host "   Full user registration to order creation workflow" -ForegroundColor Gray
Write-Host @"
# Complete workflow from registration to order
`$email = "workflow@example.com"

# 1. Register user
`$registerResponse = Invoke-RestMethod -Uri "$BaseUrl/api/users/register" -Method POST -ContentType "$ContentType" -Body "{
    `"email`": `"`$email`",
    `"user_name`": `"Workflow User`",
    `"user_type`": `"Gen`"
}"
Write-Host "Registered User ID: `$(`$registerResponse.user_id)"

# 2. Verify email (check server console for verification code)
`$verifyResponse = Invoke-RestMethod -Uri "$BaseUrl/api/users/verify" -Method POST -ContentType "$ContentType" -Body "{
    `"email`": `"`$email`",
    `"code`": `"123456`"
}"
`$token = `$verifyResponse.token
Write-Host "JWT Token obtained"

# 3. Get user profile
`$headers = @{ "Authorization" = "Bearer `$token" }
`$profile = Invoke-RestMethod -Uri "$BaseUrl/api/users/profile" -Method GET -Headers `$headers
Write-Host "User Premium Balance: `$(`$profile.premium_amount)"

# 4. Browse market
`$market = Invoke-RestMethod -Uri "$BaseUrl/api/pickers" -Method GET
Write-Host "Available Pickers: `$(`$market.total)"

# 5. Create order
`$orderResponse = Invoke-RestMethod -Uri "$BaseUrl/api/orders" -Method POST -ContentType "$ContentType" -Headers `$headers -Body '{
    "picker_id": "550e8400-e29b-41d4-a716-446655440000",
    "pay_type": "premium"
}'
Write-Host "Order Created: `$(`$orderResponse.order_id)"

# 6. Check order status
`$orderDetail = Invoke-RestMethod -Uri "$BaseUrl/api/orders/`$(`$orderResponse.order_id)" -Method GET -Headers `$headers
Write-Host "Order Status: `$(`$orderDetail.status)"
"@ -ForegroundColor Cyan

Write-Host ""
Write-Host "=== Usage Tips ===" -ForegroundColor Magenta
Write-Host "1. Replace YOUR_JWT_TOKEN_HERE with actual token from verification response" -ForegroundColor Gray
Write-Host "2. Replace YOUR_ORDER_ID_HERE with actual order ID from create order response" -ForegroundColor Gray
Write-Host "3. Check server console for verification codes during testing" -ForegroundColor Gray
Write-Host "4. Use -Verbose flag with Invoke-RestMethod for detailed request/response info" -ForegroundColor Gray
Write-Host "5. Wrap commands in try-catch blocks for proper error handling" -ForegroundColor Gray