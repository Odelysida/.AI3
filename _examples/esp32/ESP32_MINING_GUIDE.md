# ESP32 Mining with TribeChain

This guide explains how to set up and use ESP32 devices for mining on the TribeChain network with AI3 tensor computation capabilities.

## Overview

The ESP32 integration allows you to:
- Mine TribeChain blocks using ESP32 microcontrollers
- Perform AI3 tensor computations for enhanced mining rewards
- Monitor power consumption and system health
- Connect to the TribeChain network via WiFi
- Manage multiple ESP32 miners in a distributed setup

## Features

### 🔧 Hardware Management
- **Power Monitoring**: Real-time power consumption tracking with configurable limits
- **Temperature Monitoring**: Automatic thermal management to prevent overheating
- **Memory Management**: Efficient memory usage for blockchain data storage
- **WiFi Connectivity**: Robust network connection with automatic reconnection

### ⛏️ Mining Capabilities
- **Traditional PoW**: Standard proof-of-work mining with configurable difficulty
- **AI3 Enhanced Mining**: Tensor computation mining with reduced difficulty
- **Multi-threading**: Configurable number of mining threads for optimal performance
- **Automatic Sync**: Periodic blockchain synchronization with the network

### 🧠 AI3 Integration
- **Tensor Operations**: Matrix multiplication, convolution, neural network forward pass
- **Proof Generation**: Cryptographic proofs of tensor computation
- **Task Management**: Automatic assignment and completion of tensor tasks
- **Reward System**: Enhanced mining rewards for AI3 computations

## Quick Start

### 1. Basic ESP32 Mining

```bash
# Start ESP32 mining with basic configuration
cargo run -- esp32 mine \
  --device-id "ESP32_001" \
  --wifi-ssid "YourWiFiNetwork" \
  --wifi-password "YourPassword" \
  --threads 2
```

### 2. AI3 Enhanced Mining

```bash
# Start ESP32 mining with AI3 tensor computation
cargo run -- esp32 mine \
  --device-id "ESP32_AI3_001" \
  --wifi-ssid "YourWiFiNetwork" \
  --wifi-password "YourPassword" \
  --threads 2 \
  --ai3 \
  --power-limit 3.0
```

### 3. Run the Demo

```bash
# Run the comprehensive ESP32 mining demonstration
cargo run --example esp32_mining_demo
```

## Configuration Options

### ESP32Config Structure

```rust
pub struct ESP32Config {
    pub device_id: String,        // Unique identifier for the ESP32
    pub wifi_ssid: String,        // WiFi network name
    pub wifi_password: String,    // WiFi network password
    pub node_url: String,         // TribeChain node URL
    pub mining_threads: u8,       // Number of mining threads (1-4)
    pub ai3_enabled: bool,        // Enable AI3 tensor mining
    pub power_limit: f32,         // Power consumption limit in watts
}
```

### Command Line Arguments

| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--device-id` | `-d` | ESP32 device identifier | Required |
| `--wifi-ssid` | `-s` | WiFi network SSID | Required |
| `--wifi-password` | `-w` | WiFi network password | Required |
| `--node-url` | `-n` | TribeChain node URL | `http://localhost:8333` |
| `--threads` | `-t` | Number of mining threads | `2` |
| `--ai3` | | Enable AI3 tensor mining | `false` |
| `--power-limit` | `-p` | Power limit in watts | `3.0` |

## Mining Process

### 1. Initialization
```
📱 ESP32 Configuration loaded
🔧 WiFi connection established
🔗 TribeChain node connection
⚙️ Mining threads started
```

### 2. Mining Loop
```
🔄 Check network connectivity
📊 Sync blockchain state
⛏️ Mine new block
🧠 Process AI3 tensor tasks (if enabled)
📡 Submit mined block to network
🌡️ Monitor system health
```

### 3. AI3 Tensor Operations

The ESP32 can perform various tensor operations:

- **Matrix Multiplication**: Square matrix operations
- **Convolution**: 1D convolution with smoothing kernels
- **Neural Forward Pass**: Simple neural network inference
- **Tensor Addition/Multiplication**: Element-wise operations

## Performance Optimization

### Power Management
- **Dynamic Scaling**: Automatically reduce mining intensity when power limit is exceeded
- **Thermal Throttling**: Slow down operations when temperature is too high
- **Sleep Modes**: Efficient power usage during idle periods

### Memory Optimization
- **Lightweight Blockchain**: Store only essential blockchain data
- **Transaction Batching**: Process transactions in optimal batch sizes
- **Garbage Collection**: Regular cleanup of unused data

