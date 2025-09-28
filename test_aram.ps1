# Test ARAM functionality
Write-Host "=== Testing ARAM Functionality ===" -ForegroundColor Green

# Test 1: Check if HTTP server is running
Write-Host "1. Testing HTTP server connection..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8000/health" -Method Get -TimeoutSec 5
    Write-Host "✅ HTTP server is running" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ HTTP server not responding: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Test LCU connection
Write-Host "2. Testing LCU connection..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8000/lol/test" -Method Get -TimeoutSec 10
    Write-Host "✅ LCU connection test successful" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ LCU connection test failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Test champion select endpoint
Write-Host "3. Testing champion select endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8000/lol/champ-select" -Method Get -TimeoutSec 10
    Write-Host "✅ Champion select endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Champion select endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Test live game endpoint
Write-Host "4. Testing live game endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8000/lol/live-game" -Method Get -TimeoutSec 10
    Write-Host "✅ Live game endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Live game endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Test AI champion select endpoint
Write-Host "5. Testing AI champion select endpoint..." -ForegroundColor Yellow
try {
    $body = @{
        gameMode = "ARAM"
        teamComposition = @{
            allies = @()
            enemies = @()
        }
    } | ConvertTo-Json
    
    $response = Invoke-RestMethod -Uri "http://localhost:8000/ai/champ-select" -Method Post -Body $body -ContentType "application/json" -TimeoutSec 15
    Write-Host "✅ AI champion select endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ AI champion select endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "=== ARAM Testing Complete ===" -ForegroundColor Green
