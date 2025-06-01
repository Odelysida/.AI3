use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::tensor::Tensor;
use crate::operations::{TensorOp, MatrixMultiply, Convolution, ActivationFunction, VectorOp};
use tribechain_core::{TribeResult, TribeError};

/// Mining task for tensor operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningTask {
    pub id: String,
    pub operation_type: String,
    pub input_tensors: Vec<Tensor>,
    pub expected_output_shape: Option<Vec<usize>>,
    pub difficulty: u64,
    pub reward: u64,
    pub max_computation_time: u64, // seconds
    pub created_at: DateTime<Utc>,
    pub requester: String,
    pub nonce_range: (u64, u64), // Range for mining nonce
}

impl MiningTask {
    pub fn new(
        operation_type: String,
        input_tensors: Vec<Tensor>,
        difficulty: u64,
        reward: u64,
        max_computation_time: u64,
        requester: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation_type,
            input_tensors,
            expected_output_shape: None,
            difficulty,
            reward,
            max_computation_time,
            created_at: Utc::now(),
            requester,
            nonce_range: (0, u64::MAX),
        }
    }

    pub fn with_expected_output(mut self, shape: Vec<usize>) -> Self {
        self.expected_output_shape = Some(shape);
        self
    }

    pub fn with_nonce_range(mut self, start: u64, end: u64) -> Self {
        self.nonce_range = (start, end);
        self
    }

    /// Calculate task hash for mining
    pub fn calculate_hash(&self, nonce: u64) -> String {
        use sha2::{Digest, Sha256};
        
        let task_data = format!(
            "{}{}{}{}{}{}",
            self.id,
            self.operation_type,
            self.difficulty,
            self.reward,
            self.requester,
            nonce
        );
        
        let mut hasher = Sha256::new();
        hasher.update(task_data.as_bytes());
        
        // Include tensor data in hash
        for tensor in &self.input_tensors {
            hasher.update(tensor.calculate_hash().as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }

    /// Check if hash meets difficulty target
    pub fn meets_difficulty(&self, hash: &str) -> bool {
        let leading_zeros = hash.chars().take_while(|&c| c == '0').count();
        leading_zeros >= (self.difficulty as usize)
    }

    /// Get operation instance
    pub fn get_operation(&self) -> TribeResult<Box<dyn TensorOp>> {
        match self.operation_type.as_str() {
            "matrix_multiply" => Ok(Box::new(MatrixMultiply::new())),
            "convolution" => Ok(Box::new(Convolution::new(3))), // Default kernel size
            "relu" => Ok(Box::new(ActivationFunction::relu())),
            "sigmoid" => Ok(Box::new(ActivationFunction::sigmoid())),
            "tanh" => Ok(Box::new(ActivationFunction::tanh())),
            "softmax" => Ok(Box::new(ActivationFunction::softmax())),
            "dot_product" => Ok(Box::new(VectorOp::dot_product())),
            "normalize" => Ok(Box::new(VectorOp::normalize())),
            "vector_add" => Ok(Box::new(VectorOp::add())),
            _ => Err(TribeError::InvalidOperation(format!("Unknown operation type: {}", self.operation_type))),
        }
    }

    /// Execute the tensor operation
    pub fn execute_operation(&self) -> TribeResult<Tensor> {
        let operation = self.get_operation()?;
        operation.execute(&self.input_tensors)
    }

    /// Check if task is expired
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.created_at);
        elapsed.num_seconds() > self.max_computation_time as i64
    }
}

/// Mining result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    pub task_id: String,
    pub miner_id: String,
    pub nonce: u64,
    pub hash: String,
    pub output_tensor: Tensor,
    pub computation_time: u64, // milliseconds
    pub timestamp: DateTime<Utc>,
    pub is_valid: bool,
}

impl MiningResult {
    pub fn new(
        task_id: String,
        miner_id: String,
        nonce: u64,
        hash: String,
        output_tensor: Tensor,
        computation_time: u64,
    ) -> Self {
        Self {
            task_id,
            miner_id,
            nonce,
            hash,
            output_tensor,
            computation_time,
            timestamp: Utc::now(),
            is_valid: false, // Will be validated by network
        }
    }

