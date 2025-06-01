#include <WiFi.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>
#include <mbedtls/sha256.h>
#include <esp_system.h>
#include <esp_timer.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include <freertos/queue.h>

// TribeChain ESP32 Miner Configuration
#define DEVICE_ID "ESP32_WOKWI_001"
#define WIFI_SSID "Wokwi-GUEST"
#define WIFI_PASSWORD ""
#define NODE_URL "http://localhost:8333"
#define MINING_THREADS 2
#define AI3_ENABLED true
#define POWER_LIMIT 3.0

// Hardware pins
#define LED_PIN 2
#define TEMP_SENSOR_PIN A0

// Mining constants
#define MAX_TENSOR_SIZE 64
#define MAX_ITERATIONS 10000
#define DIFFICULTY_TARGET 4

// Tensor operation types
enum TensorOperationType {
    TENSOR_OP_MATRIX_MULT = 1,
    TENSOR_OP_CONVOLUTION = 2,
    TENSOR_OP_NEURAL_FORWARD = 3,
    TENSOR_OP_TENSOR_ADD = 4,
    TENSOR_OP_TENSOR_MULTIPLY = 5
};

// Structures
struct TensorTask {
    char id[33];
    TensorOperationType operation;
    float inputData[MAX_TENSOR_SIZE];
    int inputSize;
    uint32_t difficulty;
    uint64_t reward;
    uint64_t maxComputeTime;
};

struct MiningResult {
    char taskId[33];
    char blockHash[65];
    uint64_t nonce;
    float optimizationFactor;
    uint64_t computationTime;
    bool success;
    uint8_t proofHash[32];
};

struct ESP32Stats {
    uint64_t uptime;
    uint64_t blocksMined;
    float hashRate;
    float powerConsumption;
    float temperature;
    uint32_t memoryUsage;
    int8_t wifiSignal;
    uint64_t ai3TasksCompleted;
};

// Global variables
bool isConnected = false;
bool isMining = false;
uint64_t startTime = 0;
uint64_t blocksMined = 0;
uint64_t ai3TasksCompleted = 0;
float currentHashRate = 0.0;

// FreeRTOS handles
TaskHandle_t miningTaskHandle = NULL;
TaskHandle_t networkTaskHandle = NULL;
TaskHandle_t statsTaskHandle = NULL;
QueueHandle_t taskQueue = NULL;
QueueHandle_t resultQueue = NULL;

void setup() {
    Serial.begin(115200);
    delay(1000);
    
    Serial.println("=================================");
    Serial.println("🚀 TribeChain ESP32 Miner v1.0");
    Serial.println("=================================");
    Serial.printf("Device ID: %s\n", DEVICE_ID);
    Serial.printf("AI3 Enabled: %s\n", AI3_ENABLED ? "Yes" : "No");
    Serial.printf("Mining Threads: %d\n", MINING_THREADS);
    Serial.printf("Power Limit: %.1f W\n", POWER_LIMIT);
    Serial.println();
    
    // Initialize hardware
    pinMode(LED_PIN, OUTPUT);
    digitalWrite(LED_PIN, LOW);
    
    // Initialize queues
    taskQueue = xQueueCreate(10, sizeof(TensorTask));
    resultQueue = xQueueCreate(10, sizeof(MiningResult));
    
    if (!taskQueue || !resultQueue) {
        Serial.println("❌ Failed to create queues");
        return;
    }
    
    // Initialize WiFi
    initWiFi();
    
    // Start mining tasks
    startMiningTasks();
    
    startTime = esp_timer_get_time();
    Serial.println("✅ ESP32 Miner initialized successfully!");
    Serial.println("⛏️  Starting mining operations...");
}

void loop() {
    // Main loop - monitor system health
    static unsigned long lastStatsUpdate = 0;
    
    if (millis() - lastStatsUpdate > 10000) { // Update every 10 seconds
        printStats();
        lastStatsUpdate = millis();
    }
    
    // Blink LED to show activity
    static bool ledState = false;
    digitalWrite(LED_PIN, ledState ? HIGH : LOW);
    ledState = !ledState;
    
    delay(1000);
}

