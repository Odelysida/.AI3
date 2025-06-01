# Download Wokwi CLI for Windows
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$url = "https://github.com/wokwi/wokwi-cli/releases/latest/download/wokwi-cli-windows-x64.exe"
$output = "wokwi-cli.exe"

Write-Host "Downloading Wokwi CLI from GitHub releases..."
Write-Host "URL: $url"
Write-Host "Output: $output"

try {
    Invoke-WebRequest -Uri $url -OutFile $output -UseBasicParsing
    Write-Host "✅ Wokwi CLI downloaded successfully!"
    
    # Make it executable and test
    if (Test-Path $output) {
        $fileSize = (Get-Item $output).Length
        Write-Host "File size: $fileSize bytes"
        
        # Test the executable
        Write-Host "Testing Wokwi CLI..."
        & ".\$output" --version
    }
} catch {
    Write-Host "❌ Error downloading Wokwi CLI: $($_.Exception.Message)"
    Write-Host "You can manually download from: https://github.com/wokwi/wokwi-cli/releases"
} 