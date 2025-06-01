// AI3 Library - Core components for the AI3 mining network
// Reorganized into functional modules for better maintainability

pub mod mining;
pub mod operations;
pub mod tensor;
pub mod esp_compat;

// Re-export key types for convenience
pub use mining::{AI3Miner, MiningTask, MiningResult, TaskDistributor, MinerCapabilities, MinerStats};
pub use operations::{TensorOp, MatrixMultiply, Convolution, ActivationFunction, VectorOp};
pub use tensor::{Tensor, TensorShape, TensorData};
pub use esp_compat::{ESPCompatibility, ESPDeviceType, ESPMiningConfig, ESP32Miner, ESP8266Miner};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Main AI3 Engine - Coordinates all mining and tensor operations
pub struct AI3Engine {
    miners: Vec<AI3Miner>,
    task_distributor: TaskDistributor,
    performance_stats: Arc<Mutex<EngineStats>>,
    config: EngineConfig,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
    pub enable_esp_support: bool,
    pub auto_optimize_tensors: bool,
    pub performance_monitoring: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(30),
            enable_esp_support: true,
            auto_optimize_tensors: true,
            performance_monitoring: true,
        }
    }
}

/// Engine performance statistics
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub total_tasks_processed: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub average_task_time: Duration,
    pub active_miners: usize,
    pub total_compute_time: Duration,
    pub uptime: Duration,
    pub start_time: Instant,
}

impl AI3Engine {
    /// Create a new AI3 Engine with default configuration
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }

    /// Create a new AI3 Engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        let stats = EngineStats {
            start_time: Instant::now(),
            ..Default::default()
        };

        Self {
            miners: Vec::new(),
            task_distributor: TaskDistributor::new(),
            performance_stats: Arc::new(Mutex::new(stats)),
            config,
        }
    }

    /// Add a miner to the engine
    pub fn add_miner(&mut self, miner: AI3Miner) {
        self.miners.push(miner);
        if let Ok(mut stats) = self.performance_stats.lock() {
            stats.active_miners = self.miners.len();
        }
    }

    /// Add an ESP miner with automatic configuration
    pub fn add_esp_miner(&mut self, device_type: ESPDeviceType) -> tribechain_core::TribeResult<()> {
        if !self.config.enable_esp_support {
            return Err(tribechain_core::TribeError::InvalidOperation(
                "ESP support is disabled in engine configuration".to_string()
            ));
        }

        let esp_config = esp_compat::ESPCompatibility::get_recommended_config(device_type);
        let miner_id = format!("esp_miner_{}", uuid::Uuid::new_v4());
        let miner = AI3Miner::new(miner_id, "esp_address".to_string(), true);
        self.add_miner(miner);
        Ok(())
    }

    /// Submit a mining task
    pub fn submit_task(&mut self, task: MiningTask) -> tribechain_core::TribeResult<String> {
        // Auto-optimize tensors if enabled
        let optimized_task = if self.config.auto_optimize_tensors {
            self.optimize_task_tensors(task)?
        } else {
            task
        };

        let task_id = optimized_task.id.clone();
        self.task_distributor.add_task(optimized_task);
        Ok(task_id)
    }

    /// Process pending tasks
    pub fn process_tasks(&mut self) -> tribechain_core::TribeResult<Vec<MiningResult>> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Get pending tasks and distribute to miners
        let pending_tasks = self.task_distributor.get_pending_tasks();
        
        for task in pending_tasks {
            // Find available miners for this task
            for miner in &mut self.miners {
                if miner.can_handle_task(task) && miner.current_task.is_none() {
                    // Assign task to miner
                    if let Ok(()) = miner.assign_task(task.clone()) {
                        // Try mining step
                        match miner.mine_step() {
                            Ok(Some(result)) => {
                                results.push(result);
                                self.update_stats(true, start_time.elapsed());
                            }
                            Ok(None) => {
                                // No result yet, continue mining
                            }
                            Err(e) => {
                                eprintln!("Task processing failed: {}", e);
                                self.update_stats(false, start_time.elapsed());
                            }
                        }
                        break; // Task assigned, move to next task
                    }
                }
            }
        }

        // Clean up expired tasks
        self.task_distributor.cleanup_expired_tasks();

        Ok(results)
    }

    /// Get engine performance statistics
    pub fn get_stats(&self) -> EngineStats {
        if let Ok(mut stats) = self.performance_stats.lock() {
            stats.uptime = stats.start_time.elapsed();
            stats.clone()
        } else {
            EngineStats::default()
        }
    }

    /// Get miner capabilities summary
    pub fn get_miner_capabilities(&self) -> Vec<MinerCapabilities> {
        self.miners.iter().map(|miner| miner.capabilities.clone()).collect()
    }

    /// Optimize task tensors for available miners
    fn optimize_task_tensors(&self, mut task: MiningTask) -> tribechain_core::TribeResult<MiningTask> {
        // Check if we have ESP miners and optimize accordingly
        let has_esp_miners = self.miners.iter().any(|miner| {
            miner.capabilities.is_esp_device
        });

        if has_esp_miners && self.config.enable_esp_support {
            // Find the most restrictive ESP device
            let most_restrictive_device = self.get_most_restrictive_esp_device();
            
            if let Some(device_type) = most_restrictive_device {
                // Optimize tensors for the most restrictive device
                for tensor in &mut task.input_tensors {
                    *tensor = esp_compat::ESPCompatibility::optimize_for_esp(tensor, &device_type)?;
                }
            }
        }

        Ok(task)
    }

    /// Find the most restrictive ESP device among miners
    fn get_most_restrictive_esp_device(&self) -> Option<ESPDeviceType> {
        let mut min_memory = usize::MAX;
        let mut most_restrictive = None;

        for miner in &self.miners {
            if miner.capabilities.is_esp_device {
                // For ESP devices, use ESP32 as default (could be made more sophisticated)
                let memory = 320 * 1024; // ESP32 typical memory
                if memory < min_memory {
                    min_memory = memory;
                    most_restrictive = Some(ESPDeviceType::ESP32);
                }
            }
        }

        most_restrictive
    }

    /// Update performance statistics
    fn update_stats(&self, success: bool, duration: Duration) {
        if let Ok(mut stats) = self.performance_stats.lock() {
            stats.total_tasks_processed += 1;
            
            if success {
                stats.successful_tasks += 1;
            } else {
                stats.failed_tasks += 1;
            }
            
            // Update average task time
            let total_time = stats.average_task_time * (stats.total_tasks_processed - 1) as u32 + duration;
            stats.average_task_time = total_time / stats.total_tasks_processed as u32;
            
            stats.total_compute_time += duration;
        }
    }

    /// Shutdown the engine
    pub fn shutdown(&mut self) {
        // Clean up resources
        self.miners.clear();
        
        if let Ok(mut stats) = self.performance_stats.lock() {
            stats.active_miners = 0;
        }
    }
}

