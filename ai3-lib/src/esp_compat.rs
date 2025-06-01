use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape, TensorData};
use crate::mining::{AI3Miner, MiningTask, MiningResult};
use tribechain_core::{TribeResult, TribeError};

/// ESP mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESPMiningConfig {
    pub device_type: ESPDeviceType,
    pub max_memory_kb: usize,
    pub clock_speed_mhz: u32,
    pub wifi_ssid: String,
    pub server_address: String,
    pub server_port: u16,
    pub mining_intensity: u8, // 1-10 scale
    pub power_save_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ESPDeviceType {
    ESP32,
    ESP8266,
    ESP32S2,
    ESP32S3,
    ESP32C3,
}

impl ESPDeviceType {
    pub fn get_memory_limit(&self) -> usize {
        match self {
            ESPDeviceType::ESP32 => 320,      // 320KB RAM
            ESPDeviceType::ESP8266 => 80,     // 80KB RAM
            ESPDeviceType::ESP32S2 => 320,    // 320KB RAM
            ESPDeviceType::ESP32S3 => 512,    // 512KB RAM
            ESPDeviceType::ESP32C3 => 400,    // 400KB RAM
        }
    }

    pub fn get_compute_power(&self) -> u64 {
        match self {
            ESPDeviceType::ESP32 => 240,      // 240MHz dual core
            ESPDeviceType::ESP8266 => 80,     // 80MHz single core
            ESPDeviceType::ESP32S2 => 240,    // 240MHz single core
            ESPDeviceType::ESP32S3 => 240,    // 240MHz dual core
            ESPDeviceType::ESP32C3 => 160,    // 160MHz single core
        }
    }

    pub fn supports_floating_point(&self) -> bool {
        match self {
            ESPDeviceType::ESP32 => true,
            ESPDeviceType::ESP8266 => false,  // Limited FPU
            ESPDeviceType::ESP32S2 => true,
            ESPDeviceType::ESP32S3 => true,
            ESPDeviceType::ESP32C3 => true,
        }
    }
}

impl Default for ESPMiningConfig {
    fn default() -> Self {
        Self {
            device_type: ESPDeviceType::ESP32,
            max_memory_kb: 320,
            clock_speed_mhz: 240,
            wifi_ssid: "TribeChain_Mining".to_string(),
            server_address: "192.168.1.100".to_string(),
            server_port: 8333,
            mining_intensity: 5,
            power_save_mode: false,
        }
    }
}

/// ESP32 specific miner implementation
#[derive(Debug, Clone)]
pub struct ESP32Miner {
    pub base_miner: AI3Miner,
    pub config: ESPMiningConfig,
    pub connection_status: ConnectionStatus,
    pub performance_stats: ESPPerformanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESPPerformanceStats {
    pub uptime_seconds: u64,
    pub memory_usage_kb: usize,
    pub cpu_temperature: f32,
    pub wifi_signal_strength: i8, // dBm
    pub power_consumption_mw: u32,
    pub hash_rate: f64, // hashes per second
    pub successful_tasks: u64,
    pub failed_tasks: u64,
}

impl ESP32Miner {
    pub fn new(id: String, address: String, config: ESPMiningConfig) -> Self {
        let base_miner = AI3Miner::new(id, address, true);
        
        Self {
            base_miner,
            config,
            connection_status: ConnectionStatus::Disconnected,
            performance_stats: ESPPerformanceStats {
                uptime_seconds: 0,
                memory_usage_kb: 0,
                cpu_temperature: 25.0,
                wifi_signal_strength: -50,
                power_consumption_mw: 500,
                hash_rate: 0.0,
                successful_tasks: 0,
                failed_tasks: 0,
            },
        }
    }

    pub fn initialize(&mut self) -> TribeResult<()> {
        self.connection_status = ConnectionStatus::Connecting;
        
        // Simulate ESP32 initialization
        self.check_memory_constraints()?;
        self.configure_wifi()?;
        self.connect_to_server()?;
        
        self.connection_status = ConnectionStatus::Connected;
        Ok(())
    }

