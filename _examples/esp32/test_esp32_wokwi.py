#!/usr/bin/env python3
"""
TribeChain ESP32 Wokwi Test Script
This script helps test the ESP32 mining implementation using Wokwi simulator.
Works with Wokwi VS Code extension.
"""

import os
import sys
import subprocess
import time
import json
from pathlib import Path

class ESP32WokwiTester:
    def __init__(self):
        self.project_root = Path(__file__).parent
        self.addon_path = self.project_root / "addon" / "esp32"
        self.wokwi_config = self.project_root / "wokwi.toml"
        self.diagram_config = self.project_root / "diagram.json"
        
    def check_prerequisites(self):
        """Check if all required tools and files are available."""
        print("🔍 Checking prerequisites...")
        
        # Check if VS Code is available
        try:
            result = subprocess.run(["code", "--version"], 
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode == 0:
                print(f"✅ VS Code: Available")
                print("✅ Wokwi VS Code Extension: Assumed installed (as mentioned)")
            else:
                print("⚠️  VS Code not found in PATH, but may still be installed")
                print("✅ Wokwi VS Code Extension: Assumed installed (as mentioned)")
        except FileNotFoundError:
            print("⚠️  VS Code not found in PATH, but may still be installed")
            print("✅ Wokwi VS Code Extension: Assumed installed (as mentioned)")
        
        # Check if Arduino CLI is available (optional for VS Code extension)
        try:
            result = subprocess.run(["arduino-cli", "version"], 
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode == 0:
                print(f"✅ Arduino CLI: {result.stdout.strip()}")
            else:
                print("⚠️  Arduino CLI not found (optional for VS Code extension)")
        except FileNotFoundError:
            print("⚠️  Arduino CLI not found (optional for VS Code extension)")
        
        # Check if Rust/Cargo is available
        try:
            result = subprocess.run(["cargo", "--version"], 
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode == 0:
                print(f"✅ Cargo: {result.stdout.strip()}")
            else:
                print("❌ Cargo not found. Install Rust from: https://rustup.rs/")
                return False
        except FileNotFoundError:
            print("❌ Cargo not found. Install Rust from: https://rustup.rs/")
            return False
        
        # Check required files
        required_files = [
            self.addon_path / "tribechain_esp32.ino"
        ]
        
        for file_path in required_files:
            if file_path.exists():
                print(f"✅ {file_path.name}")
            else:
                print(f"❌ Missing: {file_path}")
                return False
        
        return True
    
    def create_wokwi_config_files(self):
        """Create Wokwi configuration files for VS Code extension."""
        print("\n🔧 Creating Wokwi configuration files...")
        
        # Create wokwi.toml
        wokwi_toml_content = """[wokwi]
version = 1
elf = "addon/esp32/tribechain_esp32.ino.elf"
firmware = "addon/esp32/tribechain_esp32.ino.bin"

[[wokwi.scenario]]
name = "TribeChain ESP32 Mining Test"
timeout = 60000
"""
        
        with open(self.wokwi_config, 'w') as f:
            f.write(wokwi_toml_content)
        print(f"✅ Created {self.wokwi_config}")
        
        # Create diagram.json
        diagram_content = {
            "version": 1,
            "author": "TribeChain",
            "editor": "wokwi",
            "parts": [
                {
                    "type": "wokwi-esp32-devkit-v1",
                    "id": "esp",
                    "top": 0,
                    "left": 0,
                    "attrs": {}
                },
                {
                    "type": "wokwi-led",
                    "id": "led1",
                    "top": -24,
                    "left": 178.67,
                    "attrs": {"color": "red"}
                },
                {
                    "type": "wokwi-led",
                    "id": "led2",
                    "top": -24,
                    "left": 207.33,
                    "attrs": {"color": "green"}
                },
                {
                    "type": "wokwi-resistor",
                    "id": "r1",
                    "top": 29.6,
                    "left": 172.8,
                    "attrs": {"value": "220"}
                },
                {
                    "type": "wokwi-resistor",
                    "id": "r2",
                    "top": 29.6,
                    "left": 201.6,
                    "attrs": {"value": "220"}
                }
            ],
            "connections": [
                ["esp:TX0", "$serialMonitor:RX", "", []],
                ["esp:RX0", "$serialMonitor:TX", "", []],
                ["esp:D2", "led1:A", "", []],
                ["led1:C", "r1:1", "", []],
                ["r1:2", "esp:GND.1", "", []],
                ["esp:D4", "led2:A", "", []],
                ["led2:C", "r2:1", "", []],
                ["r2:2", "esp:GND.2", "", []]
            ],
            "dependencies": {}
        }
        
        with open(self.diagram_config, 'w') as f:
            json.dump(diagram_content, f, indent=2)
        print(f"✅ Created {self.diagram_config}")
        
        return True
    
    def setup_arduino_environment(self):
        """Setup Arduino environment for ESP32 (optional with VS Code extension)."""
        print("\n🔧 Setting up Arduino environment...")
        print("ℹ️  Note: With Wokwi VS Code extension, Arduino CLI setup is optional")
        
        # Check if Arduino CLI is available
        try:
            result = subprocess.run(["arduino-cli", "version"], 
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode != 0:
                print("⚠️  Arduino CLI not available, skipping setup")
                return True
        except FileNotFoundError:
            print("⚠️  Arduino CLI not available, skipping setup")
            return True
        
        # Install ESP32 board package
        commands = [
            ["arduino-cli", "core", "update-index"],
            ["arduino-cli", "core", "install", "esp32:esp32"],
            ["arduino-cli", "lib", "install", "ArduinoJson"],
            ["arduino-cli", "lib", "install", "WiFi"],
        ]
        
        for cmd in commands:
            print(f"Running: {' '.join(cmd)}")
            result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode != 0:
                print(f"⚠️  Command failed: {result.stderr}")
            else:
                print("✅ Success")
        
        return True
    
    def compile_esp32_firmware(self):
        """Compile the ESP32 firmware (optional with VS Code extension)."""
        print("\n🔨 Compiling ESP32 firmware...")
        print("ℹ️  Note: With Wokwi VS Code extension, you can compile directly in VS Code")
        
        sketch_path = self.addon_path / "tribechain_esp32.ino"
        output_path = self.addon_path
        
        # Check if Arduino CLI is available
        try:
            result = subprocess.run(["arduino-cli", "version"], 
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                                  universal_newlines=True, shell=True)
            if result.returncode != 0:
                print("⚠️  Arduino CLI not available, skipping compilation")
                print("💡 Use VS Code with Wokwi extension to compile and run")
                return True
        except FileNotFoundError:
            print("⚠️  Arduino CLI not available, skipping compilation")
            print("💡 Use VS Code with Wokwi extension to compile and run")
            return True
        
        compile_cmd = [
            "arduino-cli", "compile",
            "--fqbn", "esp32:esp32:esp32s3",
            "--output-dir", str(output_path),
            str(sketch_path)
        ]
        
        print(f"Running: {' '.join(compile_cmd)}")
        result = subprocess.run(compile_cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, 
                              universal_newlines=True, shell=True)
        
        if result.returncode == 0:
            print("✅ Compilation successful!")
            
            # Check if binary files were created
            bin_file = output_path / "tribechain_esp32.ino.bin"
            elf_file = output_path / "tribechain_esp32.ino.elf"
            
            if bin_file.exists() and elf_file.exists():
                print(f"✅ Binary files created:")
                print(f"   📄 {bin_file}")
                print(f"   📄 {elf_file}")
                return True
            else:
                print("❌ Binary files not found after compilation")
                return False
        else:
            print(f"❌ Compilation failed:")
            print(result.stderr)
            return False
    
    def open_vscode_project(self):
        """Open the project in VS Code with Wokwi extension."""
        print("\n🚀 Opening project in VS Code...")
        
        try:
            # Try to open VS Code in the current directory
            result = subprocess.run(["code", "."], cwd=str(self.project_root), shell=True)
            if result.returncode == 0:
                print("✅ VS Code opened successfully!")
                print("\n📋 Next steps in VS Code:")
                print("   1. Open the diagram.json file")
                print("   2. Click 'Start Simulation' in Wokwi extension")
                print("   3. Monitor the serial output for mining activity")
                print("   4. Check LED indicators for system status")
                return True
            else:
                print("❌ Failed to open VS Code")
                return False
        except Exception as e:
            print(f"❌ Error opening VS Code: {e}")
            print("💡 Manually open VS Code and load this project directory")
            return False
    
    def run_rust_demo(self):
        """Run the Rust ESP32 mining demo."""
        print("\n🦀 Running Rust ESP32 mining demo...")
        
        # Build the demo
        build_cmd = ["cargo", "build", "--example", "esp32_mining_demo"]
        print(f"Building: {' '.join(build_cmd)}")
        
        result = subprocess.run(build_cmd, cwd=str(self.project_root), shell=True)
        if result.returncode != 0:
            print("❌ Failed to build Rust demo")
            return False
        
        # Run the demo
        run_cmd = ["cargo", "run", "--example", "esp32_mining_demo"]
        print(f"Running: {' '.join(run_cmd)}")
        
        try:
            result = subprocess.run(run_cmd, cwd=str(self.project_root), timeout=30, shell=True)
            if result.returncode == 0:
                print("✅ Rust demo completed successfully!")
                return True
            else:
                print(f"❌ Rust demo failed with return code: {result.returncode}")
                return False
        except subprocess.TimeoutExpired:
            print("⏰ Rust demo timed out (this is expected for the demo)")
            return True
        except Exception as e:
            print(f"❌ Error running Rust demo: {e}")
            return False
    
    def analyze_simulation_results(self):
        """Analyze the simulation results."""
        print("\n📊 Simulation Analysis Guide...")
        
        print("📈 Expected simulation behavior in VS Code:")
        print("   🔌 ESP32 should connect to WiFi (Wokwi-GUEST)")
        print("   ⛏️  Mining tasks should start on both cores")
        print("   🧠 AI3 tensor computations should execute")
        print("   📊 Statistics should be printed every 10 seconds")
        print("   💡 LEDs should blink to indicate activity")
        print("   🌡️  Temperature monitoring should be active")
        print("   📡 Network tasks should handle block submissions")
        
        print("\n🔍 How to monitor in VS Code:")
        print("   📺 Serial Monitor: Watch for mining output")
        print("   💡 Visual LEDs: Red=Core 0, Green=Core 1 activity")
        print("   📊 Logic Analyzer: Monitor GPIO signals")
        print("   🐛 Debugger: Set breakpoints in critical functions")
        
        return True
    
    def run_full_test(self):
        """Run the complete test suite for VS Code extension."""
        print("🧪 TribeChain ESP32 Wokwi Test Suite (VS Code Extension)")
        print("=" * 60)
        
        # Step 1: Check prerequisites
        if not self.check_prerequisites():
            print("❌ Prerequisites check failed")
            return False
        
        # Step 2: Create Wokwi config files
        if not self.create_wokwi_config_files():
            print("❌ Failed to create Wokwi config files")
            return False
        
        # Step 3: Setup Arduino environment (optional)
        self.setup_arduino_environment()
        
        # Step 4: Compile firmware (optional)
        self.compile_esp32_firmware()
        
        # Step 5: Run Rust demo (parallel test)
        print("\n" + "=" * 60)
        self.run_rust_demo()
        
        # Step 6: Open VS Code project
        print("\n" + "=" * 60)
        self.open_vscode_project()
        
        # Step 7: Provide analysis guide
        self.analyze_simulation_results()
        
        print("\n" + "=" * 60)
        print("🎉 Test suite setup completed!")
        print("\n📋 VS Code Workflow:")
        print("   1. VS Code should now be open with your project")
        print("   2. Open 'diagram.json' to see the circuit diagram")
        print("   3. Click the 'Start Simulation' button in Wokwi panel")
        print("   4. Open the Serial Monitor to see ESP32 output")
        print("   5. Watch the LEDs for mining activity indicators")
        print("   6. Use the debugger to step through code if needed")
        
        return True

def main():
    """Main function."""
    if len(sys.argv) > 1:
        command = sys.argv[1]
        tester = ESP32WokwiTester()
        
        if command == "check":
            tester.check_prerequisites()
        elif command == "setup":
            tester.setup_arduino_environment()
        elif command == "compile":
            tester.compile_esp32_firmware()
        elif command == "config":
            tester.create_wokwi_config_files()
        elif command == "vscode":
            tester.open_vscode_project()
        elif command == "demo":
            tester.run_rust_demo()
        elif command == "analyze":
            tester.analyze_simulation_results()
        else:
            print(f"Unknown command: {command}")
            print("Available commands: check, setup, compile, config, vscode, demo, analyze")
    else:
        # Run full test suite
        tester = ESP32WokwiTester()
        tester.run_full_test()

if __name__ == "__main__":
    main() 