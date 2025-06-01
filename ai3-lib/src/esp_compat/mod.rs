// ESP Compatibility Module
// Provides support for ESP32/ESP8266 devices in the AI3 mining network

pub mod devices;
pub mod config;
pub mod miners;
pub mod codegen;
pub mod tests;

// Re-export key types for convenience
pub use devices::ESPDeviceType;
pub use config::ESPMiningConfig;
pub use miners::{ESP32Miner, ESP8266Miner, ConnectionStatus, ESPPerformanceStats};
pub use codegen::{ESPTensorUtils, ESPCodeGenerator};

use crate::tensor::Tensor;
use crate::mining::{AI3Miner, MinerCapabilities, MinerStats};

/// Main ESP compatibility interface
pub struct ESPCompatibility;

impl ESPCompatibility {
    /// Check if a tensor operation is compatible with ESP devices
    pub fn is_esp_compatible(tensor: &Tensor, operation: &str, device_type: &ESPDeviceType) -> bool {
        ESPTensorUtils::can_run_on_esp(&[tensor.clone()], operation, device_type)
    }

    /// Optimize tensor for ESP device constraints
    pub fn optimize_for_esp(tensor: &Tensor, device_type: &ESPDeviceType) -> tribechain_core::TribeResult<Tensor> {
        ESPTensorUtils::optimize_tensor_for_esp(tensor, device_type)
    }

    /// Generate Arduino code for ESP mining
    pub fn generate_mining_code(config: &ESPMiningConfig) -> String {
        ESPCodeGenerator::generate_mining_code(config)
    }

    /// Generate C++ tensor operations for ESP
    pub fn generate_tensor_operations() -> String {
        ESPCodeGenerator::generate_tensor_operations()
    }

    /// Get recommended configuration for device type
    pub fn get_recommended_config(device_type: ESPDeviceType) -> ESPMiningConfig {
        let mut config = ESPMiningConfig::default();
        config.device_type = device_type.clone();
        config.max_memory_kb = device_type.get_memory_limit();
        config.clock_speed_mhz = device_type.get_compute_power();
        
        // Adjust settings based on device capabilities
        match device_type {
            ESPDeviceType::ESP8266 => {
                config.mining_intensity = 3; // Lower intensity for limited device
                config.power_save_mode = true; // Enable power saving
            },
            ESPDeviceType::ESP32 => {
                config.mining_intensity = 5; // Moderate intensity
                config.power_save_mode = false;
            },
            ESPDeviceType::ESP32S2 => {
                config.mining_intensity = 6; // Higher intensity
                config.power_save_mode = false;
            },
            ESPDeviceType::ESP32S3 => {
                config.mining_intensity = 7; // Highest intensity
                config.power_save_mode = false;
            },
            ESPDeviceType::ESP32C3 => {
                config.mining_intensity = 4; // Conservative for RISC-V
                config.power_save_mode = false;
            },
        }
        
        config
    }

    /// Create an ESP miner instance
    pub fn create_miner(config: ESPMiningConfig) -> Box<dyn AI3Miner> {
        match config.device_type {
            ESPDeviceType::ESP8266 => Box::new(ESP8266Miner::new(config)),
            _ => Box::new(ESP32Miner::new(config)),
        }
    }

    /// Estimate mining performance for device
    pub fn estimate_performance(device_type: &ESPDeviceType, operation: &str) -> f32 {
        let base_performance = device_type.get_compute_power() as f32;
        
        let operation_multiplier = match operation {
            "matrix_multiply" => 0.8, // Computationally intensive
            "convolution" => 0.7,     // Very intensive
            "relu" | "sigmoid" => 1.2, // Simple operations
            "dot_product" => 1.0,     // Moderate
            "softmax" => 0.9,         // Moderate to intensive
            _ => 1.0,
        };
        
        let memory_factor = if device_type.get_memory_limit() < 100 { 0.8 } else { 1.0 };
        let fp_factor = if device_type.supports_floating_point() { 1.0 } else { 0.6 };
        
        base_performance * operation_multiplier * memory_factor * fp_factor
    }

    /// Get device limitations and recommendations
    pub fn get_device_info(device_type: &ESPDeviceType) -> DeviceInfo {
        DeviceInfo {
            device_type: device_type.clone(),
            memory_limit_kb: device_type.get_memory_limit(),
            compute_power_mhz: device_type.get_compute_power(),
            supports_floating_point: device_type.supports_floating_point(),
            recommended_operations: Self::get_recommended_operations(device_type),
            limitations: Self::get_device_limitations(device_type),
        }
    }

