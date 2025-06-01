pub mod miner;
pub mod consensus;
pub mod difficulty;
pub mod pool;
pub mod proof_of_work;
pub mod ai3_mining;

// Re-export main types
pub use miner::{Miner, MinerStats, MinerCapabilities};
pub use consensus::{ConsensusEngine, ConsensusType, ConsensusStats};
pub use difficulty::{DifficultyAdjuster, DifficultyAdjustment};
pub use pool::{MiningPool, PoolStats, MiningShare};
pub use proof_of_work::{ProofOfWork, WorkProof, AI3WorkProof, MiningWork};
pub use ai3_mining::{AI3Miner, AI3MiningResult, AI3Proof, AI3MiningPool};

// Re-export ai3-lib mining types for convenience
pub use ai3_lib::mining::{
    MiningTask as LibMiningTask,
    MiningResult as LibMiningResult,
    TaskDistributor,
    MinerCapabilities as LibMinerCapabilities,
    MinerStats as LibMinerStats,
};

use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};

/// Mining engine that coordinates all mining activities
#[derive(Debug)]
pub struct MiningEngine {
    pub consensus: consensus::ConsensusEngine,
    pub pool: pool::MiningPool,
    pub difficulty: difficulty::DifficultyAdjuster,
    pub proof_of_work: proof_of_work::ProofOfWork,
    pub ai3_mining: Option<ai3_mining::AI3MiningPool>,
    pub is_mining: bool,
}

impl MiningEngine {
    pub fn new(consensus_type: consensus::ConsensusType) -> TribeResult<Self> {
        Ok(Self {
            consensus: consensus::ConsensusEngine::new(consensus_type)?,
            pool: pool::MiningPool::new("default_pool".to_string(), "Default Pool".to_string(), pool::PoolConfig::default()),
            difficulty: difficulty::DifficultyAdjuster::default(),
            proof_of_work: proof_of_work::ProofOfWork::new(4, 600),
            ai3_mining: None,
            is_mining: false,
        })
    }

    pub fn with_ai3_mining(mut self, pool_id: String) -> Self {
        self.ai3_mining = Some(ai3_mining::AI3MiningPool::new(pool_id));
        self
    }

    pub async fn start_mining(&mut self) -> TribeResult<()> {
        if self.is_mining {
            return Err(TribeError::InvalidOperation("Mining already started".to_string()));
        }
        
        self.consensus.start().await?;
        self.is_mining = true;
        Ok(())
    }

    pub async fn stop_mining(&mut self) -> TribeResult<()> {
        if !self.is_mining {
            return Ok(());
        }
        
        self.consensus.stop().await?;
        self.is_mining = false;
        Ok(())
    }

    pub async fn add_miner(&mut self, miner: miner::Miner) -> TribeResult<()> {
        self.pool.add_miner(miner).await
    }

    pub async fn remove_miner(&mut self, miner_id: &str) -> TribeResult<()> {
        self.pool.remove_miner(miner_id).await
    }

    pub async fn add_ai3_miner(&mut self, miner: ai3_mining::AI3Miner) -> TribeResult<()> {
        if let Some(ai3_pool) = &mut self.ai3_mining {
            ai3_pool.add_miner(miner);
            Ok(())
        } else {
            Err(TribeError::InvalidOperation("AI3 mining not enabled".to_string()))
        }
    }

    /// Create AI3 mining task from block template
    pub async fn create_ai3_mining_task(
        &mut self,
        block: tribechain_core::Block,
        operation_type: String,
        difficulty: u64,
    ) -> TribeResult<String> {
        if let Some(ai3_pool) = &mut self.ai3_mining {
            // Use the first available miner to create the task
            if let Some(miner) = ai3_pool.miners.values_mut().next() {
                miner.create_mining_task(&block, operation_type, difficulty).await
            } else {
                Err(TribeError::InvalidOperation("No AI3 miners available".to_string()))
            }
        } else {
            Err(TribeError::InvalidOperation("AI3 mining not enabled".to_string()))
        }
    }