    pub fn validate(&mut self, task: &MiningTask) -> TribeResult<bool> {
        // Check if hash meets difficulty
        if !task.meets_difficulty(&self.hash) {
            self.is_valid = false;
            return Ok(false);
        }

        // Verify hash calculation
        let expected_hash = task.calculate_hash(self.nonce);
        if expected_hash != self.hash {
            self.is_valid = false;
            return Ok(false);
        }

        // Verify tensor computation
        let expected_output = task.execute_operation()?;
        if !self.tensors_approximately_equal(&self.output_tensor, &expected_output) {
            self.is_valid = false;
            return Ok(false);
        }

        self.is_valid = true;
        Ok(true)
    }

    fn tensors_approximately_equal(&self, a: &Tensor, b: &Tensor) -> bool {
        if a.shape != b.shape {
            return false;
        }

        let a_data = a.data.as_f32_vec().unwrap_or_default();
        let b_data = b.data.as_f32_vec().unwrap_or_default();

        if a_data.len() != b_data.len() {
            return false;
        }

        const EPSILON: f32 = 1e-6;
        a_data.iter().zip(b_data.iter()).all(|(x, y)| (x - y).abs() < EPSILON)
    }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerCapabilities {
    pub max_tensor_size: usize,
    pub supported_operations: Vec<String>,
    pub compute_power: u64, // Relative compute power score
    pub is_esp_device: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub tasks_completed: u64,
    pub total_rewards: u64,
    pub average_computation_time: u64,
    pub success_rate: f64,
    pub last_active: DateTime<Utc>,
}

impl AI3Miner {
    pub fn new(id: String, address: String, is_esp_device: bool) -> Self {
        let capabilities = if is_esp_device {
            MinerCapabilities {
                max_tensor_size: 1024, // Limited for ESP devices
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "relu".to_string(),
                    "sigmoid".to_string(),
                    "dot_product".to_string(),
                    "vector_add".to_string(),
                ],
                compute_power: 100, // Lower compute power for ESP
                is_esp_device: true,
            }
        } else {
            MinerCapabilities {
                max_tensor_size: 1_000_000, // Much larger for regular computers
                supported_operations: vec![
                    "matrix_multiply".to_string(),
                    "convolution".to_string(),
                    "relu".to_string(),
                    "sigmoid".to_string(),
                    "tanh".to_string(),
                    "softmax".to_string(),
                    "dot_product".to_string(),
                    "normalize".to_string(),
                    "vector_add".to_string(),
                ],
                compute_power: 1000,
                is_esp_device: false,
            }
        };

        Self {
            id,
            address,
            capabilities,
            current_task: None,
            latest_result: None,
            stats: MinerStats {
                tasks_completed: 0,
                total_rewards: 0,
                average_computation_time: 0,
                success_rate: 0.0,
                last_active: Utc::now(),
            },
        }
    }

    pub fn can_handle_task(&self, task: &MiningTask) -> bool {
        // Check if operation is supported
        if !self.capabilities.supported_operations.contains(&task.operation_type) {
            return false;
        }

        // Check tensor size constraints
        for tensor in &task.input_tensors {
            if tensor.shape.total_elements() > self.capabilities.max_tensor_size {
                return false;
            }
        }

        // ESP devices need ESP-compatible tensors
        if self.capabilities.is_esp_device {
            for tensor in &task.input_tensors {
                if !tensor.is_esp_compatible() {
                    return false;
                }
            }
        }

        true
    }

