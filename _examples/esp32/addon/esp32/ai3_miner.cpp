#include "ai3_miner.h"
#include <WiFi.h>
#include <HTTPClient.h>
#include <ArduinoJson.h>
#include <mbedtls/sha256.h>
#include <esp_system.h>
#include <esp_timer.h>

AI3Miner::AI3Miner(const char* minerId, const char* nodeUrl) 
    : _minerId(minerId), _nodeUrl(nodeUrl), _difficulty(1000), _isRunning(false) {
    _taskQueue = xQueueCreate(10, sizeof(TensorTask));
    _resultQueue = xQueueCreate(10, sizeof(MiningResult));
}

AI3Miner::~AI3Miner() {
    stop();
    if (_taskQueue) vQueueDelete(_taskQueue);
    if (_resultQueue) vQueueDelete(_resultQueue);
}

bool AI3Miner::begin() {
    Serial.println("AI3 Miner: Initializing...");
    
    // Initialize WiFi connection
    if (!connectToNetwork()) {
        Serial.println("AI3 Miner: Failed to connect to network");
        return false;
    }
    
    // Register with TribeChain network
    if (!registerMiner()) {
        Serial.println("AI3 Miner: Failed to register with network");
        return false;
    }
    
    Serial.println("AI3 Miner: Initialization complete");
    return true;
}

void AI3Miner::start() {
    if (_isRunning) return;
    
    _isRunning = true;
    
    // Create mining task
    xTaskCreatePinnedToCore(
        miningTaskWrapper,
        "AI3_Mining",
        8192,  // Stack size optimized for ESP32
        this,
        1,     // Priority
        &_miningTask,
        1      // Core 1 for mining
    );
    
    // Create network task
    xTaskCreatePinnedToCore(
        networkTaskWrapper,
        "AI3_Network",
        4096,  // Smaller stack for network
        this,
        2,     // Higher priority
        &_networkTask,
        0      // Core 0 for network
    );
    
    Serial.println("AI3 Miner: Started mining tasks");
}

void AI3Miner::stop() {
    _isRunning = false;
    
    if (_miningTask) {
        vTaskDelete(_miningTask);
        _miningTask = nullptr;
    }
    
    if (_networkTask) {
        vTaskDelete(_networkTask);
        _networkTask = nullptr;
    }
    
    Serial.println("AI3 Miner: Stopped");
}

void AI3Miner::miningTaskWrapper(void* parameter) {
    AI3Miner* miner = static_cast<AI3Miner*>(parameter);
    miner->miningTask();
}

void AI3Miner::networkTaskWrapper(void* parameter) {
    AI3Miner* miner = static_cast<AI3Miner*>(parameter);
    miner->networkTask();
}

void AI3Miner::miningTask() {
    TensorTask task;
    
    while (_isRunning) {
        // Wait for new task from network
        if (xQueueReceive(_taskQueue, &task, pdMS_TO_TICKS(1000)) == pdTRUE) {
            Serial.printf("AI3 Miner: Processing task %s\n", task.id);
            
            MiningResult result = mineTensorProof(task);
            
            if (result.success) {
                Serial.printf("AI3 Miner: Found proof! Optimization factor: %.4f\n", 
                             result.optimizationFactor);
                
                // Send result to network task
                xQueueSend(_resultQueue, &result, portMAX_DELAY);
            }
        }
        
        // Yield to other tasks
        vTaskDelay(pdMS_TO_TICKS(10));
    }
}

void AI3Miner::networkTask() {
    MiningResult result;
    unsigned long lastTaskRequest = 0;
    const unsigned long TASK_REQUEST_INTERVAL = 30000; // 30 seconds
    
    while (_isRunning) {
        // Request new tasks periodically
        if (millis() - lastTaskRequest > TASK_REQUEST_INTERVAL) {
            requestNewTask();
            lastTaskRequest = millis();
        }
        
        // Submit completed proofs
        if (xQueueReceive(_resultQueue, &result, pdMS_TO_TICKS(100)) == pdTRUE) {
            submitProof(result);
        }
        
        vTaskDelay(pdMS_TO_TICKS(1000));
    }
}