    pub fn get_stats(&self) -> MiningEngineStats {
        MiningEngineStats {
            is_mining: self.is_mining,
            total_miners: self.pool.get_stats().total_miners,
            total_hash_rate: self.pool.get_stats().total_hash_rate,
            current_difficulty: self.difficulty.get_current_difficulty(),
            consensus_stats: self.consensus.get_stats(),
            proof_of_work_stats: self.proof_of_work.get_mining_stats(),
            ai3_pool_stats: self.ai3_mining.as_ref().map(|pool| pool.get_pool_stats()),
        }
    }

    /// Create mining work for proof-of-work
    pub fn create_pow_work(&self, block: tribechain_core::Block) -> proof_of_work::MiningWork {
        self.proof_of_work.create_work(block, None)
    }

    /// Mine a block using the configured mining engine
    pub async fn mine_block(
        &mut self,
        mut work: proof_of_work::MiningWork,
        miner_id: String,
    ) -> TribeResult<Option<proof_of_work::WorkProof>> {
        // Try AI3 mining first if available
        if let Some(ai3_pool) = &mut self.ai3_mining {
            if let Some(ai3_miner) = ai3_pool.miners.get_mut(&miner_id) {
                // Create AI3 task from block
                let task_id = ai3_miner.create_mining_task(
                    &work.block_template,
                    "matrix_multiply".to_string(),
                    self.difficulty.get_current_difficulty() as u64,
                ).await?;

                // Try AI3 mining step
                if let Some(ai3_result) = ai3_miner.mine_step(&task_id).await? {
                    // Convert AI3 result to work proof
                    let ai3_work_proof = proof_of_work::AI3WorkProof {
                        task_id: ai3_result.task_id,
                        tensor_result: ai3_result.tensor_result,
                        computation_hash: ai3_result.ai3_proof.computation_hash,
                        verification_nonce: 0, // Would be set from AI3 result
                    };

                    let work_proof = proof_of_work::WorkProof {
                        block_hash: ai3_result.block_hash,
                        nonce: 0, // Would be set from AI3 result
                        timestamp: ai3_result.timestamp,
                        difficulty: self.difficulty.get_current_difficulty(),
                        miner_id: miner_id.clone(),
                        ai3_proof: Some(ai3_work_proof),
                    };

                    return Ok(Some(work_proof));
                }
            }
        }

        // Fallback to regular proof-of-work mining
        self.proof_of_work.mine_block(&mut work, miner_id, None)
    }
}

/// Mining engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningEngineStats {
    pub is_mining: bool,
    pub total_miners: usize,
    pub total_hash_rate: f64,
    pub current_difficulty: u32,
    pub consensus_stats: consensus::ConsensusStats,
    pub proof_of_work_stats: proof_of_work::MiningStats,
    pub ai3_pool_stats: Option<ai3_mining::AI3PoolStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mining_engine_creation() {
        let engine = MiningEngine::new(consensus::ConsensusType::ProofOfWork).unwrap();
        assert!(!engine.is_mining);
        assert!(engine.ai3_mining.is_none());
    }

    #[tokio::test]
    async fn test_mining_engine_with_ai3() {
        let engine = MiningEngine::new(consensus::ConsensusType::TensorProofOfWork)
            .unwrap()
            .with_ai3_mining("ai3_pool".to_string());
        
        assert!(engine.ai3_mining.is_some());
    }

    #[tokio::test]
    async fn test_mining_start_stop() {
        let mut engine = MiningEngine::new(consensus::ConsensusType::ProofOfWork).unwrap();
        
        engine.start_mining().await.unwrap();
        assert!(engine.is_mining);
        
        engine.stop_mining().await.unwrap();
        assert!(!engine.is_mining);
    }

    #[tokio::test]
    async fn test_add_ai3_miner() {
        let mut engine = MiningEngine::new(consensus::ConsensusType::TensorProofOfWork)
            .unwrap()
            .with_ai3_mining("ai3_pool".to_string());
        
        let ai3_miner = ai3_mining::AI3Miner::new("test_ai3_miner".to_string());
        engine.add_ai3_miner(ai3_miner).await.unwrap();
        
        let stats = engine.get_stats();
        assert!(stats.ai3_pool_stats.is_some());
        assert_eq!(stats.ai3_pool_stats.unwrap().total_miners, 1);
    }
} 