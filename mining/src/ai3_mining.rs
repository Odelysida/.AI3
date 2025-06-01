use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;
use tribechain_core::{TribeResult, TribeError, Block};

// Import from ai3-lib mining module
use ai3_lib::mining::{
    AI3Miner as LibAI3Miner, 
    MinerCapabilities, 
    MinerStats,
    MiningTask, 
    MiningResult as LibMiningResult,
    TaskDistributor
};
use ai3_lib::{
    Tensor, TensorShape, TensorData, AI3Engine,
    ESP32Miner, ESP8266Miner, ESPMiningConfig
};

/// AI3 Mining coordinator that integrates with the AI3 library
#[derive(Debug)]
pub struct AI3Miner {
    pub id: String,
    pub lib_miner: LibAI3Miner,
    pub ai3_engine: AI3Engine,
    pub active_tasks: Arc<RwLock<HashMap<String, MiningTask>>>,
    pub completed_results: Arc<RwLock<HashMap<String, AI3MiningResult>>>,
    pub esp_miners: HashMap<String, ESPMinerWrapper>,
    pub stats: AI3MiningStats,
}

/// Wrapper for ESP miners
#[derive(Debug)]
pub enum ESPMinerWrapper {
    ESP32(ESP32Miner),
    ESP8266(ESP8266Miner),
}

/// AI3 mining result with blockchain integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3MiningResult {
    pub task_id: String,
    pub miner_id: String,
    pub tensor_result: Tensor,
    pub computation_time: u64,
    pub block_height: u64,
    pub block_hash: String,
    pub ai3_proof: AI3Proof,
    pub timestamp: DateTime<Utc>,
    pub is_valid: bool,
}

/// AI3 proof structure for blockchain validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3Proof {
    pub operation_type: String,
    pub input_hash: String,
    pub output_hash: String,
    pub computation_hash: String,
    pub difficulty_met: bool,
    pub verification_data: Vec<u8>,
}

/// AI3 mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3MiningStats {
    pub total_tasks_processed: u64,
    pub successful_computations: u64,
    pub failed_computations: u64,
    pub average_computation_time: f64,
    pub total_tensor_operations: u64,
    pub esp_miners_active: usize,
    pub current_hash_rate: f64,
    pub ai3_blocks_mined: u64,
}

impl AI3Miner {
    pub fn new(id: String) -> Self {
        let lib_miner = LibAI3Miner::new(
            id.clone(),
            format!("miner_address_{}", id),
            false // Not an ESP device by default
        );

        Self {
            id: id.clone(),
            lib_miner,
            ai3_engine: AI3Engine::new(),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_results: Arc::new(RwLock::new(HashMap::new())),
            esp_miners: HashMap::new(),
            stats: AI3MiningStats::default(),
        }
    }

    /// Add ESP32 miner to the AI3 mining pool
    pub async fn add_esp32_miner(&mut self, config: ESPMiningConfig) -> TribeResult<String> {
        let miner_id = format!("esp32_{}", uuid::Uuid::new_v4());
        let esp32_miner = ESP32Miner::new(
            miner_id.clone(),
            format!("esp32_address_{}", miner_id),
            config
        );
        
        self.esp_miners.insert(miner_id.clone(), ESPMinerWrapper::ESP32(esp32_miner));
        self.stats.esp_miners_active += 1;
        
        Ok(miner_id)
    }

    /// Add ESP8266 miner to the AI3 mining pool
    pub async fn add_esp8266_miner(&mut self, config: ESPMiningConfig) -> TribeResult<String> {
        let miner_id = format!("esp8266_{}", uuid::Uuid::new_v4());
        let esp8266_miner = ESP8266Miner::new(
            miner_id.clone(),
            format!("esp8266_address_{}", miner_id),
            config
        );
        
        self.esp_miners.insert(miner_id.clone(), ESPMinerWrapper::ESP8266(esp8266_miner));
        self.stats.esp_miners_active += 1;
        
        Ok(miner_id)
    }

