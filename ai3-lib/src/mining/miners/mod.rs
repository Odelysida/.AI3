use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::mining::tasks::MiningTask;
use crate::mining::results::MiningResult;
use crate::tensor::Tensor;
use tribechain_core::{TribeResult, TribeError};

/// Miner capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerCapabilities {
    pub max_tensor_size: usize,
    pub supported_operations: Vec<String>,
    pub compute_power: u64, // Relative compute power score
    pub is_esp_device: bool,
}

/// Miner statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub tasks_completed: u64,
    pub total_rewards: u64,
    pub average_computation_time: u64,
    pub success_rate: f64,
    pub last_active: DateTime<Utc>,
}

/// AI3 Miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3Miner {
    pub id: String,
    pub address: String,
    pub capabilities: MinerCapabilities,
    pub current_task: Option<MiningTask>,
    pub latest_result: Option<MiningResult>,
    pub stats: MinerStats,
    pub is_active: bool,
}

impl AI3Miner {
    pub fn new(id: String, address: String, is_esp_device: bool) -> Self {
        let capabilities = MinerCapabilities {
            max_tensor_size: if is_esp_device { 1024 } else { 1024 * 1024 }, // 1KB vs 1MB
            supported_operations: vec![
                "matrix_multiply".to_string(),
                "relu".to_string(),
                "sigmoid".to_string(),
                "vector_add".to_string(),
                "dot_product".to_string(),
            ],
            compute_power: if is_esp_device { 100 } else { 1000 },
            is_esp_device,
        };

        let stats = MinerStats {
            tasks_completed: 0,
            total_rewards: 0,
            average_computation_time: 0,
            success_rate: 0.0,
            last_active: Utc::now(),
        };

        Self {
            id,
            address,
            capabilities,
            current_task: None,
            latest_result: None,
            stats,
            is_active: true,
        }
    }

    pub fn can_handle_task(&self, task: &MiningTask) -> bool {
        if !self.is_active {
            return false;
        }

        // Check if operation is supported
        if !self.capabilities.supported_operations.contains(&task.operation_type) {
            return false;
        }

        // Check tensor size constraints
        let total_tensor_size: usize = task.input_tensors
            .iter()
            .map(|t| t.shape.total_elements())
            .sum();

        if total_tensor_size > self.capabilities.max_tensor_size {
            return false;
        }

        true
    }

    pub fn assign_task(&mut self, task: MiningTask) -> TribeResult<()> {
        if !self.can_handle_task(&task) {
            return Err(TribeError::InvalidOperation("Miner cannot handle this task".to_string()));
        }

        self.current_task = Some(task);
        Ok(())
    }

    pub fn mine_step(&mut self) -> TribeResult<Option<MiningResult>> {
        let task = match &self.current_task {
            Some(task) => task.clone(),
            None => return Ok(None),
        };

        if task.is_expired() {
            self.current_task = None;
            return Ok(None);
        }

        let start_time = std::time::Instant::now();

        // Simple mining simulation - try random nonces
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce = rng.gen_range(task.nonce_range.0..=task.nonce_range.1);

        let hash = task.calculate_hash(nonce);
        
        if task.meets_difficulty(&hash) {
            // Found valid hash, execute operation
            let output_tensor = task.execute_operation()?;
            let computation_time = start_time.elapsed().as_millis() as u64;

            let result = MiningResult::new(
                task.id.clone(),
                self.id.clone(),
                nonce,
                hash,
                output_tensor,
                computation_time,
            );

            self.latest_result = Some(result.clone());
            self.current_task = None;
            self.update_stats(computation_time, true);

            return Ok(Some(result));
        }

        Ok(None)
    }

    pub fn get_latest_result(&self) -> Option<MiningResult> {
        self.latest_result.clone()
    }

    fn update_stats(&mut self, computation_time: u64, success: bool) {
        self.stats.last_active = Utc::now();
        
        if success {
            self.stats.tasks_completed += 1;
            
            // Update average computation time
            let total_time = self.stats.average_computation_time * (self.stats.tasks_completed - 1) + computation_time;
            self.stats.average_computation_time = total_time / self.stats.tasks_completed;
            
            // Update success rate (simplified)
            self.stats.success_rate = self.stats.tasks_completed as f64 / (self.stats.tasks_completed + 1) as f64;
        }
    }
} 