#!/usr/bin/env python3
"""
Simple TribeChain ESP32 Test Script
Basic compatibility test for ESP32 mining implementation.
"""

import os
import sys

def check_files():
    """Check if all required files exist."""
    print("🔍 Checking ESP32 TribeChain files...")
    
    required_files = [
        "wokwi.toml",
        "diagram.json", 
        "addon/esp32/tribechain_esp32.ino",
        "addon/esp32/ai3_miner.h",
        "addon/esp32/ai3_miner.cpp",
        "examples/esp32_mining_demo.rs",
        "src/esp32_miner.rs",
        "ESP32_MINING_GUIDE.md",
        "QUICK_START_TESTING.md"
    ]
    
    all_exist = True
    for file_path in required_files:
        if os.path.exists(file_path):
            print(f"✅ {file_path}")
        else:
            print(f"❌ Missing: {file_path}")
            all_exist = False
    
    return all_exist

def check_wokwi_config():
    """Check Wokwi configuration."""
    print("\n🔧 Checking Wokwi configuration...")
    
    try:
        with open("wokwi.toml", "r", encoding='utf-8') as f:
            content = f.read()
            if "tribechain_esp32.ino.bin" in content:
                print("✅ Wokwi firmware path configured")
            else:
                print("❌ Wokwi firmware path not configured")
                return False
                
        with open("diagram.json", "r", encoding='utf-8') as f:
            content = f.read()
            if "esp32-s3-devkitc-1" in content:
                print("✅ ESP32-S3 board configured")
            else:
                print("❌ ESP32-S3 board not configured")
                return False
                
        return True
    except Exception as e:
        print(f"❌ Error reading config: {e}")
        return False

def check_arduino_sketch():
    """Check Arduino sketch."""
    print("\n📄 Checking Arduino sketch...")
    
    try:
        with open("addon/esp32/tribechain_esp32.ino", "r", encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
            checks = [
                ("WiFi.h", "WiFi library"),
                ("ArduinoJson.h", "JSON library"),
                ("TribeChain ESP32 Miner", "TribeChain header"),
                ("miningTask", "Mining task function"),
                ("networkTask", "Network task function"),
                ("statsTask", "Statistics task function"),
                ("setup()", "Setup function"),
                ("loop()", "Loop function")
            ]
            
            all_good = True
            for check, desc in checks:
                if check in content:
                    print(f"✅ {desc} found")
                else:
                    print(f"❌ {desc} missing")
                    all_good = False
                    
        return all_good
    except Exception as e:
        print(f"❌ Error reading sketch: {e}")
        return False

def check_rust_integration():
    """Check Rust integration."""
    print("\n🦀 Checking Rust integration...")
    
    try:
        # Check Cargo.toml
        with open("Cargo.toml", "r", encoding='utf-8') as f:
            content = f.read()
            if "esp32_mining_demo" in content:
                print("✅ ESP32 demo example configured")
            else:
                print("❌ ESP32 demo example not configured")
                return False
        
        # Check lib.rs
        with open("src/lib.rs", "r", encoding='utf-8') as f:
            content = f.read()
            if "esp32_miner" in content:
                print("✅ ESP32 miner module exported")
            else:
                print("❌ ESP32 miner module not exported")
                return False
                
        # Check main.rs
        with open("src/main.rs", "r", encoding='utf-8') as f:
            content = f.read()
            if "esp32" in content and "ESP32Config" in content:
                print("✅ ESP32 CLI integration found")
            else:
                print("❌ ESP32 CLI integration missing")
                return False
                
        return True
    except Exception as e:
        print(f"❌ Error checking Rust files: {e}")
        return False

def show_next_steps():
    """Show next steps for testing."""
    print("\n📋 Next Steps for Testing:")
    print("=" * 50)
    print("1. Install Prerequisites:")
    print("   - Wokwi CLI: https://docs.wokwi.com/wokwi-ci/getting-started")
    print("   - Arduino CLI: https://arduino.github.io/arduino-cli/")
    print("   - Rust: https://rustup.rs/")
    print("")
    print("2. Setup Arduino Environment:")
    print("   arduino-cli core update-index")
    print("   arduino-cli core install esp32:esp32")
    print("   arduino-cli lib install ArduinoJson")
    print("")
    print("3. Compile ESP32 Firmware:")
    print("   cd addon/esp32")
    print("   arduino-cli compile --fqbn esp32:esp32:esp32s3 tribechain_esp32.ino")
    print("")
    print("4. Run Wokwi Simulation:")
    print("   wokwi-cli simulate --timeout 60")
    print("")
    print("5. Test Rust Integration:")
    print("   cargo build --example esp32_mining_demo")
    print("   cargo run --example esp32_mining_demo")
    print("")
    print("6. Debug with VS Code:")
    print("   - Open project in VS Code")
    print("   - Press F5 and select 'Wokwi GDB - ESP32 TribeChain Miner'")
    print("")
    print("📚 Documentation:")
    print("   - ESP32_MINING_GUIDE.md - Complete guide")
    print("   - QUICK_START_TESTING.md - Quick start instructions")

def main():
    """Main function."""
    print("🧪 TribeChain ESP32 Simple Test")
    print("=" * 50)
    
    # Check all components
    files_ok = check_files()
    config_ok = check_wokwi_config()
    sketch_ok = check_arduino_sketch()
    rust_ok = check_rust_integration()
    
    print("\n" + "=" * 50)
    print("📊 Test Results:")
    print(f"   Files: {'✅ PASS' if files_ok else '❌ FAIL'}")
    print(f"   Config: {'✅ PASS' if config_ok else '❌ FAIL'}")
    print(f"   Sketch: {'✅ PASS' if sketch_ok else '❌ FAIL'}")
    print(f"   Rust: {'✅ PASS' if rust_ok else '❌ FAIL'}")
    
    if all([files_ok, config_ok, sketch_ok, rust_ok]):
        print("\n🎉 All checks passed! Ready for ESP32 testing.")
    else:
        print("\n⚠️  Some checks failed. Please review the issues above.")
    
    show_next_steps()

if __name__ == "__main__":
    main() 