### Network Efficiency
- **Periodic Sync**: Sync blockchain every 60 seconds
- **Compression**: Efficient data transmission protocols
- **Error Recovery**: Automatic reconnection on network failures

## Monitoring and Statistics

### ESP32Stats Structure

```rust
pub struct ESP32Stats {
    pub device_id: String,
    pub uptime: u64,              // Seconds since startup
    pub blocks_mined: u64,        // Total blocks mined
    pub hash_rate: f64,           // Current hash rate
    pub power_consumption: f32,   // Current power usage (watts)
    pub temperature: f32,         // Device temperature (°C)
    pub memory_usage: u32,        // Memory usage (KB)
    pub wifi_signal: i8,          // WiFi signal strength (dBm)
    pub last_block_time: u64,     // Timestamp of last mined block
    pub ai3_tasks_completed: u64, // AI3 tasks completed
}
```

### Real-time Monitoring

```bash
# Check ESP32 mining statistics
cargo run -- esp32 stats --device-id "ESP32_001"
```

## Hardware Requirements

### Minimum Requirements
- **ESP32 Development Board** (ESP32-S3 recommended)
- **4MB Flash Memory** (for blockchain data storage)
- **520KB RAM** (for mining operations)
- **WiFi Connectivity** (2.4GHz)

### Recommended Setup
- **ESP32-S3** with dual-core processor
- **8MB Flash Memory** for extended blockchain storage
- **External antenna** for better WiFi signal
- **Heat sink** for thermal management
- **Stable power supply** (3.3V, 2A)

## Wokwi Simulation

The project includes Wokwi configuration for ESP32 simulation:

### Files
- `wokwi.toml`: Wokwi project configuration
- `diagram.json`: ESP32-S3 board setup
- Target firmware: `target/xtensa-esp32s3-none-elf/release/tribechain.bin`

### Running Simulation
1. Build the project for ESP32: `cargo build --release --target xtensa-esp32s3-none-elf`
2. Open the project in Wokwi
3. Start the simulation

## Troubleshooting

### Common Issues

#### WiFi Connection Problems
```
❌ WiFi connection failed
✅ Solution: Check SSID and password, ensure 2.4GHz network
```

#### Power Limit Exceeded
```
⚠️ Power limit exceeded, reducing mining intensity
✅ Solution: Increase power limit or reduce mining threads
```

#### Memory Issues
```
❌ Out of memory error
✅ Solution: Reduce blockchain cache size or restart device
```

#### Mining Difficulty Too High
```
⏸️ No blocks mined (difficulty too high)
✅ Solution: Enable AI3 mining for difficulty reduction
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug cargo run -- esp32 mine [options]
```

## Advanced Configuration

### Custom Tensor Operations

You can extend the AI3 capabilities by implementing custom tensor operations:

```rust
impl AI3Miner {
    fn custom_tensor_operation(&self, input: &[f32]) -> TribeResult<Vec<f32>> {
        // Your custom tensor computation here
        Ok(input.iter().map(|x| x * 2.0).collect())
    }
}
```

### Mining Pool Integration

Connect multiple ESP32 devices to a mining pool:

```rust
let mut mining_pool = MiningPool::new();
mining_pool.add_miner(esp32_miner.miner.clone());
```

### Custom Network Protocols

Implement custom networking for specialized deployments:

```rust
impl ESP32Miner {
    async fn custom_network_sync(&mut self) -> TribeResult<()> {
        // Custom synchronization logic
        Ok(())
    }
}
```

## Security Considerations

### Network Security
- Use WPA2/WPA3 encrypted WiFi networks
- Implement TLS for node communication
- Regular security updates

### Device Security
- Secure boot configuration
- Encrypted flash storage
- Hardware security modules (HSM) support

### Mining Security
- Proof verification
- Anti-tampering measures
- Secure key storage

## Contributing

To contribute to ESP32 mining development:

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests and documentation
5. Submit a pull request

### Development Setup

```bash
# Install ESP32 toolchain
rustup target add xtensa-esp32s3-none-elf

# Install additional tools
cargo install espflash
cargo install cargo-espflash

# Build for ESP32
cargo build --target xtensa-esp32s3-none-elf
```

## License

This ESP32 mining integration is part of the TribeChain project and is licensed under the same terms.

## Support

For ESP32 mining support:
- 📧 Email: esp32-support@tribechain.org
- 💬 Discord: TribeChain Community
- 📖 Documentation: https://docs.tribechain.org/esp32
- 🐛 Issues: GitHub Issues

---

**Happy Mining with ESP32! ⛏️🚀** 