    fn check_memory_constraints(&self) -> TribeResult<()> {
        let available_memory = self.config.device_type.get_memory_limit();
        if self.config.max_memory_kb > available_memory {
            return Err(TribeError::InvalidOperation(
                format!("Requested memory {}KB exceeds device limit {}KB", 
                       self.config.max_memory_kb, available_memory)
            ));
        }
        Ok(())
    }

    fn configure_wifi(&mut self) -> TribeResult<()> {
        // Simulate WiFi configuration
        if self.config.wifi_ssid.is_empty() {
            return Err(TribeError::InvalidOperation("WiFi SSID cannot be empty".to_string()));
        }
        
        // Update signal strength (simulated)
        self.performance_stats.wifi_signal_strength = -45; // Good signal
        Ok(())
    }

    fn connect_to_server(&mut self) -> TribeResult<()> {
        // Simulate server connection
        if self.config.server_address.is_empty() {
            return Err(TribeError::InvalidOperation("Server address cannot be empty".to_string()));
        }
        
        // In real implementation, this would establish TCP connection
        Ok(())
    }

    pub fn mine_step(&mut self) -> TribeResult<Option<MiningResult>> {
        if !matches!(self.connection_status, ConnectionStatus::Connected) {
            return Err(TribeError::InvalidOperation("Not connected to server".to_string()));
        }

        // Update performance stats
        self.update_performance_stats();

        // Check if we need to throttle due to temperature or power
        if self.should_throttle() {
            return Ok(None);
        }

        // Delegate to base miner with ESP optimizations
        let result = self.base_miner.mine_step()?;
        
        if let Some(ref mining_result) = result {
            self.performance_stats.successful_tasks += 1;
            self.update_hash_rate(mining_result.computation_time);
        }

        Ok(result)
    }

    fn update_performance_stats(&mut self) {
        self.performance_stats.uptime_seconds += 1;
        
        // Simulate temperature based on mining intensity
        let base_temp = 25.0;
        let temp_increase = (self.config.mining_intensity as f32) * 2.0;
        self.performance_stats.cpu_temperature = base_temp + temp_increase;
        
        // Simulate memory usage
        let base_memory = 50; // Base system usage
        let mining_memory = (self.config.mining_intensity as usize) * 10;
        self.performance_stats.memory_usage_kb = base_memory + mining_memory;
        
        // Simulate power consumption
        let base_power = 200; // Base power consumption in mW
        let mining_power = (self.config.mining_intensity as u32) * 50;
        self.performance_stats.power_consumption_mw = base_power + mining_power;
    }

    fn should_throttle(&self) -> bool {
        // Throttle if temperature is too high
        if self.performance_stats.cpu_temperature > 80.0 {
            return true;
        }

        // Throttle if memory usage is too high
        let memory_limit = (self.config.max_memory_kb as f32 * 0.9) as usize; // 90% limit
        if self.performance_stats.memory_usage_kb > memory_limit {
            return true;
        }

        // Throttle if in power save mode and battery is low (simulated)
        if self.config.power_save_mode && self.performance_stats.power_consumption_mw > 800 {
            return true;
        }

        false
    }

    fn update_hash_rate(&mut self, computation_time_ms: u64) {
        if computation_time_ms > 0 {
            let hashes_per_computation = 1000.0; // Estimated hashes tried per computation
            let time_seconds = computation_time_ms as f64 / 1000.0;
            let current_rate = hashes_per_computation / time_seconds;
            
            // Exponential moving average
            let alpha = 0.1;
            self.performance_stats.hash_rate = 
                alpha * current_rate + (1.0 - alpha) * self.performance_stats.hash_rate;
        }
    }