MiningResult AI3Miner::mineTensorProof(const TensorTask& task) {
    MiningResult result = {0};
    strncpy(result.taskId, task.id, sizeof(result.taskId) - 1);
    strncpy(result.minerId, _minerId, sizeof(result.minerId) - 1);
    
    uint64_t startTime = esp_timer_get_time();
    uint32_t iterations = 0;
    const uint32_t MAX_ITERATIONS = 100000; // Reduced for ESP32
    
    while (iterations < MAX_ITERATIONS && _isRunning) {
        // Perform tensor computation
        TensorComputation comp = computeTensorOperation(task);
        
        // Calculate optimization factor
        float optimizationFactor = calculateOptimizationFactor(comp);
        
        // Generate proof
        if (generateProof(task, comp, optimizationFactor, result)) {
            // Check if proof meets difficulty
            if (validateProof(result, task.difficulty)) {
                result.success = true;
                result.optimizationFactor = optimizationFactor;
                result.iterations = iterations;
                result.computationTimeUs = esp_timer_get_time() - startTime;
                break;
            }
        }
        
        iterations++;
        
        // Yield every 100 iterations for ESP32 stability
        if (iterations % 100 == 0) {
            vTaskDelay(1);
        }
    }
    
    return result;
}

TensorComputation AI3Miner::computeTensorOperation(const TensorTask& task) {
    TensorComputation comp = {0};
    uint64_t startTime = esp_timer_get_time();
    
    switch (task.operation) {
        case TENSOR_OP_MATRIX_MULT:
            comp = matrixMultiplyESP32(task);
            break;
        case TENSOR_OP_CONVOLUTION:
            comp = convolutionESP32(task);
            break;
        case TENSOR_OP_ACTIVATION:
            comp = activationESP32(task);
            break;
        case TENSOR_OP_ELEMENTWISE:
            comp = elementwiseESP32(task);
            break;
        default:
            comp = defaultComputationESP32(task);
            break;
    }
    
    comp.executionTimeUs = esp_timer_get_time() - startTime;
    comp.memoryUsage = task.inputSize * sizeof(float) + comp.outputSize * sizeof(float);
    
    return comp;
}

TensorComputation AI3Miner::matrixMultiplyESP32(const TensorTask& task) {
    TensorComputation comp = {0};
    
    // Optimized matrix multiplication for ESP32
    // Using fixed-point arithmetic when possible
    const int rows = task.dimensions[0];
    const int cols = task.dimensions[1];
    
    comp.outputSize = rows * cols;
    
    // Allocate output buffer
    float* output = (float*)malloc(comp.outputSize * sizeof(float));
    if (!output) {
        comp.flops = 0;
        return comp;
    }
    
    // Perform matrix multiplication with ESP32 optimizations
    for (int i = 0; i < rows; i++) {
        for (int j = 0; j < cols; j++) {
            float sum = 0.0f;
            for (int k = 0; k < cols; k++) {
                sum += task.inputData[i * cols + k] * task.inputData[k * cols + j];
            }
            output[i * cols + j] = sum;
        }
    }
    
    // Copy result to computation structure
    memcpy(comp.outputData, output, min(comp.outputSize * sizeof(float), sizeof(comp.outputData)));
    comp.flops = rows * cols * cols * 2; // Multiply-add operations
    
    free(output);
    return comp;
}

