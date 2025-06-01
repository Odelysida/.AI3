#include <WiFi.h>
#include <ArduinoJson.h>

// TribeChain ESP32 Miner Configuration (Simplified for Wokwi)
#define DEVICE_ID "ESP32_WOKWI_001"
#define WIFI_SSID "Wokwi-GUEST"
#define WIFI_PASSWORD ""

// Hardware pins
#define LED_MINING 2    // Red LED - Mining status
#define LED_NETWORK 4   // Green LED - Network status

// Global variables
bool isConnected = false;
bool isMining = false;
uint64_t blocksMined = 0;
uint64_t startTime = 0;

void setup() {
    Serial.begin(115200);
    delay(1000);
    
    Serial.println("=================================");
    Serial.println("🚀 TribeChain ESP32 Miner v1.0");
    Serial.println("   (Wokwi Simulator Version)");
    Serial.println("=================================");
    Serial.printf("Device ID: %s\n", DEVICE_ID);
    Serial.println();
    
    // Initialize hardware
    pinMode(LED_MINING, OUTPUT);
    pinMode(LED_NETWORK, OUTPUT);
    digitalWrite(LED_MINING, LOW);
    digitalWrite(LED_NETWORK, LOW);
    
    // Initialize WiFi
    initWiFi();
    
    // Start mining simulation
    startMining();
    
    startTime = millis();
    Serial.println("✅ ESP32 Miner initialized successfully!");
    Serial.println("⛏️  Starting mining simulation...");
}

void loop() {
    // Update LEDs based on status
    digitalWrite(LED_NETWORK, isConnected ? HIGH : LOW);
    digitalWrite(LED_MINING, isMining ? HIGH : LOW);
    
    // Simulate mining activity
    if (isMining) {
        simulateMining();
    }
    
    // Print stats every 5 seconds
    static unsigned long lastStatsUpdate = 0;
    if (millis() - lastStatsUpdate > 5000) {
        printStats();
        lastStatsUpdate = millis();
    }
    
    delay(1000);
}

void initWiFi() {
    Serial.println("📶 Connecting to WiFi...");
    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    
    int attempts = 0;
    while (WiFi.status() != WL_CONNECTED && attempts < 10) {
        delay(500);
        Serial.print(".");
        attempts++;
    }
    
    if (WiFi.status() == WL_CONNECTED) {
        isConnected = true;
        Serial.println();
        Serial.println("✅ WiFi connected!");
        Serial.printf("IP Address: %s\n", WiFi.localIP().toString().c_str());
    } else {
        Serial.println();
        Serial.println("❌ WiFi connection failed (Simulation Mode)");
        isConnected = false;
    }
}

void startMining() {
    isMining = true;
    Serial.println("⛏️  Mining started!");
}

void simulateMining() {
    // Simulate finding a block every 10-30 seconds
    static unsigned long lastBlock = 0;
    unsigned long blockInterval = random(10000, 30000); // 10-30 seconds
    
    if (millis() - lastBlock > blockInterval) {
        blocksMined++;
        lastBlock = millis();
        
        // Generate fake block data
        String blockHash = generateFakeHash();
        uint32_t nonce = random(1000000);
        
        Serial.println("🎉 BLOCK FOUND!");
        Serial.printf("   Block #%llu\n", blocksMined);
        Serial.printf("   Hash: %s\n", blockHash.c_str());
        Serial.printf("   Nonce: %u\n", nonce);
        Serial.printf("   Reward: 50 AI3\n");
        Serial.println();
        
        // Blink mining LED rapidly to celebrate
        for (int i = 0; i < 6; i++) {
            digitalWrite(LED_MINING, HIGH);
            delay(100);
            digitalWrite(LED_MINING, LOW);
            delay(100);
        }
    }
}

String generateFakeHash() {
    String hash = "0x";
    for (int i = 0; i < 8; i++) {
        hash += String(random(0, 16), HEX);
    }
    return hash;
}

void printStats() {
    unsigned long uptime = (millis() - startTime) / 1000;
    float hashRate = blocksMined > 0 ? (float)blocksMined / (uptime / 60.0) : 0.0;
    
    Serial.println("📊 MINING STATS");
    Serial.printf("   Uptime: %lu seconds\n", uptime);
    Serial.printf("   Blocks Mined: %llu\n", blocksMined);
    Serial.printf("   Hash Rate: %.2f blocks/min\n", hashRate);
    Serial.printf("   Network: %s\n", isConnected ? "Connected" : "Disconnected");
    Serial.printf("   Mining: %s\n", isMining ? "Active" : "Inactive");
    Serial.printf("   Free Memory: %d bytes\n", ESP.getFreeHeap());
    Serial.println("   Status: ✅ Running");
    Serial.println();
} 