    pub fn get_status_report(&self) -> ESPStatusReport {
        ESPStatusReport {
            device_id: self.base_miner.id.clone(),
            device_type: self.config.device_type.clone(),
            connection_status: self.connection_status.clone(),
            performance_stats: self.performance_stats.clone(),
            current_task_id: self.base_miner.current_task.as_ref().map(|t| t.id.clone()),
            miner_stats: self.base_miner.stats.clone(),
        }
    }

    pub fn optimize_for_task(&mut self, task: &MiningTask) -> TribeResult<()> {
        // Adjust mining intensity based on task complexity
        let operation = task.get_operation()?;
        let complexity = operation.get_complexity_score();
        
        // Scale intensity based on complexity and device capabilities
        let device_power = self.config.device_type.get_compute_power();
        let optimal_intensity = ((complexity * device_power) / 10000).min(10).max(1) as u8;
        
        self.config.mining_intensity = optimal_intensity;
        
        // Enable power save mode for complex tasks on low-power devices
        if complexity > 500 && device_power < 200 {
            self.config.power_save_mode = true;
        }
        
        Ok(())
    }
}

/// ESP8266 specific miner (simplified version of ESP32)
#[derive(Debug, Clone)]
pub struct ESP8266Miner {
    pub esp32_miner: ESP32Miner,
}

impl ESP8266Miner {
    pub fn new(id: String, address: String, config: ESPMiningConfig) -> Self {
        let mut esp8266_config = config;
        esp8266_config.device_type = ESPDeviceType::ESP8266;
        esp8266_config.max_memory_kb = esp8266_config.max_memory_kb.min(80); // ESP8266 limit
        
        Self {
            esp32_miner: ESP32Miner::new(id, address, esp8266_config),
        }
    }

    pub fn initialize(&mut self) -> TribeResult<()> {
        // ESP8266 has more limited capabilities
        if !self.esp32_miner.config.device_type.supports_floating_point() {
            // Convert floating point operations to fixed point
            self.convert_to_fixed_point()?;
        }
        
        self.esp32_miner.initialize()
    }

    fn convert_to_fixed_point(&mut self) -> TribeResult<()> {
        // In a real implementation, this would configure the miner
        // to use fixed-point arithmetic instead of floating-point
        // for better performance on ESP8266
        Ok(())
    }

    pub fn mine_step(&mut self) -> TribeResult<Option<MiningResult>> {
        // ESP8266 mining with additional constraints
        self.esp32_miner.mine_step()
    }
}

/// Status report for ESP devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESPStatusReport {
    pub device_id: String,
    pub device_type: ESPDeviceType,
    pub connection_status: ConnectionStatus,
    pub performance_stats: ESPPerformanceStats,
    pub current_task_id: Option<String>,
    pub miner_stats: crate::mining::MinerStats,
}

/// Utility functions for ESP tensor operations
pub struct ESPTensorUtils;

impl ESPTensorUtils {
    /// Convert tensor to ESP-optimized format
    pub fn optimize_tensor_for_esp(tensor: &Tensor, device_type: &ESPDeviceType) -> TribeResult<Tensor> {
        if !tensor.is_esp_compatible() {
            return Err(TribeError::InvalidOperation("Tensor too large for ESP device".to_string()));
        }

        // For ESP8266, convert to fixed-point if needed
        if !device_type.supports_floating_point() {
            return Self::convert_to_fixed_point(tensor);
        }

        Ok(tensor.clone())
    }

    /// Convert floating-point tensor to fixed-point representation
    pub fn convert_to_fixed_point(tensor: &Tensor) -> TribeResult<Tensor> {
        let data = tensor.data.as_f32_vec()?;
        
        // Convert to 16-bit fixed point (Q8.8 format)
        let fixed_data: Vec<f32> = data.iter()
            .map(|&x| {
                let fixed_val = (x * 256.0).round().clamp(-32768.0, 32767.0) as i16;
                (fixed_val as f32) / 256.0
            })
            .collect();

        Tensor::from_vec(fixed_data, tensor.shape.clone())
    }

