# WIP TribeChain - AI-Powered Blockchain with Tensor Mining 

TribeChain is an innovative blockchain platform developed by BitTribe that combines traditional blockchain technology with AI-powered tensor operations. It features a unique mining algorithm optimized for small CPUs like ESP32S and ESP8266, making it accessible for IoT devices and edge computing.

## Features

### 🔗 Core Blockchain
- **Proof-of-Work consensus** with adaptive difficulty
- **Fast block times** (target: 10 minutes)
- **Persistent storage** using RocksDB
- **Transaction validation** and balance management
- **Genesis block** with initial token distribution

### 🧠 AI3 Tensor Mining
- **Tensor operation mining** - miners solve AI computation tasks
- **Multiple operation types**: matrix multiplication, convolution, activation functions
- **Distributed computing** - tasks distributed across the network
- **Reward system** for completed tensor operations
- **ESP32/ESP8266 compatibility** for IoT mining

### 🪙 Multi-Token System
- **TRIBE** - Native blockchain token
- **STOMP** - Specialized token for staking and governance
- **AUM** - Utility token for AI operations
- **Token creation** and management
- **Staking and liquidity pools**

### 🌐 P2P Network
- **Async TCP networking** with Tokio
- **Peer discovery** and management
- **Block and transaction propagation**
- **Mining task distribution**
- **Network synchronization**

### 💼 Smart Contracts
- **Token operations** (create, transfer, stake)
- **Tensor computation contracts**
- **Staking and reward distribution**
- **Liquidity pool management**

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │   Network P2P   │    │   Storage       │
│   Layer         │    │   Layer         │    │   Layer         │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • CLI Interface │    │ • Peer Discovery│    │ • RocksDB       │
│ • Wallet Ops    │    │ • Message Relay │    │ • Persistence   │
│ • Mining        │    │ • Sync Protocol │    │ • Backup        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
┌─────────────────────────────────┼─────────────────────────────────┐
│                    Core Blockchain Engine                        │
├─────────────────┬─────────────────┬─────────────────┬─────────────┤
│   Blocks &      │   AI3 Tensor    │   Token         │   Smart     │
│   Transactions  │   Engine        │   System        │   Contracts │
├─────────────────┼─────────────────┼─────────────────┼─────────────┤
│ • Block Mining  │ • Matrix Ops    │ • TRIBE/STOMP   │ • Staking   │
│ • Validation    │ • Convolution   │ • AUM Tokens    │ • Liquidity │
│ • Merkle Trees  │ • Activations   │ • Token Mgmt    │ • Rewards   │
└─────────────────┴─────────────────┴─────────────────┴─────────────┘
```

## Installation

### Prerequisites
- Rust 1.70+ 
- Git

### Build from Source
```bash
git clone https://github.com/BitTribe/TribeChain.git
cd TribeChain
cargo build --release
```

### Install Binary
```bash
cargo install --path .
```

## Usage

### Start a Node
```bash
# Start a node on default port 8333
tribechain node

# Start with custom port and data directory
tribechain node --port 8334 --data-dir ./my-data

# Connect to existing peers
tribechain node --connect 192.168.1.100:8333 --connect 192.168.1.101:8333
```

### Wallet Operations
```bash
# Check balance
tribechain wallet balance alice

# Send tokens
tribechain wallet send alice bob 10.5
```

### Mining
```bash
# Start mining
tribechain mine miner_address_123

# Mine with custom data directory
tribechain mine miner_address_123 --data-dir ./my-data
```

### Token Operations
```bash
# Create a new token
tribechain token create "My Token" "MTK" 1000000 creator_address

# Create STOMP token
tribechain token create "STOMP Token" "STOMP" 500000 creator_address
```

### AI3 Tensor Operations
```bash
# Submit tensor computation task
tribechain ai3 compute "matrix_multiply" "1.0,2.0,3.0,4.0" requester_address

# Submit convolution task
tribechain ai3 compute "convolution" "0.1,0.2,0.3,0.4,0.5" requester_address
```

### Blockchain Statistics
```bash
# View blockchain stats
tribechain stats

