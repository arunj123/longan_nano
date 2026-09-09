# PowerShell script to reset hung USB hub, restart WSL service, and install usbipd-win
# Run this script in an Administrator PowerShell window.

Write-Host "=== Step 1: Unsticking Deadlocked USB Storage Stack ===" -ForegroundColor Cyan

# 1. Reset the USB Hub holding the stuck device
$hubId = "USB\VID_2109&PID_2813\5&2495b8dd&0&2"
Write-Host "Restarting USB Hub: $hubId..." -ForegroundColor Yellow
try {
    Disable-PnpDevice -InstanceId $hubId -Confirm:$false -ErrorAction Stop
    Start-Sleep -Seconds 3
    Enable-PnpDevice -InstanceId $hubId -Confirm:$false -ErrorAction Stop
    Write-Host "USB Hub successfully cycled. Hung IRP dropped!" -ForegroundColor Green
} catch {
    Write-Warning "Could not cycle hub via PnP ($($_.Exception.Message)). If this persists, physically unplug and reconnect the external USB hub."
}

# 2. Restart WSL Service
Write-Host "`n=== Step 2: Restarting WSL Service ===" -ForegroundColor Cyan
try {
    Restart-Service -Name WSLService -Force -ErrorAction Stop
    Write-Host "WSLService restarted cleanly." -ForegroundColor Green
} catch {
    Write-Warning "Could not restart WSLService ($($_.Exception.Message))."
}

# 3. Check / Install usbipd-win
Write-Host "`n=== Step 3: Checking usbipd-win ===" -ForegroundColor Cyan
$usbipd = Get-Command usbipd -ErrorAction SilentlyContinue
if (-not $usbipd) {
    Write-Host "Installing usbipd-win via winget..." -ForegroundColor Yellow
    winget install --id dorssel.usbipd-win --exact --accept-package-agreements --accept-source-agreements
} else {
    Write-Host "usbipd is already installed at: $($usbipd.Source)" -ForegroundColor Green
}

Write-Host "`n=== Done! Please open a new terminal and run: usbipd list ===" -ForegroundColor Green