TensorComputation AI3Miner::convolutionESP32(const TensorTask& task) {
    TensorComputation comp = {0};
    
    // Simple 3x3 convolution optimized for ESP32
    const float kernel[9] = {1.0f, 0.0f, -1.0f, 2.0f, 0.0f, -2.0f, 1.0f, 0.0f, -1.0f};
    const int height = task.dimensions[0];
    const int width = task.dimensions[1];
    
    comp.outputSize = (height - 2) * (width - 2);
    
    int outputIdx = 0;
    for (int y = 1; y < height - 1; y++) {
        for (int x = 1; x < width - 1; x++) {
            float sum = 0.0f;
            for (int ky = 0; ky < 3; ky++) {
                for (int kx = 0; kx < 3; kx++) {
                    int py = y + ky - 1;
                    int px = x + kx - 1;
                    sum += task.inputData[py * width + px] * kernel[ky * 3 + kx];
                }
            }
            if (outputIdx < MAX_TENSOR_SIZE) {
                comp.outputData[outputIdx++] = sum;
            }
        }
    }
    
    comp.flops = comp.outputSize * 9 * 2; // 9 multiply-add operations per output
    return comp;
}

TensorComputation AI3Miner::activationESP32(const TensorTask& task) {
    TensorComputation comp = {0};
    comp.outputSize = task.inputSize;
    
    // ReLU activation optimized for ESP32
    for (int i = 0; i < task.inputSize && i < MAX_TENSOR_SIZE; i++) {
        comp.outputData[i] = (task.inputData[i] > 0.0f) ? task.inputData[i] : 0.0f;
    }
    
    comp.flops = task.inputSize; // One comparison per element
    return comp;
}

TensorComputation AI3Miner::elementwiseESP32(const TensorTask& task) {
    TensorComputation comp = {0};
    comp.outputSize = task.inputSize;
    
    // Element-wise square operation
    for (int i = 0; i < task.inputSize && i < MAX_TENSOR_SIZE; i++) {
        comp.outputData[i] = task.inputData[i] * task.inputData[i];
    }
    
    comp.flops = task.inputSize; // One multiplication per element
    return comp;
}

TensorComputation AI3Miner::defaultComputationESP32(const TensorTask& task) {
    TensorComputation comp = {0};
    comp.outputSize = task.inputSize;
    
    // Simple default operation: multiply by 2
    for (int i = 0; i < task.inputSize && i < MAX_TENSOR_SIZE; i++) {
        comp.outputData[i] = task.inputData[i] * 2.0f;
    }
    
    comp.flops = task.inputSize;
    return comp;
}

float AI3Miner::calculateOptimizationFactor(const TensorComputation& comp) {
    // Calculate optimization metrics
    float timeFactor = 1000000.0f / (comp.executionTimeUs + 1.0f);
    float memoryFactor = 1000000.0f / (comp.memoryUsage + 1.0f);
    float flopsFactor = comp.flops / (comp.executionTimeUs + 1.0f);
    
    // Energy efficiency estimation for ESP32
    float energyFactor = 1000.0f / (comp.executionTimeUs / 1000.0f + 1.0f);
    
    return sqrt(timeFactor * memoryFactor * flopsFactor * energyFactor) / 1000.0f;
}

bool AI3Miner::generateProof(const TensorTask& task, const TensorComputation& comp, 
                            float optimizationFactor, MiningResult& result) {
    // Generate computation proof
    uint8_t proofData[32];
    
    // Add operation type
    proofData[0] = task.operation;
    
    // Add optimization factor
    memcpy(&proofData[1], &optimizationFactor, sizeof(float));
    
    // Add input/output checksums
    uint32_t inputChecksum = calculateChecksum(task.inputData, task.inputSize);
    uint32_t outputChecksum = calculateChecksum(comp.outputData, comp.outputSize);
    
    memcpy(&proofData[5], &inputChecksum, sizeof(uint32_t));
    memcpy(&proofData[9], &outputChecksum, sizeof(uint32_t));
    
    // Add timestamp
    uint64_t timestamp = esp_timer_get_time();
    memcpy(&proofData[13], &timestamp, sizeof(uint64_t));
    
    // Generate proof hash
    mbedtls_sha256_context ctx;
    mbedtls_sha256_init(&ctx);
    mbedtls_sha256_starts(&ctx, 0);
    mbedtls_sha256_update(&ctx, proofData, sizeof(proofData));
    mbedtls_sha256_update(&ctx, (uint8_t*)_minerId, strlen(_minerId));
    mbedtls_sha256_finish(&ctx, result.proofHash);
    mbedtls_sha256_free(&ctx);
    
    return true;
}