# Output example:
# === TribeChain Statistics ===
# Blocks: 1250
# Transactions: 5420
# Pending Transactions: 12
# Difficulty: 6
# Mining Reward: 50.0 TRIBE
# Total Supply: 1000000.0 TRIBE
# Active Addresses: 156
# Average Block Time: 598 seconds
# Active Miners: 8
# Tensor Tasks: 23
```

## ESP32/ESP8266 Mining

TribeChain includes optimized mining code for ESP32 and ESP8266 microcontrollers:

### ESP32 Setup
```cpp
#include "ai3_miner.h"

AI3Miner miner("your_miner_id", "192.168.1.100", 8333);

void setup() {
    Serial.begin(115200);
    WiFi.begin("your_wifi", "password");
    
    while (WiFi.status() != WL_CONNECTED) {
        delay(1000);
    }
    
    miner.initialize();
}

void loop() {
    miner.mine_step();
    delay(100);
}
```

### Supported Operations
- **Matrix Multiplication** - Optimized for small matrices
- **Convolution** - 1D and 2D convolutions
- **Activation Functions** - ReLU, Sigmoid, Tanh
- **Vector Operations** - Dot product, normalization

## Token Economics

### TRIBE Token
- **Total Supply**: 1,000,000 TRIBE
- **Decimals**: 6
- **Mining Reward**: 50 TRIBE per block
- **Use Cases**: Transaction fees, staking, governance

### STOMP Token
- **Purpose**: Staking and governance
- **Features**: Enhanced staking rewards, voting rights
- **Utility**: Network security, decision making

### AUM Token
- **Purpose**: AI operations and tensor computing
- **Features**: Computation fees, AI model rewards
- **Utility**: Access to AI3 engine, premium features

## Network Protocol

### Message Types
- **Handshake**: Node identification and capabilities
- **Block Propagation**: New block announcements
- **Transaction Relay**: Pending transaction sharing
- **Mining Tasks**: Tensor operation distribution
- **Sync Requests**: Blockchain synchronization

### Peer Discovery
- **Bootstrap Nodes**: Initial peer connection
- **Peer Exchange**: Automatic peer discovery
- **Health Monitoring**: Connection quality tracking

## Development

### Project Structure (deprecated)
```
src/
├── lib.rs          # Main library exports
├── main.rs         # CLI application
├── block.rs        # Block structure and mining
├── transaction.rs  # Transaction types and validation
├── ai3.rs          # AI3 tensor engine
├── tokens.rs       # Token system and smart contracts
├── storage.rs      # RocksDB persistence layer
└── network.rs      # P2P networking

esp32/
├── ai3_miner.cpp   # ESP32 mining implementation
└── ai3_miner.h     # Header file
```

### Running Tests
```bash
cargo test
```

### Benchmarks
```bash
cargo bench
```

### Contributing
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

### Phase 1 (Current)
- ✅ Core blockchain implementation
- ✅ AI3 tensor mining
- ✅ Multi-token system
- 🔄 P2P networking
- 🔄 ESP32/ESP8266 support

### Phase 2 (Q2 2026)
- 🔄 Web interface and explorer
- 🔄 Mobile wallet application
- 🔄 Advanced smart contracts
- 🔄 Cross-chain bridges

### Phase 3 (Q3 2026)
- 📋 Machine learning model marketplace
- 📋 Decentralized AI training
- 📋 IoT device integration
- 📋 Enterprise partnerships

### Phase 4 (Q4 2026)
- 📋 Mainnet launch
- 📋 Exchange listings
- 📋 Ecosystem expansion
- 📋 Global adoption

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

- **Website**: https://bittribe.org
- **Email**: contact@bittribe.org
- **Discord**: https://discord.gg/bittribe
- **Twitter**: @BitTribeOrg

## Acknowledgments

- Rust community for excellent tooling
- ESP32/ESP8266 developers for IoT inspiration
- Blockchain pioneers for foundational concepts
- AI/ML community for tensor operation insights

---

**TribeChain** - Democratizing AI through blockchain technology 🚀 