    fn get_recommended_operations(device_type: &ESPDeviceType) -> Vec<String> {
        let mut operations = vec![
            "relu".to_string(),
            "sigmoid".to_string(),
            "dot_product".to_string(),
            "vector_add".to_string(),
        ];

        if device_type.supports_floating_point() && device_type.get_memory_limit() > 100 {
            operations.extend(vec![
                "matrix_multiply".to_string(),
                "convolution".to_string(),
                "softmax".to_string(),
            ]);
        }

        operations
    }

    fn get_device_limitations(device_type: &ESPDeviceType) -> Vec<String> {
        let mut limitations = Vec::new();

        if device_type.get_memory_limit() < 100 {
            limitations.push("Limited memory - avoid large tensors".to_string());
        }

        if !device_type.supports_floating_point() {
            limitations.push("No hardware floating point - use fixed-point arithmetic".to_string());
        }

        if device_type.get_compute_power() < 100 {
            limitations.push("Limited compute power - prefer simple operations".to_string());
        }

        limitations
    }
}

/// Device information structure
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: ESPDeviceType,
    pub memory_limit_kb: usize,
    pub compute_power_mhz: usize,
    pub supports_floating_point: bool,
    pub recommended_operations: Vec<String>,
    pub limitations: Vec<String>,
}

/// ESP-specific error types
#[derive(Debug, Clone)]
pub enum ESPError {
    InsufficientMemory(String),
    UnsupportedOperation(String),
    DeviceNotSupported(String),
    ConfigurationError(String),
    ConnectionError(String),
}

impl std::fmt::Display for ESPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ESPError::InsufficientMemory(msg) => write!(f, "Insufficient memory: {}", msg),
            ESPError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
            ESPError::DeviceNotSupported(msg) => write!(f, "Device not supported: {}", msg),
            ESPError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            ESPError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for ESPError {}

/// Utility functions for ESP development
pub mod utils {
    use super::*;

    /// Convert tensor size to memory usage estimate
    pub fn tensor_memory_usage(tensor: &Tensor) -> usize {
        tensor.shape.total_elements() * 4 // 4 bytes per f32
    }

    /// Check if tensor fits in device memory
    pub fn tensor_fits_in_memory(tensor: &Tensor, device_type: &ESPDeviceType) -> bool {
        let tensor_size = tensor_memory_usage(tensor);
        let available_memory = (device_type.get_memory_limit() * 1024) as f32 * 0.75; // 75% usable
        tensor_size <= available_memory as usize
    }

    /// Generate device-specific compiler flags
    pub fn get_compiler_flags(device_type: &ESPDeviceType) -> Vec<String> {
        let mut flags = vec![
            "-Os".to_string(), // Optimize for size
            "-ffunction-sections".to_string(),
            "-fdata-sections".to_string(),
        ];

        match device_type {
            ESPDeviceType::ESP8266 => {
                flags.extend(vec![
                    "-DESP8266".to_string(),
                    "-mno-serialize-volatile".to_string(),
                ]);
            },
            ESPDeviceType::ESP32 => {
                flags.extend(vec![
                    "-DESP32".to_string(),
                    "-mfix-esp32-psram-cache-issue".to_string(),
                ]);
            },
            ESPDeviceType::ESP32S2 => {
                flags.extend(vec![
                    "-DESP32S2".to_string(),
                ]);
            },
            ESPDeviceType::ESP32S3 => {
                flags.extend(vec![
                    "-DESP32S3".to_string(),
                ]);
            },
            ESPDeviceType::ESP32C3 => {
                flags.extend(vec![
                    "-DESP32C3".to_string(),
                    "-march=rv32imc".to_string(), // RISC-V specific
                ]);
            },
        }

        flags
    }

    /// Generate linker script recommendations
    pub fn get_linker_recommendations(device_type: &ESPDeviceType) -> String {
        match device_type {
            ESPDeviceType::ESP8266 => {
                "Use eagle.flash.4m1m.ld for 4MB flash with 1MB SPIFFS".to_string()
            },
            ESPDeviceType::ESP32 => {
                "Use esp32.common.ld with appropriate partition table".to_string()
            },
            ESPDeviceType::ESP32S2 => {
                "Use esp32s2.common.ld with USB CDC support".to_string()
            },
            ESPDeviceType::ESP32S3 => {
                "Use esp32s3.common.ld with PSRAM support if available".to_string()
            },
            ESPDeviceType::ESP32C3 => {
                "Use esp32c3.common.ld with RISC-V optimizations".to_string()
            },
        }
    }
} 