bool AI3Miner::validateProof(const MiningResult& result, uint32_t difficulty) {
    // Check if proof hash meets difficulty requirement
    uint32_t targetZeros = difficulty / 4;
    
    for (uint32_t i = 0; i < targetZeros && i < 32; i++) {
        if (result.proofHash[i] != 0) {
            return false;
        }
    }
    
    return true;
}

uint32_t AI3Miner::calculateChecksum(const float* data, int size) {
    uint32_t checksum = 0;
    for (int i = 0; i < size; i++) {
        checksum = checksum ^ ((uint32_t)(data[i] * 1000.0f));
    }
    return checksum;
}

bool AI3Miner::connectToNetwork() {
    // WiFi connection logic would go here
    // For now, assume connection is successful
    return true;
}

bool AI3Miner::registerMiner() {
    if (!WiFi.isConnected()) return false;
    
    HTTPClient http;
    http.begin(String(_nodeUrl) + "/api/miners/register");
    http.addHeader("Content-Type", "application/json");
    
    DynamicJsonDocument doc(1024);
    doc["minerId"] = _minerId;
    doc["deviceType"] = "ESP32";
    doc["capabilities"] = "tensor_operations";
    
    String payload;
    serializeJson(doc, payload);
    
    int httpCode = http.POST(payload);
    bool success = (httpCode == 200);
    
    http.end();
    return success;
}

void AI3Miner::requestNewTask() {
    if (!WiFi.isConnected()) return;
    
    HTTPClient http;
    http.begin(String(_nodeUrl) + "/api/mining/task");
    http.addHeader("X-Miner-ID", _minerId);
    
    int httpCode = http.GET();
    
    if (httpCode == 200) {
        String response = http.getString();
        DynamicJsonDocument doc(2048);
        
        if (deserializeJson(doc, response) == DeserializationError::Ok) {
            TensorTask task = {0};
            
            strncpy(task.id, doc["id"], sizeof(task.id) - 1);
            task.operation = doc["operation"];
            task.difficulty = doc["difficulty"];
            task.reward = doc["reward"];
            task.dimensions[0] = doc["dimensions"][0];
            task.dimensions[1] = doc["dimensions"][1];
            
            JsonArray inputArray = doc["inputData"];
            task.inputSize = min((int)inputArray.size(), MAX_TENSOR_SIZE);
            for (int i = 0; i < task.inputSize; i++) {
                task.inputData[i] = inputArray[i];
            }
            
            // Add task to queue
            xQueueSend(_taskQueue, &task, 0);
        }
    }
    
    http.end();
}

void AI3Miner::submitProof(const MiningResult& result) {
    if (!WiFi.isConnected()) return;
    
    HTTPClient http;
    http.begin(String(_nodeUrl) + "/api/mining/submit");
    http.addHeader("Content-Type", "application/json");
    http.addHeader("X-Miner-ID", _minerId);
    
    DynamicJsonDocument doc(2048);
    doc["taskId"] = result.taskId;
    doc["minerId"] = result.minerId;
    doc["optimizationFactor"] = result.optimizationFactor;
    doc["iterations"] = result.iterations;
    doc["computationTimeUs"] = result.computationTimeUs;
    
    // Convert proof hash to hex string
    String proofHashHex = "";
    for (int i = 0; i < 32; i++) {
        if (result.proofHash[i] < 16) proofHashHex += "0";
        proofHashHex += String(result.proofHash[i], HEX);
    }
    doc["proofHash"] = proofHashHex;
    
    String payload;
    serializeJson(doc, payload);
    
    int httpCode = http.POST(payload);
    
    if (httpCode == 200) {
        Serial.printf("AI3 Miner: Proof submitted successfully for task %s\n", result.taskId);
    } else {
        Serial.printf("AI3 Miner: Failed to submit proof, HTTP code: %d\n", httpCode);
    }
    
    http.end();
} 