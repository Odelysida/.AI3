# TribeChain Setup Guide

TribeChain is an AI-powered blockchain with tensor mining capabilities. This guide will help you set up and run TribeChain on your system.

## Prerequisites

- Windows 10/11 with WSL2 (recommended) or Windows native
- At least 4GB RAM
- 10GB free disk space
- Internet connection

## Option 1: WSL Ubuntu Setup (Recommended)

WSL provides a better development environment for Rust and blockchain development.

### 1. Ensure WSL is Running
```powershell
# Start your WSL Ubuntu distribution
wsl -d Ubuntu
```

### 2. Copy Files to WSL
```bash
# In WSL, navigate to your home directory
cd ~

# Create TribeChain directory
mkdir -p TribeChain
cd TribeChain

# Copy files from Windows (adjust path as needed)
cp -r /mnt/t/coding/TribeChain/.AI3/* .
```

### 3. Run Setup Script
```bash
# Make setup script executable
chmod +x setup-wsl.sh

# Run the setup script
./setup-wsl.sh
```

### 4. Start TribeChain
```bash
# Start the node
./run-tribechain.sh

# Or run specific commands
./target/release/tribechain --help
./target/release/tribechain node --port 8333
```

## Option 2: Windows Native Setup

### 1. Run PowerShell as Administrator
Right-click PowerShell and select "Run as Administrator"

### 2. Enable Script Execution
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 3. Run Setup Script
```powershell
# Navigate to TribeChain directory
cd T:\coding\TribeChain\.AI3

# Run the setup script
.\setup-windows.ps1
```

### 4. Start TribeChain
```powershell
# Start the node
.\run-tribechain.bat

# Or run specific commands
.\target\release\tribechain.exe --help
.\target\release\tribechain.exe node --port 8333
```

## Manual Installation (If Scripts Fail)

### Install Rust
1. Visit https://rustup.rs/
2. Download and run the installer
3. Restart your terminal
4. Verify: `cargo --version`

### Build TribeChain
```bash
# Install dependencies (Ubuntu/WSL)
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Build the project
cargo build --release
```

## ESP32 Development

For ESP32 AI3 mining development:

### 1. Install PlatformIO
```bash
# Install Python and pip first
pip install platformio

# Or use VS Code extension
# Install "PlatformIO IDE" extension in VS Code
```

### 2. ESP32 Project Setup
```bash
cd esp32/
pio init --board esp32dev
pio lib install "ArduinoJson" "WiFi" "HTTPClient"
```

### 3. Upload to ESP32
```bash
# Connect ESP32 via USB
pio run --target upload
```

## Usage Examples

### Start a Node
```bash
./target/release/tribechain node --port 8333 --data-dir ./data
```

### Create a Wallet
```bash
./target/release/tribechain wallet create --name my-wallet
```

### Start Mining
```bash
./target/release/tribechain mine --address YOUR_ADDRESS --threads 4
```

### Check Stats
```bash
./target/release/tribechain stats
```

### AI3 Tensor Mining
```bash
./target/release/tribechain ai3 mine --difficulty 1000
```

## Network Configuration

### Default Ports
- Node P2P: 8333
- RPC API: 8334 (if enabled)
- Web Interface: 8080 (if enabled)

### Firewall Configuration
Make sure to open the required ports in your firewall:

```powershell
# Windows Firewall
New-NetFirewallRule -DisplayName "TribeChain Node" -Direction Inbound -Port 8333 -Protocol TCP -Action Allow
```

```bash
# Ubuntu UFW
sudo ufw allow 8333/tcp
```

## Troubleshooting

### Common Issues

1. **Rust not found**: Restart terminal after installation
2. **Build errors**: Install Visual Studio Build Tools (Windows) or build-essential (Linux)
3. **Permission denied**: Run as administrator (Windows) or use sudo (Linux)
4. **Port already in use**: Change port with `--port` flag

### Getting Help

1. Check logs in `./data/logs/`
2. Run with verbose output: `--verbose`
3. Check GitHub issues: https://github.com/BitTribe/TribeChain

## Development

### Project Structure
```
TribeChain/
├── src/           # Rust source code
├── esp32/         # ESP32 firmware
├── contracts/     # Smart contracts
├── docs/          # Documentation
└── target/        # Build output
```

### Building for Development
```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

TribeChain is released under the MIT License. See LICENSE file for details. 