    /// Estimate memory usage for tensor operation on ESP
    pub fn estimate_memory_usage(tensors: &[Tensor], operation: &str) -> usize {
        let mut total_size = 0;
        
        for tensor in tensors {
            total_size += tensor.shape.total_elements() * 4; // 4 bytes per f32
        }
        
        // Add operation-specific overhead
        let overhead = match operation {
            "matrix_multiply" => total_size, // Output tensor size
            "convolution" => total_size / 2, // Typically smaller output
            "relu" | "sigmoid" | "tanh" => 0, // In-place operations
            _ => total_size / 4, // Conservative estimate
        };
        
        total_size + overhead
    }

    /// Check if operation can run on ESP device
    pub fn can_run_on_esp(
        tensors: &[Tensor], 
        operation: &str, 
        device_type: &ESPDeviceType
    ) -> bool {
        let memory_needed = Self::estimate_memory_usage(tensors, operation);
        let memory_limit = device_type.get_memory_limit() * 1024; // Convert KB to bytes
        
        // Use only 70% of available memory for safety
        let safe_limit = (memory_limit as f32 * 0.7) as usize;
        
        memory_needed <= safe_limit
    }
}

/// Generate C++ code for ESP32/ESP8266
pub struct ESPCodeGenerator;

impl ESPCodeGenerator {
    pub fn generate_mining_code(config: &ESPMiningConfig) -> String {
        format!(r#"
#include <WiFi.h>
#include <WebSocketsClient.h>
#include "ai3_miner.h"

// Configuration
const char* ssid = "{}";
const char* password = "your_password";
const char* server_host = "{}";
const int server_port = {};
const int mining_intensity = {};

AI3Miner miner("esp32_miner", server_host, server_port);
WebSocketsClient webSocket;

void setup() {{
    Serial.begin(115200);
    
    // Initialize WiFi
    WiFi.begin(ssid, password);
    while (WiFi.status() != WL_CONNECTED) {{
        delay(1000);
        Serial.println("Connecting to WiFi...");
    }}
    Serial.println("WiFi connected!");
    
    // Initialize miner
    miner.initialize();
    miner.set_intensity(mining_intensity);
    
    // Setup WebSocket connection
    webSocket.begin(server_host, server_port, "/");
    webSocket.onEvent(webSocketEvent);
    webSocket.setReconnectInterval(5000);
}}

void loop() {{
    webSocket.loop();
    
    // Mining step
    if (miner.mine_step()) {{
        // Send result to server
        String result = miner.get_latest_result();
        webSocket.sendTXT(result);
    }}
    
    // Monitor performance
    if (millis() % 10000 == 0) {{ // Every 10 seconds
        print_performance_stats();
    }}
    
    delay(100);
}}

void webSocketEvent(WStype_t type, uint8_t * payload, size_t length) {{
    switch(type) {{
        case WStype_CONNECTED:
            Serial.println("WebSocket Connected");
            break;
        case WStype_TEXT:
            // Handle new mining task
            miner.handle_new_task((char*)payload);
            break;
        case WStype_DISCONNECTED:
            Serial.println("WebSocket Disconnected");
            break;
    }}
}}

void print_performance_stats() {{
    Serial.println("=== Performance Stats ===");
    Serial.printf("Uptime: %lu seconds\n", millis() / 1000);
    Serial.printf("Free heap: %d bytes\n", ESP.getFreeHeap());
    Serial.printf("CPU temperature: %.1f°C\n", temperatureRead());
    Serial.printf("WiFi signal: %d dBm\n", WiFi.RSSI());
    Serial.printf("Hash rate: %.2f H/s\n", miner.get_hash_rate());
    Serial.printf("Tasks completed: %d\n", miner.get_completed_tasks());
}}
"#, 
            config.wifi_ssid,
            config.server_address,
            config.server_port,
            config.mining_intensity
        )
    }