    pub fn assign_task(&mut self, task: MiningTask) -> TribeResult<()> {
        if !self.can_handle_task(&task) {
            return Err(TribeError::InvalidOperation("Miner cannot handle this task".to_string()));
        }

        self.current_task = Some(task);
        self.stats.last_active = Utc::now();
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

        // Simple mining: try random nonces
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let nonce = rng.gen_range(task.nonce_range.0..=task.nonce_range.1);

        let start_time = std::time::Instant::now();
        
        // Calculate hash
        let hash = task.calculate_hash(nonce);
        
        // Check if it meets difficulty
        if task.meets_difficulty(&hash) {
            // Execute the operation
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
        self.stats.tasks_completed += 1;
        
        // Update average computation time
        let total_time = self.stats.average_computation_time * (self.stats.tasks_completed - 1) + computation_time;
        self.stats.average_computation_time = total_time / self.stats.tasks_completed;

        // Update success rate
        let successful_tasks = (self.stats.success_rate * (self.stats.tasks_completed - 1) as f64) + if success { 1.0 } else { 0.0 };
        self.stats.success_rate = successful_tasks / self.stats.tasks_completed as f64;

        self.stats.last_active = Utc::now();
    }
}

/// Task distributor for managing mining tasks
#[derive(Debug)]
pub struct TaskDistributor {
    pub pending_tasks: HashMap<String, MiningTask>,
    pub active_tasks: HashMap<String, (MiningTask, String)>, // task_id -> (task, miner_id)
    pub completed_tasks: HashMap<String, MiningResult>,
}

impl TaskDistributor {
    pub fn new() -> Self {
        Self {
            pending_tasks: HashMap::new(),
            active_tasks: HashMap::new(),
            completed_tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: MiningTask) {
        self.pending_tasks.insert(task.id.clone(), task);
    }

    pub fn distribute(&mut self, task: MiningTask, miners: &[AI3Miner]) -> TribeResult<Vec<String>> {
        let mut assigned_miners = Vec::new();

        // Find suitable miners
        for miner in miners {
            if miner.can_handle_task(&task) && miner.current_task.is_none() {
                assigned_miners.push(miner.id.clone());
                
                // In a real implementation, you would send the task to the miner
                // For now, we just track it
                self.active_tasks.insert(task.id.clone(), (task.clone(), miner.id.clone()));
                break; // Assign to first available miner for now
            }
        }

        if assigned_miners.is_empty() {
            self.pending_tasks.insert(task.id.clone(), task);
        }

        Ok(assigned_miners)
    }

    pub fn submit_result(&mut self, result: MiningResult) -> TribeResult<()> {
        // Validate result
        if let Some((task, _)) = self.active_tasks.get(&result.task_id) {
            let mut validated_result = result;
            validated_result.validate(task)?;
            
            if validated_result.is_valid {
                self.completed_tasks.insert(result.task_id.clone(), validated_result);
                self.active_tasks.remove(&result.task_id);
            }
        }

        Ok(())
    }

    pub fn get_pending_tasks(&self) -> Vec<&MiningTask> {
        self.pending_tasks.values().collect()
    }

    pub fn get_completed_results(&self) -> Vec<&MiningResult> {
        self.completed_tasks.values().collect()
    }

    pub fn cleanup_expired_tasks(&mut self) {
        // Remove expired tasks
        self.pending_tasks.retain(|_, task| !task.is_expired());
        self.active_tasks.retain(|_, (task, _)| !task.is_expired());
    }
}

impl Default for TaskDistributor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::{Tensor, TensorShape};

    #[test]
    fn test_mining_task_creation() {
        let input = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input],
            4,
            100,
            300,
            "test_requester".to_string(),
        );

        assert_eq!(task.operation_type, "relu");
        assert_eq!(task.difficulty, 4);
        assert_eq!(task.reward, 100);
    }

    #[test]
    fn test_miner_capabilities() {
        let esp_miner = AI3Miner::new("esp1".to_string(), "addr1".to_string(), true);
        let regular_miner = AI3Miner::new("miner1".to_string(), "addr2".to_string(), false);

        assert!(esp_miner.capabilities.is_esp_device);
        assert!(!regular_miner.capabilities.is_esp_device);
        assert!(regular_miner.capabilities.max_tensor_size > esp_miner.capabilities.max_tensor_size);
    }

    #[test]
    fn test_task_assignment() {
        let mut miner = AI3Miner::new("miner1".to_string(), "addr1".to_string(), false);
        let input = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input],
            4,
            100,
            300,
            "test_requester".to_string(),
        );

        assert!(miner.can_handle_task(&task));
        assert!(miner.assign_task(task).is_ok());
        assert!(miner.current_task.is_some());
    }

    #[test]
    fn test_difficulty_check() {
        let input = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input],
            2, // 2 leading zeros required
            100,
            300,
            "test_requester".to_string(),
        );

        assert!(task.meets_difficulty("00abc123")); // 2 leading zeros
        assert!(!task.meets_difficulty("0abc123")); // Only 1 leading zero
        assert!(!task.meets_difficulty("abc123")); // No leading zeros
    }
} 