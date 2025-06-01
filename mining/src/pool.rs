use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use std::sync::Arc;
use tribechain_core::{TribeResult, TribeError, Block, Transaction};
use crate::miner::{Miner, MinerStats, MinerType};

/// Mining pool for coordinating multiple miners
#[derive(Debug)]
pub struct MiningPool {
    pub pool_id: String,
    pub name: String,
    pub miners: Arc<RwLock<HashMap<String, Miner>>>,
    pub stats: PoolStats,
    pub config: PoolConfig,
    pub reward_distribution: RewardDistribution,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_miners: usize,
    pub min_difficulty: u32,
    pub pool_fee_percentage: f64,
    pub payout_threshold: u64,
    pub payout_interval_hours: u64,
    pub auto_difficulty_adjustment: bool,
    pub allow_ai3_mining: bool,
    pub require_registration: bool,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_miners: usize,
    pub active_miners: usize,
    pub total_hash_rate: f64,
    pub blocks_found: u64,
    pub total_shares: u64,
    pub valid_shares: u64,
    pub invalid_shares: u64,
    pub last_block_time: Option<DateTime<Utc>>,
    pub average_block_time: f64,
    pub pool_luck: f64, // percentage
    pub uptime: f64, // percentage
}

/// Reward distribution methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardDistribution {
    Proportional, // Based on shares contributed
    PayPerShare, // Fixed payment per share
    PayPerLastNShares(u64), // Based on last N shares
    ScoreBasedShares, // Time-weighted shares
}

/// Mining share submitted by a miner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningShare {
    pub miner_id: String,
    pub block_height: u64,
    pub nonce: u64,
    pub hash: String,
    pub difficulty: u32,
    pub is_valid: bool,
    pub timestamp: DateTime<Utc>,
    pub ai3_proof: Option<AI3ShareProof>,
}

/// AI3-specific share proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3ShareProof {
    pub task_id: String,
    pub tensor_result: Vec<f32>,
    pub computation_time: u64,
    pub verification_hash: String,
}

/// Miner earnings and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerEarnings {
    pub miner_id: String,
    pub total_shares: u64,
    pub valid_shares: u64,
    pub total_earnings: u64,
    pub pending_payout: u64,
    pub last_payout: Option<DateTime<Utc>>,
    pub efficiency_score: f64,
}

