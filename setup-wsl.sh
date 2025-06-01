#!/bin/bash

# TribeChain WSL Ubuntu Setup Script
echo "=== TribeChain WSL Ubuntu Setup ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Update package list
print_status "Updating package list..."
sudo apt update

# Install essential build tools
print_status "Installing build essentials..."
sudo apt install -y build-essential curl wget git pkg-config libssl-dev

# Install Rust using rustup
if ! command -v cargo &> /dev/null; then
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the cargo environment
    source ~/.cargo/env
    
    # Add to bashrc for future sessions
    echo 'source ~/.cargo/env' >> ~/.bashrc
    
    print_status "Rust installed successfully!"
    cargo --version
else
    print_status "Rust is already installed."
    cargo --version
fi

# Install additional dependencies for TribeChain
print_status "Installing additional dependencies..."
sudo apt install -y \
    clang \
    llvm \
    librocksdb-dev \
    libclang-dev \
    protobuf-compiler

# Install Node.js and npm (for potential web interface)
if ! command -v node &> /dev/null; then
    print_status "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt install -y nodejs
fi

# Build TribeChain
print_status "Building TribeChain..."
if [ -f "Cargo.toml" ]; then
    # Set environment variables for cross-compilation if needed
    export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig"
    
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_status "TribeChain built successfully!"
        echo -e "${CYAN}Executable location: target/release/tribechain${NC}"
    else
        print_error "Build failed. Please check the error messages above."
        exit 1
    fi
else
    print_error "Cargo.toml not found. Make sure you're in the TribeChain directory."
    exit 1
fi

# Create run script
cat > run-tribechain.sh << 'EOF'
#!/bin/bash
echo "Starting TribeChain Node..."
./target/release/tribechain node --port 8333 --data-dir ./data
EOF

chmod +x run-tribechain.sh

# Create systemd service file (optional)
cat > tribechain.service << 'EOF'
[Unit]
Description=TribeChain Node
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/TribeChain
ExecStart=/home/ubuntu/TribeChain/target/release/tribechain node --port 8333 --data-dir /home/ubuntu/TribeChain/data
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

print_status "Setup complete!"
echo ""
echo -e "${CYAN}To start TribeChain:${NC}"
echo -e "  1. Run: ${GREEN}./run-tribechain.sh${NC}"
echo -e "  2. Or: ${GREEN}./target/release/tribechain --help${NC}"
echo ""
echo -e "${CYAN}To install as a system service:${NC}"
echo -e "  1. ${GREEN}sudo cp tribechain.service /etc/systemd/system/${NC}"
echo -e "  2. ${GREEN}sudo systemctl enable tribechain${NC}"
echo -e "  3. ${GREEN}sudo systemctl start tribechain${NC}"
echo ""
echo -e "${YELLOW}For ESP32 development, install PlatformIO:${NC}"
echo -e "  ${GREEN}pip install platformio${NC}" 