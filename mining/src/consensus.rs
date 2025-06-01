use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;
use tribechain_core::{TribeResult, TribeError, Block, Transaction};

/// Consensus engine for managing different consensus algorithms
#[derive(Debug)]
pub struct ConsensusEngine {
    pub consensus_type: ConsensusType,
    pub is_running: bool,
    pub stats: ConsensusStats,
    pub validators: Arc<RwLock<HashMap<String, ValidatorInfo>>>,
    pub current_epoch: u64,
    pub last_finalized_block: Option<String>,
}

/// Types of consensus algorithms supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusType {
    ProofOfWork,
    ProofOfStake,
    DelegatedProofOfStake,
    TensorProofOfWork, // AI3-specific consensus
}

/// Consensus statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStats {
    pub blocks_validated: u64,
    pub transactions_processed: u64,
    pub current_difficulty: u32,
    pub network_hash_rate: f64,
    pub average_block_time: f64,
    pub finality_time: f64, // seconds
    pub validator_count: usize,
    pub stake_participation: f64, // percentage for PoS
    pub last_block_time: Option<DateTime<Utc>>,
}

/// Validator information for PoS consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub stake: u64,
    pub voting_power: f64,
    pub is_active: bool,
    pub uptime: f64,
    pub slash_count: u32,
    pub last_activity: DateTime<Utc>,
    pub commission_rate: f64,
}

/// Block validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub gas_used: u64,
    pub execution_time: u64, // milliseconds
}

impl ConsensusEngine {
    pub fn new(consensus_type: ConsensusType) -> TribeResult<Self> {
        Ok(Self {
            consensus_type,
            is_running: false,
            stats: ConsensusStats::default(),
            validators: Arc::new(RwLock::new(HashMap::new())),
            current_epoch: 0,
            last_finalized_block: None,
        })
    }

    pub async fn start(&mut self) -> TribeResult<()> {
        if self.is_running {
            return Err(TribeError::InvalidOperation("Consensus already running".to_string()));
        }

        match self.consensus_type {
            ConsensusType::ProofOfWork => self.start_pow().await?,
            ConsensusType::ProofOfStake => self.start_pos().await?,
            ConsensusType::DelegatedProofOfStake => self.start_dpos().await?,
            ConsensusType::TensorProofOfWork => self.start_tensor_pow().await?,
        }

        self.is_running = true;
        Ok(())
    }