impl MiningPool {
    pub fn new(pool_id: String, name: String, config: PoolConfig) -> Self {
        Self {
            pool_id,
            name,
            miners: Arc::new(RwLock::new(HashMap::new())),
            stats: PoolStats::default(),
            config,
            reward_distribution: RewardDistribution::Proportional,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    pub async fn add_miner(&mut self, miner: Miner) -> TribeResult<()> {
        let mut miners = self.miners.write().await;
        
        if miners.len() >= self.config.max_miners {
            return Err(TribeError::InvalidOperation("Pool is full".to_string()));
        }

        if self.config.require_registration && !miner.is_active {
            return Err(TribeError::InvalidOperation("Miner must be registered".to_string()));
        }

        miners.insert(miner.id.clone(), miner);
        self.stats.total_miners = miners.len();
        self.update_active_miners().await;
        
        Ok(())
    }

    pub async fn remove_miner(&mut self, miner_id: &str) -> TribeResult<()> {
        let mut miners = self.miners.write().await;
        
        if miners.remove(miner_id).is_none() {
            return Err(TribeError::InvalidOperation("Miner not found".to_string()));
        }

        self.stats.total_miners = miners.len();
        self.update_active_miners().await;
        
        Ok(())
    }

    pub async fn submit_share(&mut self, share: MiningShare) -> TribeResult<bool> {
        let miners = self.miners.read().await;
        
        if !miners.contains_key(&share.miner_id) {
            return Err(TribeError::InvalidOperation("Miner not in pool".to_string()));
        }

        // Validate share
        let is_valid = self.validate_share(&share).await?;
        
        self.stats.total_shares += 1;
        if is_valid {
            self.stats.valid_shares += 1;
            
            // Check if this share solves a block
            if self.is_block_solution(&share) {
                self.handle_block_found(&share).await?;
            }
        } else {
            self.stats.invalid_shares += 1;
        }

        Ok(is_valid)
    }

    async fn validate_share(&self, share: &MiningShare) -> TribeResult<bool> {
        // Basic validation
        if share.hash.is_empty() || share.miner_id.is_empty() {
            return Ok(false);
        }

        // Check difficulty
        let target = "0".repeat(self.config.min_difficulty as usize);
        if !share.hash.starts_with(&target) {
            return Ok(false);
        }

        // Validate AI3 proof if present
        if let Some(ai3_proof) = &share.ai3_proof {
            if !self.config.allow_ai3_mining {
                return Ok(false);
            }
            
            return self.validate_ai3_proof(ai3_proof).await;
        }

        Ok(true)
    }

    async fn validate_ai3_proof(&self, proof: &AI3ShareProof) -> TribeResult<bool> {
        // Validate AI3 tensor computation proof
        if proof.task_id.is_empty() || proof.tensor_result.is_empty() {
            return Ok(false);
        }

        // Verify computation hash
        let computed_hash = self.compute_ai3_hash(&proof.tensor_result, &proof.task_id);
        Ok(computed_hash == proof.verification_hash)
    }

    fn compute_ai3_hash(&self, tensor_result: &[f32], task_id: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(task_id.as_bytes());
        
        for &value in tensor_result {
            hasher.update(value.to_le_bytes());
        }
        
        format!("{:x}", hasher.finalize())
    }

    fn is_block_solution(&self, share: &MiningShare) -> bool {
        // Check if share meets block difficulty (higher than pool difficulty)
        let block_target = "0".repeat(6); // Example block difficulty
        share.hash.starts_with(&block_target)
    }

    async fn handle_block_found(&mut self, share: &MiningShare) -> TribeResult<()> {
        self.stats.blocks_found += 1;
        self.stats.last_block_time = Some(share.timestamp);
        
        // Calculate and distribute rewards
        self.distribute_block_reward(share).await?;
        
        // Update pool luck
        self.update_pool_luck().await;
        
        Ok(())
    }

    async fn distribute_block_reward(&mut self, _share: &MiningShare) -> TribeResult<()> {
        let block_reward = 50_000_000; // Example: 50 TRIBE tokens
        let pool_fee = (block_reward as f64 * self.config.pool_fee_percentage / 100.0) as u64;
        let miner_reward = block_reward - pool_fee;

        match self.reward_distribution {
            RewardDistribution::Proportional => {
                self.distribute_proportional_reward(miner_reward).await?;
            }
            RewardDistribution::PayPerShare => {
                self.distribute_pps_reward(miner_reward).await?;
            }
            RewardDistribution::PayPerLastNShares(n) => {
                self.distribute_pplns_reward(miner_reward, n).await?;
            }
            RewardDistribution::ScoreBasedShares => {
                self.distribute_score_based_reward(miner_reward).await?;
            }
        }

        Ok(())
    }

    async fn distribute_proportional_reward(&mut self, total_reward: u64) -> TribeResult<()> {
        let miners = self.miners.read().await;
        let total_shares: u64 = miners.values().map(|m| m.stats.successful_hashes).sum();
        
        if total_shares == 0 {
            return Ok(());
        }

        for miner in miners.values() {
            let miner_shares = miner.stats.successful_hashes;
            let reward = (total_reward as f64 * miner_shares as f64 / total_shares as f64) as u64;
            
            // In a real implementation, you would update the miner's balance
            println!("Miner {} earned {} tokens", miner.id, reward);
        }

        Ok(())
    }

    async fn distribute_pps_reward(&mut self, _total_reward: u64) -> TribeResult<()> {
        // Pay-per-share implementation
        let pps_rate = 1000; // Example: 1000 satoshis per share
        
        let miners = self.miners.read().await;
        for miner in miners.values() {
            let reward = miner.stats.successful_hashes * pps_rate;
            println!("Miner {} earned {} tokens (PPS)", miner.id, reward);
        }

        Ok(())
    }

    async fn distribute_pplns_reward(&mut self, total_reward: u64, n_shares: u64) -> TribeResult<()> {
        // Pay-per-last-N-shares implementation
        let miners = self.miners.read().await;
        let recent_shares: u64 = miners.values()
            .map(|m| std::cmp::min(m.stats.successful_hashes, n_shares))
            .sum();
        
        if recent_shares == 0 {
            return Ok(());
        }

        for miner in miners.values() {
            let miner_recent_shares = std::cmp::min(miner.stats.successful_hashes, n_shares);
            let reward = (total_reward as f64 * miner_recent_shares as f64 / recent_shares as f64) as u64;
            println!("Miner {} earned {} tokens (PPLNS)", miner.id, reward);
        }

        Ok(())
    }

    async fn distribute_score_based_reward(&mut self, total_reward: u64) -> TribeResult<()> {
        // Score-based shares implementation (time-weighted)
        let miners = self.miners.read().await;
        let total_score: f64 = miners.values()
            .map(|m| m.get_efficiency_score())
            .sum();
        
        if total_score == 0.0 {
            return Ok(());
        }

        for miner in miners.values() {
            let miner_score = miner.get_efficiency_score();
            let reward = (total_reward as f64 * miner_score / total_score) as u64;
            println!("Miner {} earned {} tokens (Score-based)", miner.id, reward);
        }

        Ok(())
    }

    async fn update_active_miners(&mut self) {
        let miners = self.miners.read().await;
        self.stats.active_miners = miners.values().filter(|m| m.is_online()).count();
        
        // Update total hash rate
        self.stats.total_hash_rate = miners.values()
            .filter(|m| m.is_online())
            .map(|m| m.capabilities.hash_rate)
            .sum();
    }

    async fn update_pool_luck(&mut self) {
        // Calculate pool luck based on expected vs actual blocks found
        let expected_blocks = self.stats.valid_shares as f64 / 1000000.0; // Example calculation
        if expected_blocks > 0.0 {
            self.stats.pool_luck = (self.stats.blocks_found as f64 / expected_blocks) * 100.0;
        }
    }

    pub async fn get_miner_earnings(&self, miner_id: &str) -> TribeResult<MinerEarnings> {
        let miners = self.miners.read().await;
        
        if let Some(miner) = miners.get(miner_id) {
            Ok(MinerEarnings {
                miner_id: miner.id.clone(),
                total_shares: miner.stats.total_hash_attempts,
                valid_shares: miner.stats.successful_hashes,
                total_earnings: miner.stats.earnings,
                pending_payout: 0, // Would be calculated from pending shares
                last_payout: None, // Would be tracked separately
                efficiency_score: miner.get_efficiency_score(),
            })
        } else {
            Err(TribeError::InvalidOperation("Miner not found".to_string()))
        }
    }

    pub fn get_stats(&self) -> PoolStats {
        self.stats.clone()
    }

    pub async fn get_top_miners(&self, limit: usize) -> Vec<(String, f64)> {
        let miners = self.miners.read().await;
        let mut miner_scores: Vec<_> = miners.iter()
            .map(|(id, miner)| (id.clone(), miner.get_efficiency_score()))
            .collect();
        
        miner_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        miner_scores.truncate(limit);
        miner_scores
    }

    pub async fn set_reward_distribution(&mut self, distribution: RewardDistribution) {
        self.reward_distribution = distribution;
    }

    pub async fn update_config(&mut self, config: PoolConfig) {
        self.config = config;
    }
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_miners: 0,
            active_miners: 0,
            total_hash_rate: 0.0,
            blocks_found: 0,
            total_shares: 0,
            valid_shares: 0,
            invalid_shares: 0,
            last_block_time: None,
            average_block_time: 600.0, // 10 minutes
            pool_luck: 100.0,
            uptime: 100.0,
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_miners: 1000,
            min_difficulty: 4,
            pool_fee_percentage: 2.0,
            payout_threshold: 1_000_000, // 1 TRIBE token
            payout_interval_hours: 24,
            auto_difficulty_adjustment: true,
            allow_ai3_mining: true,
            require_registration: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::miner::{MinerCapabilities, MinerType};

    #[tokio::test]
    async fn test_mining_pool_creation() {
        let config = PoolConfig::default();
        let pool = MiningPool::new("pool1".to_string(), "Test Pool".to_string(), config);
        
        assert_eq!(pool.pool_id, "pool1");
        assert_eq!(pool.name, "Test Pool");
        assert!(pool.is_active);
    }

    #[tokio::test]
    async fn test_add_remove_miner() {
        let config = PoolConfig::default();
        let mut pool = MiningPool::new("pool1".to_string(), "Test Pool".to_string(), config);
        
        let capabilities = MinerCapabilities::default();
        let miner = Miner::new("miner1".to_string(), "addr1".to_string(), MinerType::CPU, capabilities);
        
        pool.add_miner(miner).await.unwrap();
        assert_eq!(pool.stats.total_miners, 1);
        
        pool.remove_miner("miner1").await.unwrap();
        assert_eq!(pool.stats.total_miners, 0);
    }

    #[tokio::test]
    async fn test_share_submission() {
        let config = PoolConfig::default();
        let mut pool = MiningPool::new("pool1".to_string(), "Test Pool".to_string(), config);
        
        let capabilities = MinerCapabilities::default();
        let miner = Miner::new("miner1".to_string(), "addr1".to_string(), MinerType::CPU, capabilities);
        pool.add_miner(miner).await.unwrap();
        
        let share = MiningShare {
            miner_id: "miner1".to_string(),
            block_height: 1,
            nonce: 12345,
            hash: "0000abcd".to_string(), // Valid hash for difficulty 4
            difficulty: 4,
            is_valid: true,
            timestamp: Utc::now(),
            ai3_proof: None,
        };
        
        let result = pool.submit_share(share).await.unwrap();
        assert!(result);
        assert_eq!(pool.stats.valid_shares, 1);
    }

    #[test]
    fn test_reward_distribution_types() {
        let proportional = RewardDistribution::Proportional;
        let pps = RewardDistribution::PayPerShare;
        let pplns = RewardDistribution::PayPerLastNShares(1000);
        let score_based = RewardDistribution::ScoreBasedShares;
        
        // Test serialization
        let _prop_json = serde_json::to_string(&proportional).unwrap();
        let _pps_json = serde_json::to_string(&pps).unwrap();
        let _pplns_json = serde_json::to_string(&pplns).unwrap();
        let _score_json = serde_json::to_string(&score_based).unwrap();
    }
} 