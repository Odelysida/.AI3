#ifndef AI3_MINER_H
#define AI3_MINER_H

#include <Arduino.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include <freertos/queue.h>

// Maximum tensor size for ESP32 memory constraints
#define MAX_TENSOR_SIZE 256

// Tensor operation types
enum TensorOperationType {
    TENSOR_OP_MATRIX_MULT = 1,
    TENSOR_OP_CONVOLUTION = 2,
    TENSOR_OP_ACTIVATION = 3,
    TENSOR_OP_POOLING = 4,
    TENSOR_OP_NORMALIZATION = 5,
    TENSOR_OP_REDUCTION = 6,
    TENSOR_OP_ELEMENTWISE = 7,
    TENSOR_OP_CUSTOM = 255
};

// Tensor task structure for ESP32
struct TensorTask {
    char id[65];                    // Task ID (64 chars + null terminator)
    TensorOperationType operation;   // Operation type
    float inputData[MAX_TENSOR_SIZE]; // Input tensor data
    int inputSize;                  // Size of input data
    int dimensions[2];              // Tensor dimensions [height, width]
    uint32_t difficulty;            // Mining difficulty
    uint64_t reward;                // Reward for completing task
};

// Tensor computation result
struct TensorComputation {
    float outputData[MAX_TENSOR_SIZE]; // Output tensor data
    int outputSize;                    // Size of output data
    uint64_t executionTimeUs;          // Execution time in microseconds
    uint32_t memoryUsage;              // Memory usage in bytes
    uint64_t flops;                    // Floating point operations count
};

// Mining result structure
struct MiningResult {
    char taskId[65];                // Task ID
    char minerId[33];               // Miner ID (32 chars + null terminator)
    bool success;                   // Whether proof was found
    float optimizationFactor;       // Optimization factor achieved
    uint32_t iterations;            // Number of iterations performed
    uint64_t computationTimeUs;     // Total computation time
    uint8_t proofHash[32];          // SHA256 proof hash
};

class AI3Miner {
public:
    // Constructor
    AI3Miner(const char* minerId, const char* nodeUrl);
    
    // Destructor
    ~AI3Miner();
    
    // Initialize the miner
    bool begin();
    
    // Start mining
    void start();
    
    // Stop mining
    void stop();
    
    // Set mining difficulty
    void setDifficulty(uint32_t difficulty) { _difficulty = difficulty; }
    
    // Get current difficulty
    uint32_t getDifficulty() const { return _difficulty; }
    
    // Check if miner is running
    bool isRunning() const { return _isRunning; }

private:
    // Member variables
    const char* _minerId;
    const char* _nodeUrl;
    uint32_t _difficulty;
    bool _isRunning;
    
    // FreeRTOS tasks and queues
    TaskHandle_t _miningTask;
    TaskHandle_t _networkTask;
    QueueHandle_t _taskQueue;
    QueueHandle_t _resultQueue;
    
    // Task wrapper functions for FreeRTOS
    static void miningTaskWrapper(void* parameter);
    static void networkTaskWrapper(void* parameter);
    
    // Main task functions
    void miningTask();
    void networkTask();
    
    // Mining functions
    MiningResult mineTensorProof(const TensorTask& task);
    TensorComputation computeTensorOperation(const TensorTask& task);
    
    // Tensor operation implementations optimized for ESP32
    TensorComputation matrixMultiplyESP32(const TensorTask& task);
    TensorComputation convolutionESP32(const TensorTask& task);
    TensorComputation activationESP32(const TensorTask& task);
    TensorComputation elementwiseESP32(const TensorTask& task);
    TensorComputation defaultComputationESP32(const TensorTask& task);
    
    // Optimization and proof functions
    float calculateOptimizationFactor(const TensorComputation& comp);
    bool generateProof(const TensorTask& task, const TensorComputation& comp, 
                      float optimizationFactor, MiningResult& result);
    bool validateProof(const MiningResult& result, uint32_t difficulty);
    
    // Utility functions
    uint32_t calculateChecksum(const float* data, int size);
    
    // Network functions
    bool connectToNetwork();
    bool registerMiner();
    void requestNewTask();
    void submitProof(const MiningResult& result);
};

// Utility macros for ESP32 optimization
#define ESP32_YIELD() vTaskDelay(1)
#define ESP32_DISABLE_INTERRUPTS() taskDISABLE_INTERRUPTS()
#define ESP32_ENABLE_INTERRUPTS() taskENABLE_INTERRUPTS()

// Memory management macros
#define ESP32_MALLOC(size) heap_caps_malloc(size, MALLOC_CAP_8BIT)
#define ESP32_FREE(ptr) heap_caps_free(ptr)

// Performance monitoring macros
#define ESP32_GET_TIME_US() esp_timer_get_time()
#define ESP32_GET_FREE_HEAP() esp_get_free_heap_size()

#endif // AI3_MINER_H 