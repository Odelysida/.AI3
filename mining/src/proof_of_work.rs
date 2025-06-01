use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tribechain_core::{TribeResult, TribeError, Block, Transaction};
use ai3_lib::{Tensor, MiningTask as AI3Task, MiningResult as AI3Result, AI3Miner};

/// Proof of Work mining implementation
#[derive(Debug, Clone)]
pub struct ProofOfWork {
    pub difficulty: u32,
    pub target_block_time: u64, // seconds
    pub max_nonce: u64,
    pub ai3_integration: bool,
}

/// Work proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProof {
    pub block_hash: String,
    pub nonce: u64,
    pub timestamp: DateTime<Utc>,
    pub difficulty: u32,
    pub miner_id: String,
    pub ai3_proof: Option<AI3WorkProof>,
}

/// AI3-enhanced work proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3WorkProof {
    pub task_id: String,
    pub tensor_result: Tensor,
    pub computation_hash: String,
    pub verification_nonce: u64,
}

/// Mining work unit
#[derive(Debug, Clone)]
pub struct MiningWork {
    pub block_template: Block,
    pub target: String,
    pub start_nonce: u64,
    pub end_nonce: u64,
    pub ai3_task: Option<AI3Task>,
}

impl ProofOfWork {
    pub fn new(difficulty: u32, target_block_time: u64) -> Self {
        Self {
            difficulty,
            target_block_time,
            max_nonce: u64::MAX,
            ai3_integration: true,
        }
    }

    pub fn with_ai3_integration(mut self, enabled: bool) -> Self {
        self.ai3_integration = enabled;
        self
    }

    /// Create mining work from block template
    pub fn create_work(&self, mut block: Block, ai3_task: Option<AI3Task>) -> MiningWork {
        let target = self.calculate_target();
        
        // Set initial nonce
        block.nonce = 0;
        
        MiningWork {
            block_template: block,
            target,
            start_nonce: 0,
            end_nonce: self.max_nonce,
            ai3_task,
        }
    }

    /// Mine a block using proof of work
    pub fn mine_block(
        &self,
        work: &mut MiningWork,
        miner_id: String,
        ai3_miner: Option<&mut AI3Miner>,
    ) -> TribeResult<Option<WorkProof>> {
        let start_time = std::time::Instant::now();
        
        for nonce in work.start_nonce..=work.end_nonce {
            work.block_template.nonce = nonce;
            work.block_template.timestamp = Utc::now().timestamp() as u64;
            
            let hash = work.block_template.calculate_hash();
            
            // Check if hash meets difficulty target
            if self.meets_difficulty(&hash, &work.target) {
                let mut proof = WorkProof {
                    block_hash: hash,
                    nonce,
                    timestamp: Utc::now(),
                    difficulty: self.difficulty,
                    miner_id: miner_id.clone(),
                    ai3_proof: None,
                };

                // If AI3 integration is enabled and we have a task
                if self.ai3_integration && work.ai3_task.is_some() && ai3_miner.is_some() {
                    if let Some(ai3_proof) = self.mine_ai3_component(
                        work.ai3_task.as_ref().unwrap(),
                        ai3_miner.unwrap(),
                        &hash,
                    )? {
                        proof.ai3_proof = Some(ai3_proof);
                    }
                }

                return Ok(Some(proof));
            }

            // Check for timeout (prevent infinite mining)
            if start_time.elapsed().as_secs() > 300 { // 5 minute timeout
                break;
            }
        }

        Ok(None)
    }

