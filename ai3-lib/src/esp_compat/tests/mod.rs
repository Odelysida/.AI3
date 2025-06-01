#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::esp_compat::devices::ESPDeviceType;
    use crate::esp_compat::config::ESPMiningConfig;
    use crate::esp_compat::miners::{ESP32Miner, ESP8266Miner, ConnectionStatus};
    use crate::esp_compat::codegen::{ESPTensorUtils, ESPCodeGenerator};
    use crate::tensor::Tensor;

    #[test]
    fn test_esp_device_types() {
        let esp32 = ESPDeviceType::ESP32;
        assert_eq!(esp32.get_memory_limit(), 520);
        assert_eq!(esp32.get_compute_power(), 240);
        assert!(esp32.supports_floating_point());

        let esp8266 = ESPDeviceType::ESP8266;
        assert_eq!(esp8266.get_memory_limit(), 80);
        assert_eq!(esp8266.get_compute_power(), 80);
        assert!(!esp8266.supports_floating_point());

        let esp32_s3 = ESPDeviceType::ESP32S3;
        assert_eq!(esp32_s3.get_memory_limit(), 512);
        assert_eq!(esp32_s3.get_compute_power(), 240);
        assert!(esp32_s3.supports_floating_point());
    }

    #[test]
    fn test_esp_mining_config() {
        let config = ESPMiningConfig::default();
        assert_eq!(config.device_type, ESPDeviceType::ESP32);
        assert_eq!(config.max_memory_kb, 520);
        assert_eq!(config.clock_speed_mhz, 240);
        assert_eq!(config.wifi_ssid, "TribeChain_Network");
        assert_eq!(config.server_address, "192.168.1.100");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.mining_intensity, 5);
        assert!(!config.power_save_mode);
    }

    #[test]
    fn test_esp32_miner_creation() {
        let config = ESPMiningConfig::default();
        let miner = ESP32Miner::new(config);
        
        assert_eq!(miner.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(miner.performance_stats.uptime_seconds, 0);
        assert_eq!(miner.performance_stats.successful_tasks, 0);
        assert_eq!(miner.performance_stats.failed_tasks, 0);
    }

    #[test]
    fn test_esp8266_miner_creation() {
        let mut config = ESPMiningConfig::default();
        config.device_type = ESPDeviceType::ESP8266;
        config.max_memory_kb = 80;
        
        let miner = ESP8266Miner::new(config);
        
        assert_eq!(miner.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(miner.performance_stats.uptime_seconds, 0);
    }

    #[test]
    fn test_esp_memory_check() {
        let config = ESPMiningConfig::default();
        let miner = ESP32Miner::new(config);
        
        // Test with small tensor (should pass)
        let small_tensor = Tensor::zeros(vec![10, 10]).unwrap();
        assert!(miner.check_memory_constraints(&small_tensor));
        
        // Test with large tensor (should fail)
        let large_tensor = Tensor::zeros(vec![1000, 1000]).unwrap();
        assert!(!miner.check_memory_constraints(&large_tensor));
    }

    #[test]
    fn test_esp_tensor_optimization() {
        let tensor = Tensor::from_vec(vec![1.5, -2.3, 0.7, -0.1], vec![2, 2]).unwrap();
        
        // Test optimization for ESP32 (should preserve floating point)
        let esp32_result = ESPTensorUtils::optimize_tensor_for_esp(&tensor, &ESPDeviceType::ESP32);
        assert!(esp32_result.is_ok());
        
        // Test optimization for ESP8266 (should convert to fixed point)
        let esp8266_result = ESPTensorUtils::optimize_tensor_for_esp(&tensor, &ESPDeviceType::ESP8266);
        assert!(esp8266_result.is_ok());
        
        let optimized = esp8266_result.unwrap();
        let data = optimized.data.as_f32_vec().unwrap();
        
        // Values should be quantized due to fixed-point conversion
        assert!((data[0] - 1.49609375).abs() < 0.01); // 1.5 quantized to Q8.8
    }

    #[test]
    fn test_esp_memory_estimation() {
        let tensor1 = Tensor::zeros(vec![10, 10]).unwrap();
        let tensor2 = Tensor::zeros(vec![10, 5]).unwrap();
        let tensors = vec![tensor1, tensor2];
        
        let matrix_memory = ESPTensorUtils::estimate_memory_usage(&tensors, "matrix_multiply");
        let relu_memory = ESPTensorUtils::estimate_memory_usage(&tensors, "relu");
        let dot_memory = ESPTensorUtils::estimate_memory_usage(&tensors, "dot_product");
        
        // Matrix multiply should use more memory than ReLU
        assert!(matrix_memory >= relu_memory);
        
        // Dot product should use less memory (single output value)
        assert!(dot_memory < relu_memory);
    }

    #[test]
    fn test_esp_operation_feasibility() {
        let small_tensor = Tensor::zeros(vec![5, 5]).unwrap();
        let large_tensor = Tensor::zeros(vec![100, 100]).unwrap();
        
        // Small tensor should work on ESP8266
        assert!(ESPTensorUtils::can_run_on_esp(
            &[small_tensor.clone()], 
            "matrix_multiply", 
            &ESPDeviceType::ESP8266
        ));
        
        // Large tensor should not work on ESP8266
        assert!(!ESPTensorUtils::can_run_on_esp(
            &[large_tensor.clone()], 
            "matrix_multiply", 
            &ESPDeviceType::ESP8266
        ));
        
        // Large tensor should work on ESP32 (more memory)
        assert!(ESPTensorUtils::can_run_on_esp(
            &[large_tensor], 
            "matrix_multiply", 
            &ESPDeviceType::ESP32
        ));
    }

    #[test]
    fn test_esp_code_generation() {
        let config = ESPMiningConfig::default();
        let code = ESPCodeGenerator::generate_mining_code(&config);
        
        // Check that generated code contains expected elements
        assert!(code.contains("WiFi.h"));
        assert!(code.contains("HTTPClient.h"));
        assert!(code.contains("TribeChain_Network")); // Default SSID
        assert!(code.contains("192.168.1.100")); // Default server address
        assert!(code.contains("8080")); // Default port
        assert!(code.contains("void setup()"));
        assert!(code.contains("void loop()"));
        assert!(code.contains("getMiningTask()"));
        assert!(code.contains("processMiningTask"));
        assert!(code.contains("submitResult"));
    }

    #[test]
    fn test_esp_tensor_operations_code() {
        let code = ESPCodeGenerator::generate_tensor_operations();
        
        // Check that generated code contains expected tensor operations
        assert!(code.contains("struct Tensor"));
        assert!(code.contains("createTensor"));
        assert!(code.contains("freeTensor"));
        assert!(code.contains("relu_esp"));
        assert!(code.contains("vector_add_esp"));
        assert!(code.contains("dot_product_esp"));
        assert!(code.contains("matrix_multiply_esp"));
        assert!(code.contains("fixed_t")); // Fixed-point support
        assert!(code.contains("float_to_fixed"));
        assert!(code.contains("fixed_to_float"));
    }

    #[test]
    fn test_esp_performance_monitoring() {
        let config = ESPMiningConfig::default();
        let mut miner = ESP32Miner::new(config);
        
        // Simulate some mining activity
        miner.performance_stats.successful_tasks = 10;
        miner.performance_stats.failed_tasks = 2;
        miner.performance_stats.uptime_seconds = 3600; // 1 hour
        miner.performance_stats.cpu_temperature = 65.0;
        
        // Test throttling check
        assert!(!miner.should_throttle()); // 65°C should be fine
        
        miner.performance_stats.cpu_temperature = 85.0;
        assert!(miner.should_throttle()); // 85°C should trigger throttling
    }

    #[test]
    fn test_esp_task_optimization() {
        let config = ESPMiningConfig::default();
        let miner = ESP32Miner::new(config);
        
        // Test task optimization for different operation types
        let matrix_task = "matrix_multiply";
        let relu_task = "relu";
        let conv_task = "convolution";
        
        // All should return some optimization (even if minimal)
        assert!(miner.optimize_task_for_device(matrix_task).len() > 0);
        assert!(miner.optimize_task_for_device(relu_task).len() > 0);
        assert!(miner.optimize_task_for_device(conv_task).len() > 0);
    }

    #[test]
    fn test_esp8266_fixed_point_conversion() {
        let mut config = ESPMiningConfig::default();
        config.device_type = ESPDeviceType::ESP8266;
        
        let miner = ESP8266Miner::new(config);
        
        // Test fixed-point conversion
        let float_val = 3.14159;
        let fixed_val = miner.float_to_fixed_point(float_val);
        let converted_back = miner.fixed_point_to_float(fixed_val);
        
        // Should be close but not exact due to quantization
        assert!((converted_back - float_val).abs() < 0.01);
    }

    #[test]
    fn test_esp_connection_status() {
        let config = ESPMiningConfig::default();
        let mut miner = ESP32Miner::new(config);
        
        // Test connection status transitions
        assert_eq!(miner.connection_status, ConnectionStatus::Disconnected);
        
        // Simulate connection process
        miner.connection_status = ConnectionStatus::Connecting;
        assert_eq!(miner.connection_status, ConnectionStatus::Connecting);
        
        miner.connection_status = ConnectionStatus::Connected;
        assert_eq!(miner.connection_status, ConnectionStatus::Connected);
        
        miner.connection_status = ConnectionStatus::Error;
        assert_eq!(miner.connection_status, ConnectionStatus::Error);
    }
} 