impl Default for AI3Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for ESPDeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "esp32" => Ok(ESPDeviceType::ESP32),
            "esp8266" => Ok(ESPDeviceType::ESP8266),
            _ => Err(format!("Unknown ESP device type: {}", s)),
        }
    }
}

/// Utility functions for AI3 Engine
pub mod utils {
    use super::*;

    /// Create an AI3 engine with ESP mining setup
    pub fn create_esp_mining_setup(device_types: Vec<ESPDeviceType>) -> tribechain_core::TribeResult<AI3Engine> {
        let mut engine = AI3Engine::new();
        
        for device_type in device_types {
            engine.add_esp_miner(device_type)?;
        }
        
        Ok(engine)
    }

    /// Benchmark a tensor operation
    pub fn benchmark_operation(
        operation: &str, 
        tensor: &Tensor, 
        iterations: usize
    ) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Simulate operation execution
            let _result = match operation {
                "relu" => tensor.clone(), // Simplified
                "matrix_multiply" => tensor.clone(),
                _ => tensor.clone(),
            };
        }
        
        start.elapsed() / iterations as u32
    }

    /// Generate a performance report for the engine
    pub fn generate_performance_report(engine: &AI3Engine) -> String {
        let stats = engine.get_stats();
        let capabilities = engine.get_miner_capabilities();
        
        format!(
            "AI3 Engine Performance Report\n\
             =============================\n\
             Uptime: {:?}\n\
             Total Tasks: {}\n\
             Successful: {}\n\
             Failed: {}\n\
             Success Rate: {:.2}%\n\
             Average Task Time: {:?}\n\
             Active Miners: {}\n\
             Total Compute Time: {:?}\n\
             \n\
             Miner Capabilities:\n\
             {}\n",
            stats.uptime,
            stats.total_tasks_processed,
            stats.successful_tasks,
            stats.failed_tasks,
            if stats.total_tasks_processed > 0 {
                (stats.successful_tasks as f64 / stats.total_tasks_processed as f64) * 100.0
            } else {
                0.0
            },
            stats.average_task_time,
            stats.active_miners,
            stats.total_compute_time,
            capabilities.iter()
                .enumerate()
                .map(|(i, cap)| format!("  Miner {}: {} ops, {}KB max tensor, ESP: {}", 
                                      i, cap.supported_operations.len(), 
                                      cap.max_tensor_size / 1024, cap.is_esp_device))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
} 