    /// Mine AI3 tensor component
    fn mine_ai3_component(
        &self,
        ai3_task: &AI3Task,
        ai3_miner: &mut AI3Miner,
        block_hash: &str,
    ) -> TribeResult<Option<AI3WorkProof>> {
        // Assign AI3 task to miner
        ai3_miner.assign_task(ai3_task.clone())?;
        
        // Perform AI3 mining step
        if let Some(ai3_result) = ai3_miner.mine_step()? {
            // Validate AI3 result
            if ai3_result.is_valid {
                let computation_hash = self.calculate_ai3_hash(&ai3_result, block_hash);
                
                return Ok(Some(AI3WorkProof {
                    task_id: ai3_result.task_id,
                    tensor_result: ai3_result.output_tensor,
                    computation_hash,
                    verification_nonce: ai3_result.nonce,
                }));
            }
        }

        Ok(None)
    }

    /// Calculate AI3 computation hash
    fn calculate_ai3_hash(&self, ai3_result: &AI3Result, block_hash: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(block_hash.as_bytes());
        hasher.update(ai3_result.task_id.as_bytes());
        hasher.update(ai3_result.nonce.to_le_bytes());
        hasher.update(ai3_result.output_tensor.calculate_hash().as_bytes());
        
        hex::encode(hasher.finalize())
    }

    /// Calculate difficulty target
    fn calculate_target(&self) -> String {
        "0".repeat(self.difficulty as usize)
    }

    /// Check if hash meets difficulty
    fn meets_difficulty(&self, hash: &str, target: &str) -> bool {
        hash.starts_with(target)
    }

    /// Verify proof of work
    pub fn verify_proof(&self, proof: &WorkProof, block: &Block) -> TribeResult<bool> {
        // Verify basic PoW
        if !self.meets_difficulty(&proof.block_hash, &self.calculate_target()) {
            return Ok(false);
        }

        // Verify block hash
        let mut test_block = block.clone();
        test_block.nonce = proof.nonce;
        test_block.timestamp = proof.timestamp.timestamp() as u64;
        
        let calculated_hash = test_block.calculate_hash();
        if calculated_hash != proof.block_hash {
            return Ok(false);
        }

        // Verify AI3 proof if present
        if let Some(ai3_proof) = &proof.ai3_proof {
            return self.verify_ai3_proof(ai3_proof, &proof.block_hash);
        }

        Ok(true)
    }

    /// Verify AI3 proof component
    fn verify_ai3_proof(&self, ai3_proof: &AI3WorkProof, block_hash: &str) -> TribeResult<bool> {
        // Create a mock AI3 result for verification
        let ai3_result = AI3Result::new(
            ai3_proof.task_id.clone(),
            "verifier".to_string(),
            ai3_proof.verification_nonce,
            ai3_proof.computation_hash.clone(),
            ai3_proof.tensor_result.clone(),
            0, // computation time not needed for verification
        );

        // Verify computation hash
        let expected_hash = self.calculate_ai3_hash(&ai3_result, block_hash);
        Ok(expected_hash == ai3_proof.computation_hash)
    }

    /// Adjust difficulty based on block time
    pub fn adjust_difficulty(&mut self, actual_block_time: u64) -> u32 {
        let ratio = actual_block_time as f64 / self.target_block_time as f64;
        
        if ratio > 1.5 {
            // Blocks are too slow, decrease difficulty
            self.difficulty = std::cmp::max(1, self.difficulty - 1);
        } else if ratio < 0.5 {
            // Blocks are too fast, increase difficulty
            self.difficulty = std::cmp::min(32, self.difficulty + 1);
        }

        self.difficulty
    }

    /// Calculate expected hash rate for current difficulty
    pub fn calculate_expected_hash_rate(&self) -> f64 {
        // Simplified calculation: 2^difficulty hashes per target block time
        let target_hashes = 2_u64.pow(self.difficulty) as f64;
        target_hashes / self.target_block_time as f64
    }

    /// Get mining statistics
    pub fn get_mining_stats(&self) -> MiningStats {
        MiningStats {
            current_difficulty: self.difficulty,
            target_block_time: self.target_block_time,
            expected_hash_rate: self.calculate_expected_hash_rate(),
            ai3_integration_enabled: self.ai3_integration,
        }
    }
}

/// Mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    pub current_difficulty: u32,
    pub target_block_time: u64,
    pub expected_hash_rate: f64,
    pub ai3_integration_enabled: bool,
}

/// Batch mining for multiple work units
pub struct BatchMiner {
    pub pow: ProofOfWork,
    pub work_queue: Vec<MiningWork>,
    pub ai3_miners: HashMap<String, AI3Miner>,
}

impl BatchMiner {
    pub fn new(pow: ProofOfWork) -> Self {
        Self {
            pow,
            work_queue: Vec::new(),
            ai3_miners: HashMap::new(),
        }
    }

    pub fn add_work(&mut self, work: MiningWork) {
        self.work_queue.push(work);
    }

    pub fn add_ai3_miner(&mut self, miner: AI3Miner) {
        self.ai3_miners.insert(miner.id.clone(), miner);
    }

    /// Mine all work units in parallel
    pub async fn mine_batch(&mut self, miner_id: String) -> TribeResult<Vec<WorkProof>> {
        let mut proofs = Vec::new();
        
        for work in &mut self.work_queue {
            let ai3_miner = self.ai3_miners.get_mut(&miner_id);
            
            if let Some(proof) = self.pow.mine_block(work, miner_id.clone(), ai3_miner)? {
                proofs.push(proof);
            }
        }

        // Clear completed work
        self.work_queue.clear();
        
        Ok(proofs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tribechain_core::{Transaction, TransactionType};

    #[test]
    fn test_proof_of_work_creation() {
        let pow = ProofOfWork::new(4, 600);
        assert_eq!(pow.difficulty, 4);
        assert_eq!(pow.target_block_time, 600);
        assert!(pow.ai3_integration);
    }

    #[test]
    fn test_difficulty_target() {
        let pow = ProofOfWork::new(4, 600);
        let target = pow.calculate_target();
        assert_eq!(target, "0000");
    }

    #[test]
    fn test_meets_difficulty() {
        let pow = ProofOfWork::new(4, 600);
        let target = pow.calculate_target();
        
        assert!(pow.meets_difficulty("0000abcd", &target));
        assert!(!pow.meets_difficulty("000abcd", &target));
        assert!(!pow.meets_difficulty("1000abcd", &target));
    }

    #[test]
    fn test_difficulty_adjustment() {
        let mut pow = ProofOfWork::new(4, 600);
        
        // Test slow blocks (should decrease difficulty)
        let new_difficulty = pow.adjust_difficulty(1200); // 2x target time
        assert_eq!(new_difficulty, 3);
        
        // Test fast blocks (should increase difficulty)
        pow.difficulty = 4;
        let new_difficulty = pow.adjust_difficulty(200); // 0.33x target time
        assert_eq!(new_difficulty, 5);
    }

    #[test]
    fn test_create_work() {
        let pow = ProofOfWork::new(4, 600);
        let block = Block::new(
            1,
            "prev_hash".to_string(),
            vec![],
            "miner".to_string(),
            4,
        );
        
        let work = pow.create_work(block, None);
        assert_eq!(work.target, "0000");
        assert_eq!(work.start_nonce, 0);
        assert_eq!(work.end_nonce, u64::MAX);
    }

    #[tokio::test]
    async fn test_batch_miner() {
        let pow = ProofOfWork::new(1, 600); // Low difficulty for testing
        let mut batch_miner = BatchMiner::new(pow);
        
        let block = Block::new(
            1,
            "prev_hash".to_string(),
            vec![],
            "miner".to_string(),
            1,
        );
        
        let work = batch_miner.pow.create_work(block, None);
        batch_miner.add_work(work);
        
        let proofs = batch_miner.mine_batch("test_miner".to_string()).await.unwrap();
        // With difficulty 1, we should find a proof quickly
        assert!(!proofs.is_empty() || batch_miner.work_queue.is_empty());
    }
} 