    /// Create AI3 mining task from block template
    pub async fn create_mining_task(
        &mut self,
        block: &Block,
        operation_type: String,
        difficulty: u64,
    ) -> TribeResult<String> {
        // Create input tensors based on block data
        let input_tensors = self.block_to_tensors(block)?;
        
        // Create AI3 mining task using ai3-lib
        let task = MiningTask::new(
            operation_type.clone(),
            input_tensors,
            difficulty,
            50_000_000, // 50 TRIBE reward
            300, // 5 minutes max computation time
            block.miner.clone(),
        );

        let task_id = task.id.clone();
        
        // Store task
        let mut active_tasks = self.active_tasks.write().await;
        active_tasks.insert(task_id.clone(), task);
        
        Ok(task_id)
    }

    /// Convert block data to tensors for AI3 processing
    fn block_to_tensors(&self, block: &Block) -> TribeResult<Vec<Tensor>> {
        let mut tensors = Vec::new();
        
        // Convert block hash to tensor
        let hash_bytes: Vec<u8> = hex::decode(&block.hash)
            .map_err(|_| TribeError::InvalidOperation("Invalid block hash".to_string()))?;
        
        let hash_floats: Vec<f32> = hash_bytes.iter().map(|&b| b as f32 / 255.0).collect();
        let hash_tensor = Tensor::new(
            TensorShape::new(vec![hash_floats.len()]),
            TensorData::F32(hash_floats),
        )?;
        tensors.push(hash_tensor);

        // Convert transaction data to tensor if available
        if !block.transactions.is_empty() {
            let tx_data: Vec<f32> = block.transactions
                .iter()
                .take(16) // Limit to first 16 transactions
                .flat_map(|tx| {
                    vec![
                        tx.fee as f32,
                        tx.timestamp as f32,
                        tx.nonce as f32,
                    ]
                })
                .collect();
            
            if !tx_data.is_empty() {
                let tx_tensor = Tensor::new(
                    TensorShape::new(vec![tx_data.len()]),
                    TensorData::F32(tx_data),
                )?;
                tensors.push(tx_tensor);
            }
        }

        // Add block metadata tensor
        let metadata = vec![
            block.height as f32,
            block.timestamp as f32,
            block.nonce as f32,
            block.difficulty as f32,
        ];
        
        let metadata_tensor = Tensor::new(
            TensorShape::new(vec![metadata.len()]),
            TensorData::F32(metadata),
        )?;
        tensors.push(metadata_tensor);

        Ok(tensors)
    }

    /// Perform AI3 mining step using ai3-lib miner
    pub async fn mine_step(&mut self, task_id: &str) -> TribeResult<Option<AI3MiningResult>> {
        let task = {
            let active_tasks = self.active_tasks.read().await;
            match active_tasks.get(task_id) {
                Some(task) => task.clone(),
                None => return Ok(None),
            }
        };

        // Check if the lib miner can handle this task
        if !self.lib_miner.can_handle_task(&task) {
            return Ok(None);
        }

        // Assign task to lib miner
        let mut lib_miner = self.lib_miner.clone();
        lib_miner.assign_task(task.clone())?;

        // Try mining step using ai3-lib
        if let Some(lib_result) = lib_miner.mine_step()? {
            // Convert lib result to blockchain-compatible result
            let ai3_result = self.convert_to_ai3_result(lib_result, self.id.clone()).await?;
            
            // Store completed result
            let mut completed_results = self.completed_results.write().await;
            completed_results.insert(task_id.to_string(), ai3_result.clone());
            
            // Remove from active tasks
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(task_id);
            
            // Update stats
            self.update_stats(&ai3_result).await;
            
            return Ok(Some(ai3_result));
        }

        // Try ESP miners if lib miner didn't find solution
        self.try_esp_mining(&task).await
    }