    pub fn generate_tensor_operations() -> String {
        r#"
// AI3 Tensor Operations for ESP32/ESP8266
#include "ai3_tensor.h"

class AI3TensorOps {
public:
    // Matrix multiplication optimized for ESP
    static bool matrix_multiply(const float* a, const float* b, float* result, 
                               int rows_a, int cols_a, int cols_b) {
        for (int i = 0; i < rows_a; i++) {
            for (int j = 0; j < cols_b; j++) {
                float sum = 0.0f;
                for (int k = 0; k < cols_a; k++) {
                    sum += a[i * cols_a + k] * b[k * cols_b + j];
                }
                result[i * cols_b + j] = sum;
            }
        }
        return true;
    }
    
    // ReLU activation function
    static void relu(float* data, int size) {
        for (int i = 0; i < size; i++) {
            if (data[i] < 0.0f) {
                data[i] = 0.0f;
            }
        }
    }
    
    // Sigmoid activation function
    static void sigmoid(float* data, int size) {
        for (int i = 0; i < size; i++) {
            data[i] = 1.0f / (1.0f + expf(-data[i]));
        }
    }
    
    // Vector dot product
    static float dot_product(const float* a, const float* b, int size) {
        float result = 0.0f;
        for (int i = 0; i < size; i++) {
            result += a[i] * b[i];
        }
        return result;
    }
    
    // 1D Convolution
    static bool convolution_1d(const float* input, const float* kernel, float* output,
                              int input_size, int kernel_size, int stride) {
        int output_size = (input_size - kernel_size) / stride + 1;
        
        for (int i = 0; i < output_size; i++) {
            float sum = 0.0f;
            for (int j = 0; j < kernel_size; j++) {
                sum += input[i * stride + j] * kernel[j];
            }
            output[i] = sum;
        }
        return true;
    }
};
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esp_device_capabilities() {
        let esp32 = ESPDeviceType::ESP32;
        let esp8266 = ESPDeviceType::ESP8266;

        assert!(esp32.get_memory_limit() > esp8266.get_memory_limit());
        assert!(esp32.get_compute_power() > esp8266.get_compute_power());
        assert!(esp32.supports_floating_point());
        assert!(!esp8266.supports_floating_point());
    }

    #[test]
    fn test_esp32_miner_creation() {
        let config = ESPMiningConfig::default();
        let miner = ESP32Miner::new("esp32_1".to_string(), "192.168.1.10".to_string(), config);

        assert_eq!(miner.base_miner.id, "esp32_1");
        assert!(miner.base_miner.capabilities.is_esp_device);
        assert!(matches!(miner.connection_status, ConnectionStatus::Disconnected));
    }

    #[test]
    fn test_tensor_esp_compatibility() {
        let small_tensor = Tensor::vector(vec![1.0; 100]);
        let large_tensor = Tensor::vector(vec![1.0; 2000]);

        assert!(small_tensor.is_esp_compatible());
        assert!(!large_tensor.is_esp_compatible());
    }

    #[test]
    fn test_memory_estimation() {
        let tensor = Tensor::vector(vec![1.0; 100]);
        let memory_usage = ESPTensorUtils::estimate_memory_usage(&[tensor], "relu");
        
        // 100 elements * 4 bytes = 400 bytes
        assert_eq!(memory_usage, 400);
    }

    #[test]
    fn test_esp_operation_compatibility() {
        let tensor = Tensor::vector(vec![1.0; 100]);
        let esp32 = ESPDeviceType::ESP32;
        let esp8266 = ESPDeviceType::ESP8266;

        assert!(ESPTensorUtils::can_run_on_esp(&[tensor.clone()], "relu", &esp32));
        assert!(ESPTensorUtils::can_run_on_esp(&[tensor], "relu", &esp8266));
    }
} 