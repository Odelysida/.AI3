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
    miners: Vec<Box<dyn AI3Miner>>,
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
    pub fn add_miner(&mut self, miner: Box<dyn AI3Miner>) {
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
        let miner = esp_compat::ESPCompatibility::create_miner(esp_config);
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

        self.task_distributor.add_task(optimized_task)
    }

    /// Process pending tasks
    pub fn process_tasks(&mut self) -> tribechain_core::TribeResult<Vec<MiningResult>> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Distribute tasks to available miners
        for miner in &mut self.miners {
            if let Some(task) = self.task_distributor.get_next_task() {
                match miner.process_task(&task) {
                    Ok(result) => {
                        results.push(result);
                        self.update_stats(true, start_time.elapsed());
                    }
                    Err(e) => {
                        eprintln!("Task processing failed: {}", e);
                        self.update_stats(false, start_time.elapsed());
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
        self.miners.iter().map(|miner| miner.get_capabilities()).collect()
    }

    /// Optimize task tensors for available miners
    fn optimize_task_tensors(&self, mut task: MiningTask) -> tribechain_core::TribeResult<MiningTask> {
        // Check if we have ESP miners and optimize accordingly
        let has_esp_miners = self.miners.iter().any(|miner| {
            miner.get_capabilities().device_type.contains("ESP")
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
            let caps = miner.get_capabilities();
            if caps.device_type.contains("ESP") {
                // Parse device type and check memory
                if let Ok(device_type) = caps.device_type.parse::<ESPDeviceType>() {
                    let memory = device_type.get_memory_limit();
                    if memory < min_memory {
                        min_memory = memory;
                        most_restrictive = Some(device_type);
                    }
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
            
            stats.total_compute_time += duration;
            stats.average_task_time = stats.total_compute_time / stats.total_tasks_processed as u32;
        }
    }

    /// Shutdown the engine gracefully
    pub fn shutdown(&mut self) {
        // Stop all miners
        for miner in &mut self.miners {
            // Miners should implement graceful shutdown
            drop(miner);
        }
        
        // Clear task queue
        self.task_distributor = TaskDistributor::new();
        
        println!("AI3 Engine shutdown complete");
    }
}

impl Default for AI3Engine {
    fn default() -> Self {
        Self::new()
    }
}

// Implement string parsing for ESPDeviceType
impl std::str::FromStr for ESPDeviceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ESP32" => Ok(ESPDeviceType::ESP32),
            "ESP8266" => Ok(ESPDeviceType::ESP8266),
            "ESP32S2" => Ok(ESPDeviceType::ESP32S2),
            "ESP32S3" => Ok(ESPDeviceType::ESP32S3),
            "ESP32C3" => Ok(ESPDeviceType::ESP32C3),
            _ => Err(format!("Unknown ESP device type: {}", s)),
        }
    }
}

/// Utility functions for the AI3 library
pub mod utils {
    use super::*;

    /// Create a simple AI3 engine with ESP support
    pub fn create_esp_mining_setup(device_types: Vec<ESPDeviceType>) -> tribechain_core::TribeResult<AI3Engine> {
        let mut engine = AI3Engine::new();
        
        for device_type in device_types {
            engine.add_esp_miner(device_type)?;
        }
        
        Ok(engine)
    }

    /// Benchmark tensor operation performance
    pub fn benchmark_operation(
        operation: &str, 
        tensor: &Tensor, 
        iterations: usize
    ) -> Duration {
        let start = Instant::now();
        
        for _ in 0..iterations {
            // Simulate operation execution
            match operation {
                "matrix_multiply" => {
                    let _ = MatrixMultiply::new().execute(&[tensor.clone()]);
                }
                "convolution" => {
                    let _ = Convolution::new(3).execute(&[tensor.clone()]);
                }
                "relu" => {
                    let _ = ActivationFunction::relu().execute(&[tensor.clone()]);
                }
                _ => {
                    // Default operation
                    std::thread::sleep(Duration::from_micros(100));
                }
            }
        }
        
        start.elapsed() / iterations as u32
    }

    /// Generate performance report
    pub fn generate_performance_report(engine: &AI3Engine) -> String {
        let stats = engine.get_stats();
        let capabilities = engine.get_miner_capabilities();
        
        format!(
            "=== AI3 Engine Performance Report ===\n\
            Uptime: {:?}\n\
            Total Tasks: {}\n\
            Successful: {} ({:.1}%)\n\
            Failed: {} ({:.1}%)\n\
            Average Task Time: {:?}\n\
            Active Miners: {}\n\
            Total Compute Time: {:?}\n\
            \n\
            Miner Capabilities:\n{}\n\
            =====================================",
            stats.uptime,
            stats.total_tasks_processed,
            stats.successful_tasks,
            (stats.successful_tasks as f64 / stats.total_tasks_processed as f64) * 100.0,
            stats.failed_tasks,
            (stats.failed_tasks as f64 / stats.total_tasks_processed as f64) * 100.0,
            stats.average_task_time,
            stats.active_miners,
            stats.total_compute_time,
            capabilities.iter()
                .enumerate()
                .map(|(i, cap)| format!("  Miner {}: {} (Memory: {}KB, Compute: {}MHz)", 
                    i + 1, cap.device_type, cap.memory_limit_kb, cap.compute_power_mhz))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
} 