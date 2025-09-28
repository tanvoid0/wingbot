# Test common League client ports
Write-Host "=== Testing League Client Ports ===" -ForegroundColor Green

$common_ports = @(5000, 5001, 5002, 5003, 5004, 5005, 5006, 5007, 5008, 5009, 5010)
$common_passwords = @("riot", "riot123", "league", "lol")

foreach ($port in $common_ports) {
    Write-Host "Testing port $port..." -ForegroundColor Yellow
    
    foreach ($password in $common_passwords) {
        try {
            $url = "https://127.0.0.1:$port/lol-summoner/v1/current-summoner"
            $headers = @{
                'Authorization' = "Basic $password"
                'Content-Type' = 'application/json'
            }
            
            # Ignore SSL certificate errors
            [System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}
            
            $response = Invoke-RestMethod -Uri $url -Headers $headers -Method Get -TimeoutSec 2
            Write-Host "✅ SUCCESS! Port: $port, Password: $password" -ForegroundColor Green
            Write-Host "Response: $($response | ConvertTo-Json -Depth 2)" -ForegroundColor Cyan
            exit 0
        } catch {
            # Continue to next password
        }
    }
}

Write-Host "❌ No working connection found on common ports" -ForegroundColor Red