    pub async fn stop(&mut self) -> TribeResult<()> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        Ok(())
    }

    pub async fn validate_block(&self, block: &Block) -> TribeResult<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            gas_used: 0,
            execution_time: 0,
        };

        // Basic block validation
        if block.transactions.is_empty() {
            result.warnings.push("Block has no transactions".to_string());
        }

        // Validate block hash
        let calculated_hash = block.calculate_hash();
        if calculated_hash != block.hash {
            result.is_valid = false;
            result.errors.push("Invalid block hash".to_string());
        }

        // Consensus-specific validation
        match self.consensus_type {
            ConsensusType::ProofOfWork => {
                self.validate_pow_block(block, &mut result).await?;
            }
            ConsensusType::ProofOfStake => {
                self.validate_pos_block(block, &mut result).await?;
            }
            ConsensusType::DelegatedProofOfStake => {
                self.validate_dpos_block(block, &mut result).await?;
            }
            ConsensusType::TensorProofOfWork => {
                self.validate_tensor_pow_block(block, &mut result).await?;
            }
        }

        result.execution_time = start_time.elapsed().as_millis() as u64;
        Ok(result)
    }

    async fn start_pow(&mut self) -> TribeResult<()> {
        // Initialize Proof of Work consensus
        self.stats.current_difficulty = 4; // Starting difficulty
        Ok(())
    }

    async fn start_pos(&mut self) -> TribeResult<()> {
        // Initialize Proof of Stake consensus
        self.current_epoch = 1;
        Ok(())
    }

    async fn start_dpos(&mut self) -> TribeResult<()> {
        // Initialize Delegated Proof of Stake consensus
        self.current_epoch = 1;
        Ok(())
    }

    async fn start_tensor_pow(&mut self) -> TribeResult<()> {
        // Initialize Tensor Proof of Work consensus
        self.stats.current_difficulty = 3; // Lower difficulty for AI3
        Ok(())
    }

    async fn validate_pow_block(&self, block: &Block, result: &mut ValidationResult) -> TribeResult<()> {
        // Validate proof of work
        let target = "0".repeat(self.stats.current_difficulty as usize);
        if !block.hash.starts_with(&target) {
            result.is_valid = false;
            result.errors.push("Invalid proof of work".to_string());
        }
        Ok(())
    }

    async fn validate_pos_block(&self, block: &Block, result: &mut ValidationResult) -> TribeResult<()> {
        // Validate proof of stake
        let validators = self.validators.read().await;
        
        if let Some(validator) = validators.get(&block.miner) {
            if !validator.is_active {
                result.is_valid = false;
                result.errors.push("Block mined by inactive validator".to_string());
            }
            if validator.stake < 1000 { // Minimum stake requirement
                result.is_valid = false;
                result.errors.push("Validator has insufficient stake".to_string());
            }
        } else {
            result.is_valid = false;
            result.errors.push("Unknown validator".to_string());
        }
        
        Ok(())
    }

    async fn validate_dpos_block(&self, block: &Block, result: &mut ValidationResult) -> TribeResult<()> {
        // Validate delegated proof of stake
        let validators = self.validators.read().await;
        
        if let Some(validator) = validators.get(&block.miner) {
            if !validator.is_active {
                result.is_valid = false;
                result.errors.push("Block produced by inactive delegate".to_string());
            }
            
            // Check if it's the validator's turn to produce a block
            let slot_time = 3; // 3 seconds per slot
            let current_slot = (block.timestamp / slot_time) % validators.len() as u64;
            let validator_index = validators.keys().position(|k| k == &block.miner).unwrap_or(0);
            
            if current_slot != validator_index as u64 {
                result.is_valid = false;
                result.errors.push("Block produced out of turn".to_string());
            }
        } else {
            result.is_valid = false;
            result.errors.push("Unknown delegate".to_string());
        }
        
        Ok(())
    }

    async fn validate_tensor_pow_block(&self, block: &Block, result: &mut ValidationResult) -> TribeResult<()> {
        // Validate tensor proof of work
        if let Some(ai3_proof) = &block.ai3_proof {
            // Validate AI3 proof
            if ai3_proof.task_id.is_empty() {
                result.is_valid = false;
                result.errors.push("Invalid AI3 proof: empty task ID".to_string());
            }
            
            if ai3_proof.tensor_result.is_empty() {
                result.is_valid = false;
                result.errors.push("Invalid AI3 proof: empty tensor result".to_string());
            }
            
            // Reduced difficulty for AI3 mining
            let adjusted_difficulty = std::cmp::max(1, self.stats.current_difficulty - 1);
            let target = "0".repeat(adjusted_difficulty as usize);
            if !block.hash.starts_with(&target) {
                result.is_valid = false;
                result.errors.push("Invalid tensor proof of work".to_string());
            }
        } else {
            // Regular PoW validation
            let target = "0".repeat(self.stats.current_difficulty as usize);
            if !block.hash.starts_with(&target) {
                result.is_valid = false;
                result.errors.push("Invalid proof of work".to_string());
            }
        }
        
        Ok(())
    }

    pub async fn add_validator(&mut self, validator: ValidatorInfo) -> TribeResult<()> {
        let mut validators = self.validators.write().await;
        validators.insert(validator.address.clone(), validator);
        self.stats.validator_count = validators.len();
        Ok(())
    }

    pub async fn remove_validator(&mut self, address: &str) -> TribeResult<()> {
        let mut validators = self.validators.write().await;
        validators.remove(address);
        self.stats.validator_count = validators.len();
        Ok(())
    }

    pub fn get_hash_rate(&self) -> f64 {
        self.stats.network_hash_rate
    }

    pub fn get_stats(&self) -> ConsensusStats {
        self.stats.clone()
    }

    pub fn update_difficulty(&mut self, new_difficulty: u32) {
        self.stats.current_difficulty = new_difficulty;
    }

    pub fn update_hash_rate(&mut self, hash_rate: f64) {
        self.stats.network_hash_rate = hash_rate;
    }

    pub async fn finalize_block(&mut self, block_hash: String) -> TribeResult<()> {
        self.last_finalized_block = Some(block_hash);
        self.stats.blocks_validated += 1;
        Ok(())
    }
}

impl Default for ConsensusStats {
    fn default() -> Self {
        Self {
            blocks_validated: 0,
            transactions_processed: 0,
            current_difficulty: 4,
            network_hash_rate: 0.0,
            average_block_time: 600.0, // 10 minutes
            finality_time: 60.0, // 1 minute
            validator_count: 0,
            stake_participation: 0.0,
            last_block_time: None,
        }
    }
}

impl ValidatorInfo {
    pub fn new(address: String, stake: u64) -> Self {
        Self {
            address,
            stake,
            voting_power: 0.0,
            is_active: true,
            uptime: 100.0,
            slash_count: 0,
            last_activity: Utc::now(),
            commission_rate: 0.05, // 5% default commission
        }
    }

    pub fn calculate_voting_power(&mut self, total_stake: u64) {
        if total_stake > 0 {
            self.voting_power = (self.stake as f64 / total_stake as f64) * 100.0;
        }
    }

    pub fn slash(&mut self, percentage: f64) {
        let slash_amount = (self.stake as f64 * percentage / 100.0) as u64;
        self.stake = self.stake.saturating_sub(slash_amount);
        self.slash_count += 1;
        
        if self.slash_count >= 3 {
            self.is_active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_engine_creation() {
        let engine = ConsensusEngine::new(ConsensusType::ProofOfWork).unwrap();
        assert!(!engine.is_running);
        assert_eq!(engine.current_epoch, 0);
    }

    #[tokio::test]
    async fn test_consensus_start_stop() {
        let mut engine = ConsensusEngine::new(ConsensusType::ProofOfWork).unwrap();
        
        engine.start().await.unwrap();
        assert!(engine.is_running);
        
        engine.stop().await.unwrap();
        assert!(!engine.is_running);
    }

    #[tokio::test]
    async fn test_validator_management() {
        let mut engine = ConsensusEngine::new(ConsensusType::ProofOfStake).unwrap();
        let validator = ValidatorInfo::new("validator1".to_string(), 1000);
        
        engine.add_validator(validator).await.unwrap();
        assert_eq!(engine.stats.validator_count, 1);
        
        engine.remove_validator("validator1").await.unwrap();
        assert_eq!(engine.stats.validator_count, 0);
    }
} 