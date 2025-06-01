# Quick Start: Testing ESP32 TribeChain Mining with Wokwi

This guide will help you quickly test the ESP32 TribeChain mining implementation using the Wokwi simulator.

## 🚀 Quick Test (Automated)

Run the automated test suite:

```bash
# Make the test script executable (Linux/Mac)
chmod +x test_esp32_wokwi.py

# Run full test suite
python test_esp32_wokwi.py

# Or run individual components
python test_esp32_wokwi.py check      # Check prerequisites
python test_esp32_wokwi.py setup      # Setup Arduino environment
python test_esp32_wokwi.py compile    # Compile ESP32 firmware
python test_esp32_wokwi.py simulate   # Run Wokwi simulation
python test_esp32_wokwi.py demo       # Run Rust demo
```

## 📋 Prerequisites

Before testing, ensure you have:

1. **Wokwi CLI** - Install from [Wokwi CI Documentation](https://docs.wokwi.com/wokwi-ci/getting-started)
2. **Arduino CLI** - Install from [Arduino CLI](https://arduino.github.io/arduino-cli/)
3. **Rust** - Install from [rustup.rs](https://rustup.rs/)
4. **Python 3.7+** - For the test script

## 🔧 Manual Setup

If you prefer manual setup:

### 1. Install Arduino CLI and ESP32 Support

```bash
# Install ESP32 board package
arduino-cli core update-index
arduino-cli core install esp32:esp32

# Install required libraries
arduino-cli lib install ArduinoJson
arduino-cli lib install WiFi
```

### 2. Compile ESP32 Firmware

```bash
# Navigate to ESP32 directory
cd addon/esp32

# Compile the sketch
arduino-cli compile --fqbn esp32:esp32:esp32s3 --output-dir . tribechain_esp32.ino
```

### 3. Run Wokwi Simulation

```bash
# From project root
wokwi-cli simulate --timeout 60
```

## 🧪 Testing Scenarios

### Scenario 1: Basic Mining Test
- **Duration**: 30 seconds
- **Expected**: WiFi connection, mining tasks start, LED indicators active
- **Command**: `python test_esp32_wokwi.py simulate 30`

### Scenario 2: AI3 Tensor Computation Test
- **Duration**: 60 seconds
- **Expected**: Tensor tasks execution, AI3 proofs generation
- **Command**: `python test_esp32_wokwi.py simulate 60`

### Scenario 3: Rust Integration Test
- **Duration**: 30 seconds
- **Expected**: Rust demo runs, ESP32 config validated
- **Command**: `python test_esp32_wokwi.py demo`

## 📊 Expected Simulation Output

When running the simulation, you should see:

```
[00:00:01] TribeChain ESP32 Miner v1.0.0
[00:00:01] Device ID: ESP32-MINER-001
[00:00:02] Connecting to WiFi: Wokwi-GUEST
[00:00:05] WiFi connected! IP: 192.168.1.100
[00:00:06] Starting mining tasks on 2 cores
[00:00:07] AI3 tensor computation enabled
[00:00:08] Mining task started on Core 0
[00:00:08] Mining task started on Core 1
[00:00:10] [STATS] Blocks mined: 0, Hash rate: 1250 H/s
[00:00:15] [AI3] Tensor task received: MATRIX_MULTIPLY
[00:00:16] [AI3] Tensor computation completed
[00:00:20] [STATS] Blocks mined: 1, Hash rate: 1340 H/s
```

## 🔍 Visual Indicators

In the Wokwi simulator, monitor these visual elements:

- **Green LED (GPIO 2)**: Mining activity indicator
- **Blue LED (GPIO 4)**: AI3 computation indicator  
- **Red LED (GPIO 5)**: Error/warning indicator
- **LCD Display**: Shows current status and statistics
- **Temperature Sensor**: Monitors system temperature

## 🐛 Debugging

### Using VS Code Debugger

1. Open VS Code in the project directory
2. Set breakpoints in `addon/esp32/tribechain_esp32.ino`
3. Press F5 and select "Wokwi GDB - ESP32 TribeChain Miner"
4. The debugger will connect to the Wokwi simulator

### Common Issues

**Issue**: Compilation fails
- **Solution**: Ensure Arduino CLI and ESP32 core are installed
- **Check**: `arduino-cli core list | grep esp32`

**Issue**: Wokwi simulation doesn't start
- **Solution**: Verify `wokwi.toml` and `diagram.json` are correct
- **Check**: `wokwi-cli --version`

**Issue**: No mining activity
- **Solution**: Check WiFi connection in simulation
- **Debug**: Monitor serial output for connection status

## 📈 Performance Metrics

Expected performance on ESP32-S3:

- **Hash Rate**: 1000-1500 H/s per core
- **AI3 Tensor Ops**: 10-50 ops/second
- **Memory Usage**: ~200KB RAM
- **Power Consumption**: ~150mA @ 3.3V
- **Temperature**: 35-45°C under load

## 🔗 Next Steps

After successful testing:

1. **Deploy to Real Hardware**: Flash the compiled firmware to actual ESP32-S3
2. **Network Integration**: Connect to real TribeChain network
3. **Pool Mining**: Configure for mining pool participation
4. **Optimization**: Tune parameters for your specific use case

## 📚 Additional Resources

- [ESP32 Mining Guide](ESP32_MINING_GUIDE.md) - Comprehensive documentation
- [Wokwi Debugging](https://docs.wokwi.com/guides/debugging) - Advanced debugging techniques
- [TribeChain Documentation](README.md) - Main project documentation

## 🆘 Support

If you encounter issues:

1. Check the [troubleshooting section](ESP32_MINING_GUIDE.md#troubleshooting)
2. Review simulation logs for error messages
3. Verify all prerequisites are installed correctly
4. Open an issue on the project repository

---

**Happy Mining! ⛏️🚀** 