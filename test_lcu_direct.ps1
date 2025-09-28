# Test LCU connection directly
Write-Host "=== Testing LCU Connection Directly ===" -ForegroundColor Green

# Ignore SSL certificate errors
[System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}

$port = 59536
$common_passwords = @("riot", "riot123", "league", "lol")

foreach ($password in $common_passwords) {
    Write-Host "Testing password: $password" -ForegroundColor Yellow
    
    try {
        $url = "https://127.0.0.1:$port/lol-summoner/v1/current-summoner"
        $headers = @{
            'Authorization' = "Basic $([Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("riot:$password")))"
            'Content-Type' = 'application/json'
        }
        
        $response = Invoke-RestMethod -Uri $url -Headers $headers -Method Get -TimeoutSec 5
        Write-Host "✅ Success with password: $password" -ForegroundColor Green
        Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
        break
    } catch {
        Write-Host "❌ Failed with password: $password - $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test champion select endpoint
Write-Host "`nTesting champion select endpoint..." -ForegroundColor Yellow
try {
    $url = "https://127.0.0.1:$port/lol-champ-select/v1/session"
    $headers = @{
        'Authorization' = "Basic $([Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("riot:riot")))"
        'Content-Type' = 'application/json'
    }
    
    $response = Invoke-RestMethod -Uri $url -Headers $headers -Method Get -TimeoutSec 5
    Write-Host "✅ Champion select endpoint working" -ForegroundColor Green
    Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Champion select endpoint failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n=== LCU Direct Test Complete ===" -ForegroundColor Green
