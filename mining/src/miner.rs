use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tribechain_core::{TribeResult, TribeError};

/// Basic miner structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Miner {
    pub id: String,
    pub address: String,
    pub miner_type: MinerType,
    pub capabilities: MinerCapabilities,
    pub stats: MinerStats,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// Types of miners supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinerType {
    CPU,
    GPU,
    ASIC,
    ESP32,
    ESP8266,
    AI3Tensor,
}

/// Miner capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerCapabilities {
    pub hash_rate: f64,
    pub power_consumption: u32, // Watts
    pub supports_ai3: bool,
    pub max_tensor_size: usize,
    pub supported_operations: Vec<String>,
    pub memory_limit: usize, // KB
    pub compute_units: u32,
}

/// Miner statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub blocks_mined: u64,
    pub total_hash_attempts: u64,
    pub successful_hashes: u64,
    pub uptime: u64, // seconds
    pub average_hash_time: f64, // milliseconds
    pub power_efficiency: f64, // hashes per watt
    pub last_block_time: Option<DateTime<Utc>>,
    pub earnings: u64,
}

impl Miner {
    pub fn new(id: String, address: String, miner_type: MinerType) -> Self {
        let capabilities = match miner_type {
            MinerType::CPU => MinerCapabilities {
                hash_rate: 1000.0,
                power_consumption: 100,
                supports_ai3: true,
                max_tensor_size: 1024,
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "convolution".to_string(),
                    "neural_forward".to_string(),
                ],
                memory_limit: 8192, // 8MB
                compute_units: 4,
            },
            MinerType::GPU => MinerCapabilities {
                hash_rate: 50000.0,
                power_consumption: 300,
                supports_ai3: true,
                max_tensor_size: 16384,
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "convolution".to_string(),
                    "neural_forward".to_string(),
                    "tensor_add".to_string(),
                    "tensor_multiply".to_string(),
                ],
                memory_limit: 65536, // 64MB
                compute_units: 2048,
            },
            MinerType::ASIC => MinerCapabilities {
                hash_rate: 100000.0,
                power_consumption: 1500,
                supports_ai3: false,
                max_tensor_size: 0,
                supported_operations: vec![],
                memory_limit: 1024, // 1MB
                compute_units: 1,
            },
            MinerType::ESP32 => MinerCapabilities {
                hash_rate: 10.0,
                power_consumption: 5,
                supports_ai3: true,
                max_tensor_size: 256,
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "tensor_add".to_string(),
                ],
                memory_limit: 512, // 512KB
                compute_units: 2,
            },
            MinerType::ESP8266 => MinerCapabilities {
                hash_rate: 5.0,
                power_consumption: 3,
                supports_ai3: true,
                max_tensor_size: 128,
                supported_operations: vec![
                    "tensor_add".to_string(),
                ],
                memory_limit: 160, // 160KB
                compute_units: 1,
            },
            MinerType::AI3Tensor => MinerCapabilities {
                hash_rate: 25000.0,
                power_consumption: 200,
                supports_ai3: true,
                max_tensor_size: 32768,
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "convolution".to_string(),
                    "neural_forward".to_string(),
                    "tensor_add".to_string(),
                    "tensor_multiply".to_string(),
                    "activation_functions".to_string(),
                ],
                memory_limit: 131072, // 128MB
                compute_units: 1024,
            },
        };

        Self {
            id,
            address,
            miner_type,
            capabilities,
            stats: MinerStats::default(),
            is_active: true,
            created_at: Utc::now(),
            last_seen: Utc::now(),
        }
    }

    pub fn update_stats(&mut self, hash_attempts: u64, successful: bool, hash_time: f64) {
        self.stats.total_hash_attempts += hash_attempts;
        if successful {
            self.stats.successful_hashes += 1;
            self.stats.blocks_mined += 1;
            self.stats.last_block_time = Some(Utc::now());
        }
        
        // Update average hash time
        let total_time = self.stats.average_hash_time * (self.stats.total_hash_attempts - hash_attempts) as f64;
        self.stats.average_hash_time = (total_time + hash_time) / self.stats.total_hash_attempts as f64;
        
        // Update power efficiency
        if self.capabilities.power_consumption > 0 {
            self.stats.power_efficiency = self.capabilities.hash_rate / self.capabilities.power_consumption as f64;
        }
        
        self.last_seen = Utc::now();
    }

    pub fn can_handle_tensor_operation(&self, operation: &str, tensor_size: usize) -> bool {
        self.capabilities.supports_ai3 &&
        self.capabilities.supported_operations.contains(&operation.to_string()) &&
        tensor_size <= self.capabilities.max_tensor_size
    }

    pub fn get_efficiency_score(&self) -> f64 {
        if self.stats.total_hash_attempts == 0 {
            return 0.0;
        }
        
        let success_rate = self.stats.successful_hashes as f64 / self.stats.total_hash_attempts as f64;
        let power_efficiency = self.stats.power_efficiency;
        let uptime_factor = if self.stats.uptime > 0 { 
            (self.stats.uptime as f64 / 86400.0).min(1.0) // Max 1 day
        } else { 
            0.0 
        };
        
        success_rate * power_efficiency * uptime_factor
    }

    pub fn is_online(&self) -> bool {
        let now = Utc::now();
        let offline_threshold = chrono::Duration::minutes(5);
        now.signed_duration_since(self.last_seen) < offline_threshold
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.last_seen = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

impl Default for MinerStats {
    fn default() -> Self {
        Self {
            blocks_mined: 0,
            total_hash_attempts: 0,
            successful_hashes: 0,
            uptime: 0,
            average_hash_time: 0.0,
            power_efficiency: 0.0,
            last_block_time: None,
            earnings: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let miner = Miner::new(
            "test_miner".to_string(),
            "test_address".to_string(),
            MinerType::CPU,
        );
        
        assert_eq!(miner.id, "test_miner");
        assert_eq!(miner.address, "test_address");
        assert!(miner.is_active);
        assert!(miner.capabilities.supports_ai3);
    }

    #[test]
    fn test_miner_capabilities() {
        let gpu_miner = Miner::new("gpu".to_string(), "addr".to_string(), MinerType::GPU);
        let esp32_miner = Miner::new("esp32".to_string(), "addr".to_string(), MinerType::ESP32);
        
        assert!(gpu_miner.capabilities.hash_rate > esp32_miner.capabilities.hash_rate);
        assert!(gpu_miner.capabilities.max_tensor_size > esp32_miner.capabilities.max_tensor_size);
    }

    #[test]
    fn test_tensor_operation_capability() {
        let miner = Miner::new("test".to_string(), "addr".to_string(), MinerType::CPU);
        
        assert!(miner.can_handle_tensor_operation("matrix_multiply", 512));
        assert!(!miner.can_handle_tensor_operation("matrix_multiply", 2048));
        assert!(!miner.can_handle_tensor_operation("unsupported_op", 256));
    }

    #[test]
    fn test_miner_stats_update() {
        let mut miner = Miner::new("test".to_string(), "addr".to_string(), MinerType::CPU);
        
        miner.update_stats(1000, true, 100.0);
        assert_eq!(miner.stats.total_hash_attempts, 1000);
        assert_eq!(miner.stats.successful_hashes, 1);
        assert_eq!(miner.stats.blocks_mined, 1);
        assert_eq!(miner.stats.average_hash_time, 100.0);
    }
} 