void initWiFi() {
    Serial.println("📶 Connecting to WiFi...");
    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    
    int attempts = 0;
    while (WiFi.status() != WL_CONNECTED && attempts < 20) {
        delay(500);
        Serial.print(".");
        attempts++;
    }
    
    if (WiFi.status() == WL_CONNECTED) {
        isConnected = true;
        Serial.println();
        Serial.println("✅ WiFi connected!");
        Serial.printf("IP Address: %s\n", WiFi.localIP().toString().c_str());
        Serial.printf("Signal Strength: %d dBm\n", WiFi.RSSI());
    } else {
        Serial.println();
        Serial.println("❌ WiFi connection failed");
        isConnected = false;
    }
}

void startMiningTasks() {
    // Create mining task
    xTaskCreatePinnedToCore(
        miningTask,
        "Mining",
        8192,
        NULL,
        2,
        &miningTaskHandle,
        1  // Core 1
    );
    
    // Create network task
    xTaskCreatePinnedToCore(
        networkTask,
        "Network",
        4096,
        NULL,
        1,
        &networkTaskHandle,
        0  // Core 0
    );
    
    // Create stats task
    xTaskCreatePinnedToCore(
        statsTask,
        "Stats",
        2048,
        NULL,
        1,
        &statsTaskHandle,
        0  // Core 0
    );
    
    Serial.println("✅ Mining tasks started");
}

void miningTask(void* parameter) {
    TensorTask task;
    isMining = true;
    
    Serial.println("⛏️  Mining task started on Core 1");
    
    while (true) {
        // Generate demo tensor task
        generateDemoTask(task);
        
        // Mine the task
        MiningResult result = mineTensorTask(task);
        
        if (result.success) {
            blocksMined++;
            if (task.operation != TENSOR_OP_MATRIX_MULT) { // AI3 tasks
                ai3TasksCompleted++;
            }
            
            // Send result to network task
            xQueueSend(resultQueue, &result, portMAX_DELAY);
            
            Serial.printf("✅ Block mined! Hash: %.8s..., Nonce: %llu\n", 
                         result.blockHash, result.nonce);
            
            if (AI3_ENABLED && result.optimizationFactor > 1.0) {
                Serial.printf("🧠 AI3 Optimization: %.2fx\n", result.optimizationFactor);
            }
        }
        
        // Update hash rate
        updateHashRate();
        
        // Yield to other tasks
        vTaskDelay(pdMS_TO_TICKS(100));
    }
}

