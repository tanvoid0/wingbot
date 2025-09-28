# Test script to verify Live Client Data API is working
# This tests the same endpoints that the fixed Tauri app now uses

Write-Host "Testing Live Client Data API..." -ForegroundColor Green
Write-Host "This script tests if League's Live Client Data API is accessible on port 2999" -ForegroundColor Yellow
Write-Host ""

# Test endpoints
$baseUrl = "https://127.0.0.1:2999"
$endpoints = @(
    "/liveclientdata/allgamedata",
    "/liveclientdata/gamestats"
)

# Ignore SSL certificate errors (League uses self-signed cert)
if (-not ([System.Management.Automation.PSTypeName]'ServerCertificateValidationCallback').Type) {
    Add-Type @"
        using System;
        using System.Net;
        using System.Net.Security;
        using System.Security.Cryptography.X509Certificates;
        public class ServerCertificateValidationCallback
        {
            public static void Ignore()
            {
                if(ServicePointManager.ServerCertificateValidationCallback ==null)
                {
                    ServicePointManager.ServerCertificateValidationCallback += 
                        delegate
                        (
                            Object obj, 
                            X509Certificate certificate, 
                            X509Chain chain, 
                            SslPolicyErrors errors
                        )
                        {
                            return true;
                        };
                }
            }
        }
"@
}
[ServerCertificateValidationCallback]::Ignore()

foreach ($endpoint in $endpoints) {
    $url = $baseUrl + $endpoint
    Write-Host "Testing: $url" -ForegroundColor Cyan
    
    try {
        $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 5
        if ($response.StatusCode -eq 200) {
            Write-Host "✅ SUCCESS - Status: $($response.StatusCode)" -ForegroundColor Green
            Write-Host "Response length: $($response.Content.Length) bytes" -ForegroundColor Gray
            
            # Try to parse as JSON and show some key fields
            try {
                $data = $response.Content | ConvertFrom-Json
                if ($endpoint -eq "/liveclientdata/allgamedata") {
                    Write-Host "Game Mode: $($data.gameData.gameMode)" -ForegroundColor Magenta
                    Write-Host "Game Time: $($data.gameData.gameTime) seconds" -ForegroundColor Magenta
                    Write-Host "Active Player: $($data.activePlayer.summonerName)" -ForegroundColor Magenta
                }
                if ($endpoint -eq "/liveclientdata/gamestats") {
                    Write-Host "Game Mode: $($data.gameMode)" -ForegroundColor Magenta
                    Write-Host "Game Time: $($data.gameTime) seconds" -ForegroundColor Magenta
                }
            } catch {
                Write-Host "Could not parse JSON response" -ForegroundColor Yellow
            }
        }
    } catch {
        $errorMsg = $_.Exception.Message
        if ($errorMsg -like "*No connection could be made*") {
            Write-Host "❌ FAILED - League's Live Client Data API not available" -ForegroundColor Red
            Write-Host "This means you're not in a live game or League is not running" -ForegroundColor Yellow
        } else {
            Write-Host "❌ FAILED - Error: $errorMsg" -ForegroundColor Red
        }
    }
    Write-Host ""
}

Write-Host ""
Write-Host "Instructions:" -ForegroundColor Yellow
Write-Host "1. Start League of Legends" -ForegroundColor White
Write-Host "2. Enter a live game (Practice Tool, Custom Game, or Matchmade Game)" -ForegroundColor White
Write-Host "3. Run this script again - you should see SUCCESS messages" -ForegroundColor White
Write-Host "4. If it works here, your Tauri app should now detect the game properly!" -ForegroundColor White
Write-Host ""
Write-Host "The Tauri app has been fixed to use the same APIs tested by this script." -ForegroundColor Green