    /// Try mining with ESP devices
    async fn try_esp_mining(&mut self, task: &MiningTask) -> TribeResult<Option<AI3MiningResult>> {
        for (miner_id, esp_miner) in &mut self.esp_miners {
            match esp_miner {
                ESPMinerWrapper::ESP32(esp32) => {
                    // Check if ESP32 can handle the task
                    if esp32.base_miner.can_handle_task(task) {
                        // Assign task to the base miner
                        esp32.base_miner.assign_task(task.clone())?;
                        
                        // Try mining step
                        if let Some(result) = esp32.mine_step()? {
                            return Ok(Some(self.convert_to_ai3_result(result, miner_id.clone()).await?));
                        }
                    }
                }
                ESPMinerWrapper::ESP8266(esp8266) => {
                    // Check if ESP8266 can handle the task
                    if esp8266.esp32_miner.base_miner.can_handle_task(task) {
                        // Assign task to the base miner
                        esp8266.esp32_miner.base_miner.assign_task(task.clone())?;
                        
                        // Try mining step
                        if let Some(result) = esp8266.mine_step()? {
                            return Ok(Some(self.convert_to_ai3_result(result, miner_id.clone()).await?));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Convert ai3-lib mining result to blockchain-compatible result
    async fn convert_to_ai3_result(
        &mut self,
        lib_result: LibMiningResult,
        miner_id: String,
    ) -> TribeResult<AI3MiningResult> {
        let ai3_proof = AI3Proof {
            operation_type: "tensor_operation".to_string(),
            input_hash: lib_result.hash.clone(),
            output_hash: lib_result.output_tensor.calculate_hash(),
            computation_hash: format!("{}_{}", lib_result.nonce, lib_result.computation_time),
            difficulty_met: lib_result.is_valid,
            verification_data: self.create_verification_data(&lib_result)?,
        };

        Ok(AI3MiningResult {
            task_id: lib_result.task_id,
            miner_id,
            tensor_result: lib_result.output_tensor,
            computation_time: lib_result.computation_time,
            block_height: 0, // Will be set by mining engine
            block_hash: String::new(), // Will be set by mining engine
            ai3_proof,
            timestamp: lib_result.timestamp,
            is_valid: lib_result.is_valid,
        })
    }

    /// Create verification data for AI3 proof
    fn create_verification_data(&self, result: &LibMiningResult) -> TribeResult<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(result.task_id.as_bytes());
        hasher.update(&result.nonce.to_le_bytes());
        hasher.update(result.output_tensor.calculate_hash().as_bytes());
        Ok(hasher.finalize().to_vec())
    }

    /// Update mining statistics
    async fn update_stats(&mut self, result: &AI3MiningResult) {
        self.stats.total_tasks_processed += 1;
        
        if result.is_valid {
            self.stats.successful_computations += 1;
            self.stats.ai3_blocks_mined += 1;
        } else {
            self.stats.failed_computations += 1;
        }

        // Update average computation time
        let total_time = self.stats.average_computation_time * (self.stats.total_tasks_processed - 1) as f64 
            + result.computation_time as f64;
        self.stats.average_computation_time = total_time / self.stats.total_tasks_processed as f64;

        self.stats.total_tensor_operations += 1;
    }

    /// Validate AI3 mining result
    pub async fn validate_result(&self, result: &AI3MiningResult) -> TribeResult<bool> {
        let task = {
            let active_tasks = self.active_tasks.read().await;
            match active_tasks.get(&result.task_id) {
                Some(task) => task.clone(),
                None => {
                    // Check completed tasks
                    return Ok(result.is_valid);
                }
            }
        };

        self.validate_ai3_proof(&result.ai3_proof, &task).await
    }

    /// Validate AI3 proof against task
    async fn validate_ai3_proof(&self, proof: &AI3Proof, task: &MiningTask) -> TribeResult<bool> {
        // Verify computation hash meets difficulty
        if !task.meets_difficulty(&proof.computation_hash) {
            return Ok(false);
        }

        // Additional AI3-specific validations would go here
        Ok(proof.difficulty_met)
    }

    /// Get miner capabilities from ai3-lib
    pub fn get_capabilities(&self) -> &MinerCapabilities {
        &self.lib_miner.capabilities
    }

    /// Get miner stats from ai3-lib
    pub fn get_miner_stats(&self) -> &MinerStats {
        &self.lib_miner.stats
    }

    /// Get AI3 mining stats
    pub fn get_stats(&self) -> AI3MiningStats {
        self.stats.clone()
    }

    pub async fn get_active_tasks_count(&self) -> usize {
        self.active_tasks.read().await.len()
    }

    pub async fn get_completed_results_count(&self) -> usize {
        self.completed_results.read().await.len()
    }

    pub async fn cleanup_expired_tasks(&mut self) {
        let mut active_tasks = self.active_tasks.write().await;
        active_tasks.retain(|_, task| !task.is_expired());
    }
}

impl Default for AI3MiningStats {
    fn default() -> Self {
        Self {
            total_tasks_processed: 0,
            successful_computations: 0,
            failed_computations: 0,
            average_computation_time: 0.0,
            total_tensor_operations: 0,
            esp_miners_active: 0,
            current_hash_rate: 0.0,
            ai3_blocks_mined: 0,
        }
    }
}

/// AI3 Mining Pool that uses ai3-lib TaskDistributor
#[derive(Debug)]
pub struct AI3MiningPool {
    pub pool_id: String,
    pub miners: HashMap<String, AI3Miner>,
    pub task_distributor: TaskDistributor,
    pub pool_stats: AI3PoolStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3PoolStats {
    pub total_miners: usize,
    pub active_miners: usize,
    pub total_hash_rate: f64,
    pub blocks_mined: u64,
    pub total_rewards: u64,
    pub average_block_time: f64,
}

impl AI3MiningPool {
    pub fn new(pool_id: String) -> Self {
        Self {
            pool_id,
            miners: HashMap::new(),
            task_distributor: TaskDistributor::new(),
            pool_stats: AI3PoolStats::default(),
        }
    }

    pub fn add_miner(&mut self, miner: AI3Miner) {
        self.miners.insert(miner.id.clone(), miner);
        self.pool_stats.total_miners += 1;
        self.pool_stats.active_miners += 1;
    }

    pub fn remove_miner(&mut self, miner_id: &str) {
        if self.miners.remove(miner_id).is_some() {
            self.pool_stats.total_miners = self.pool_stats.total_miners.saturating_sub(1);
            self.pool_stats.active_miners = self.pool_stats.active_miners.saturating_sub(1);
        }
    }

    /// Distribute task using ai3-lib TaskDistributor
    pub async fn distribute_task(&mut self, task: MiningTask) -> TribeResult<Vec<String>> {
        let lib_miners: Vec<_> = self.miners.values().map(|m| &m.lib_miner).collect();
        self.task_distributor.distribute(task, &lib_miners)
    }

    pub fn get_pool_stats(&self) -> AI3PoolStats {
        self.pool_stats.clone()
    }
}

impl Default for AI3PoolStats {
    fn default() -> Self {
        Self {
            total_miners: 0,
            active_miners: 0,
            total_hash_rate: 0.0,
            blocks_mined: 0,
            total_rewards: 0,
            average_block_time: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai3_miner_creation() {
        let miner = AI3Miner::new("test_miner".to_string());
        assert_eq!(miner.id, "test_miner");
        assert_eq!(miner.lib_miner.id, "test_miner");
    }

    #[tokio::test]
    async fn test_ai3_mining_pool() {
        let mut pool = AI3MiningPool::new("test_pool".to_string());
        let miner = AI3Miner::new("test_miner".to_string());
        
        pool.add_miner(miner);
        assert_eq!(pool.get_pool_stats().total_miners, 1);
        
        pool.remove_miner("test_miner");
        assert_eq!(pool.get_pool_stats().total_miners, 0);
    }

    #[test]
    fn test_ai3_proof_creation() {
        let proof = AI3Proof {
            operation_type: "matrix_multiply".to_string(),
            input_hash: "test_input".to_string(),
            output_hash: "test_output".to_string(),
            computation_hash: "test_computation".to_string(),
            difficulty_met: true,
            verification_data: vec![1, 2, 3, 4],
        };
        
        assert!(proof.difficulty_met);
        assert_eq!(proof.operation_type, "matrix_multiply");
    }
} 