# Test LCU Connection
Write-Host "=== Testing LCU Connection ===" -ForegroundColor Green

# Check if League client is running
Write-Host "1. Checking if League client is running..." -ForegroundColor Yellow
$leagueProcess = Get-Process -Name "LeagueClientUx" -ErrorAction SilentlyContinue
if ($leagueProcess) {
    Write-Host "✅ League client is running (PID: $($leagueProcess.Id))" -ForegroundColor Green
} else {
    Write-Host "❌ League client not found" -ForegroundColor Red
    exit 1
}

# Get command line arguments
Write-Host "2. Getting League client command line..." -ForegroundColor Yellow
try {
    $wmiQuery = "SELECT CommandLine FROM Win32_Process WHERE Name='LeagueClientUx.exe'"
    $processes = Get-WmiObject -Query $wmiQuery
    
    if ($processes) {
        $commandLine = $processes[0].CommandLine
        Write-Host "Raw command line:" -ForegroundColor Cyan
        Write-Host $commandLine -ForegroundColor Gray
        
        # Extract port
        Write-Host "3. Extracting port..." -ForegroundColor Yellow
        if ($commandLine -match "--app-port=(\d+)") {
            $port = $matches[1]
            Write-Host "✅ Found port: $port" -ForegroundColor Green
        } else {
            Write-Host "❌ Port not found" -ForegroundColor Red
        }
        
        # Extract password
        Write-Host "4. Extracting password..." -ForegroundColor Yellow
        if ($commandLine -match "--remoting-auth-token=([a-zA-Z0-9_-]+)") {
            $password = $matches[1]
            Write-Host "✅ Found password: $($password.Substring(0,8))..." -ForegroundColor Green
        } else {
            Write-Host "❌ Password not found" -ForegroundColor Red
        }
        
        # Test connection
        Write-Host "5. Testing LCU connection..." -ForegroundColor Yellow
        if ($port -and $password) {
            $url = "https://127.0.0.1:$port/lol-champ-select/v1/session"
            $headers = @{
                'Authorization' = "Basic $password"
                'Content-Type' = 'application/json'
            }
            
            try {
                # Ignore SSL certificate errors
                [System.Net.ServicePointManager]::ServerCertificateValidationCallback = {$true}
                $response = Invoke-RestMethod -Uri $url -Headers $headers -Method Get
                Write-Host "✅ LCU connection successful!" -ForegroundColor Green
                Write-Host "Champion select data:" -ForegroundColor Cyan
                $response | ConvertTo-Json -Depth 3
            } catch {
                Write-Host "❌ LCU connection failed: $($_.Exception.Message)" -ForegroundColor Red
            }
        }
    } else {
        Write-Host "❌ Could not get command line for League client" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}