void networkTask(void* parameter) {
    MiningResult result;
    
    Serial.println("📡 Network task started on Core 0");
    
    while (true) {
        // Check WiFi connection
        if (WiFi.status() != WL_CONNECTED) {
            isConnected = false;
            Serial.println("📶 WiFi disconnected, attempting reconnection...");
            initWiFi();
        }
        
        // Process mining results
        if (xQueueReceive(resultQueue, &result, pdMS_TO_TICKS(100)) == pdTRUE) {
            if (isConnected) {
                submitBlock(result);
            } else {
                Serial.println("📡 Block ready but no network connection");
            }
        }
        
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
}

void statsTask(void* parameter) {
    while (true) {
        // Monitor system health
        float temperature = getTemperature();
        float power = getPowerConsumption();
        
        // Power management
        if (power > POWER_LIMIT) {
            Serial.printf("⚠️  Power limit exceeded: %.1fW > %.1fW\n", power, POWER_LIMIT);
            vTaskDelay(pdMS_TO_TICKS(5000)); // Throttle
        }
        
        // Temperature management
        if (temperature > 80.0) {
            Serial.printf("🌡️  High temperature: %.1f°C\n", temperature);
            vTaskDelay(pdMS_TO_TICKS(2000)); // Cool down
        }
        
        vTaskDelay(pdMS_TO_TICKS(5000));
    }
}

void generateDemoTask(TensorTask& task) {
    static uint32_t taskCounter = 0;
    taskCounter++;
    
    sprintf(task.id, "task_%08x", taskCounter);
    
    // Randomly select operation type
    int opType = (esp_random() % 5) + 1;
    task.operation = (TensorOperationType)opType;
    
    // Generate random input data
    task.inputSize = 16; // 4x4 matrix or 16-element vector
    for (int i = 0; i < task.inputSize; i++) {
        task.inputData[i] = (float)(esp_random() % 1000) / 100.0f; // 0-10.0
    }
    
    task.difficulty = DIFFICULTY_TARGET;
    task.reward = 100000; // 0.1 TRIBE
    task.maxComputeTime = 5000000; // 5 seconds in microseconds
}

MiningResult mineTensorTask(const TensorTask& task) {
    MiningResult result = {0};
    strcpy(result.taskId, task.id);
    
    uint64_t startTime = esp_timer_get_time();
    uint64_t nonce = 0;
    
    // Perform tensor computation
    float tensorResult[MAX_TENSOR_SIZE];
    int outputSize = computeTensorOperation(task, tensorResult);
    
    // Calculate optimization factor for AI3
    float optimizationFactor = 1.0f;
    if (AI3_ENABLED && task.operation != TENSOR_OP_MATRIX_MULT) {
        optimizationFactor = calculateOptimizationFactor(tensorResult, outputSize);
    }
    
    // Mine for valid hash
    char target[16];
    memset(target, '0', task.difficulty);
    target[task.difficulty] = '\0';
    
    while (nonce < MAX_ITERATIONS) {
        // Create block data
        char blockData[256];
        sprintf(blockData, "%s%llu%.4f", task.id, nonce, optimizationFactor);
        
        // Calculate hash
        uint8_t hash[32];
        mbedtls_sha256((const unsigned char*)blockData, strlen(blockData), hash, 0);
        
        // Convert to hex string
        char hashStr[65];
        for (int i = 0; i < 32; i++) {
            sprintf(hashStr + i * 2, "%02x", hash[i]);
        }
        
        // Check if hash meets difficulty
        if (strncmp(hashStr, target, task.difficulty) == 0) {
            result.success = true;
            result.nonce = nonce;
            result.optimizationFactor = optimizationFactor;
            result.computationTime = esp_timer_get_time() - startTime;
            strcpy(result.blockHash, hashStr);
            memcpy(result.proofHash, hash, 32);
            break;
        }
        
        nonce++;
        
        // Yield every 100 iterations
        if (nonce % 100 == 0) {
            vTaskDelay(1);
        }
    }
    
    return result;
}

int computeTensorOperation(const TensorTask& task, float* output) {
    int outputSize = task.inputSize;
    
    switch (task.operation) {
        case TENSOR_OP_MATRIX_MULT:
            return matrixMultiply(task.inputData, task.inputSize, output);
            
        case TENSOR_OP_CONVOLUTION:
            return convolution(task.inputData, task.inputSize, output);
            
        case TENSOR_OP_NEURAL_FORWARD:
            return neuralForward(task.inputData, task.inputSize, output);
            
        case TENSOR_OP_TENSOR_ADD:
            return tensorAdd(task.inputData, task.inputSize, output);
            
        case TENSOR_OP_TENSOR_MULTIPLY:
            return tensorMultiply(task.inputData, task.inputSize, output);
            
        default:
            // Default: copy input to output
            memcpy(output, task.inputData, task.inputSize * sizeof(float));
            return task.inputSize;
    }
}

int matrixMultiply(const float* input, int size, float* output) {
    int dim = (int)sqrt(size);
    if (dim * dim != size) return size; // Not a square matrix
    
    for (int i = 0; i < dim; i++) {
        for (int j = 0; j < dim; j++) {
            float sum = 0.0f;
            for (int k = 0; k < dim; k++) {
                sum += input[i * dim + k] * input[k * dim + j];
            }
            output[i * dim + j] = sum;
        }
    }
    return size;
}

int convolution(const float* input, int size, float* output) {
    // Simple 1D convolution with smoothing kernel
    float kernel[] = {0.1f, 0.8f, 0.1f};
    int kernelSize = 3;
    
    for (int i = 1; i < size - 1; i++) {
        float sum = 0.0f;
        for (int k = 0; k < kernelSize; k++) {
            sum += input[i - 1 + k] * kernel[k];
        }
        output[i - 1] = sum;
    }
    return size - 2;
}

int neuralForward(const float* input, int size, float* output) {
    // Simple neural network with ReLU activation
    float weights[] = {0.5f, -0.3f, 0.8f, 0.2f};
    int weightCount = 4;
    
    for (int i = 0; i < size; i++) {
        float weighted = input[i] * weights[i % weightCount];
        output[i] = fmaxf(0.0f, weighted); // ReLU activation
    }
    return size;
}

int tensorAdd(const float* input, int size, float* output) {
    float constant = 1.0f;
    for (int i = 0; i < size; i++) {
        output[i] = input[i] + constant;
    }
    return size;
}

int tensorMultiply(const float* input, int size, float* output) {
    float constant = 2.0f;
    for (int i = 0; i < size; i++) {
        output[i] = input[i] * constant;
    }
    return size;
}

float calculateOptimizationFactor(const float* result, int size) {
    // Calculate optimization based on computation complexity
    float sum = 0.0f;
    for (int i = 0; i < size; i++) {
        sum += fabsf(result[i]);
    }
    
    float avgMagnitude = sum / size;
    return 1.0f + (avgMagnitude / 10.0f); // Scale factor
}

void submitBlock(const MiningResult& result) {
    if (!isConnected) return;
    
    Serial.printf("📡 Submitting block %s to network...\n", result.taskId);
    
    // In a real implementation, this would send HTTP POST to the node
    // For simulation, we'll just log the submission
    Serial.printf("   Block Hash: %s\n", result.blockHash);
    Serial.printf("   Nonce: %llu\n", result.nonce);
    Serial.printf("   Computation Time: %llu μs\n", result.computationTime);
    
    if (result.optimizationFactor > 1.0f) {
        Serial.printf("   AI3 Optimization: %.2fx\n", result.optimizationFactor);
    }
    
    Serial.println("✅ Block submitted successfully!");
}

void updateHashRate() {
    static uint64_t lastUpdate = 0;
    static uint64_t lastBlocks = 0;
    
    uint64_t now = esp_timer_get_time();
    if (now - lastUpdate > 10000000) { // Update every 10 seconds
        uint64_t timeDiff = now - lastUpdate;
        uint64_t blockDiff = blocksMined - lastBlocks;
        
        currentHashRate = (float)blockDiff / (timeDiff / 1000000.0f); // blocks per second
        
        lastUpdate = now;
        lastBlocks = blocksMined;
    }
}

float getTemperature() {
    // Simulate temperature based on mining activity
    float baseTemp = 25.0f;
    float loadTemp = isMining ? (MINING_THREADS * 10.0f) : 0.0f;
    float randomVariation = (esp_random() % 100) / 100.0f - 0.5f;
    
    return baseTemp + loadTemp + randomVariation;
}

float getPowerConsumption() {
    // Simulate power consumption
    float basePower = 0.5f; // Base ESP32 consumption
    float miningPower = isMining ? (MINING_THREADS * 0.4f) : 0.0f;
    float ai3Power = AI3_ENABLED ? 0.3f : 0.0f;
    
    return basePower + miningPower + ai3Power;
}

uint32_t getMemoryUsage() {
    return ESP.getFreeHeap();
}

void printStats() {
    uint64_t uptime = (esp_timer_get_time() - startTime) / 1000000; // seconds
    
    Serial.println("\n📊 === ESP32 Mining Statistics ===");
    Serial.printf("Device ID: %s\n", DEVICE_ID);
    Serial.printf("Uptime: %llu seconds\n", uptime);
    Serial.printf("Blocks Mined: %llu\n", blocksMined);
    Serial.printf("AI3 Tasks: %llu\n", ai3TasksCompleted);
    Serial.printf("Hash Rate: %.2f blocks/sec\n", currentHashRate);
    Serial.printf("Temperature: %.1f°C\n", getTemperature());
    Serial.printf("Power: %.1fW\n", getPowerConsumption());
    Serial.printf("Free Memory: %u bytes\n", getMemoryUsage());
    
    if (isConnected) {
        Serial.printf("WiFi Signal: %d dBm\n", WiFi.RSSI());
        Serial.printf("IP Address: %s\n", WiFi.localIP().toString().c_str());
    } else {
        Serial.println("WiFi: Disconnected");
    }
    Serial.println("=====================================\n");
} 