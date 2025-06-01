use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use crate::{Transaction, TribeResult};

/// Block structure for TribeChain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u64,
    pub transactions: Vec<Transaction>,
    pub miner: String,
    pub merkle_root: String,
    pub ai3_proof: Option<AI3Proof>,
}

/// AI3 Proof structure for tensor mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI3Proof {
    pub task_id: String,
    pub optimization_factor: f32,
    pub tensor_hash: String,
    pub computation_time: u64,
    pub miner_signature: String,
}

impl Block {
    /// Create a new block
    pub fn new(
        index: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        miner: String,
    ) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        Block {
            index,
            timestamp,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            difficulty: 4, // Default difficulty
            transactions,
            miner,
            merkle_root,
            ai3_proof: None,
        }
    }

    /// Create genesis block
    pub fn genesis() -> Self {
        let mut genesis = Block {
            index: 0,
            timestamp: 1640995200, // Jan 1, 2022
            previous_hash: "0".repeat(64),
            hash: String::new(),
            nonce: 0,
            difficulty: 1,
            transactions: Vec::new(),
            miner: "genesis".to_string(),
            merkle_root: "0".repeat(64),
            ai3_proof: None,
        };
        
        genesis.hash = genesis.calculate_hash();
        genesis
    }

    /// Calculate block hash
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}{}{}{}",
            self.index,
            self.timestamp,
            self.previous_hash,
            self.nonce,
            self.difficulty,
            self.miner,
            self.merkle_root,
            serde_json::to_string(&self.ai3_proof).unwrap_or_default()
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Mine the block (find valid nonce)
    pub fn mine_block(&mut self, difficulty: u64) -> TribeResult<()> {
        self.difficulty = difficulty;
        let target = "0".repeat(difficulty as usize);
        
        println!("Mining block {} with difficulty {}...", self.index, difficulty);
        
        loop {
            self.hash = self.calculate_hash();
            
            if self.hash.starts_with(&target) {
                println!("Block mined! Hash: {}, Nonce: {}", self.hash, self.nonce);
                break;
            }
            
            self.nonce += 1;
            
            // Prevent infinite loop in case of very high difficulty
            if self.nonce % 100000 == 0 {
                println!("Mining progress: nonce = {}", self.nonce);
            }
        }
        
        Ok(())
    }

    /// Mine block with AI3 proof
    pub fn mine_with_ai3_proof(&mut self, difficulty: u64, ai3_proof: AI3Proof) -> TribeResult<()> {
        self.ai3_proof = Some(ai3_proof);
        self.mine_block(difficulty)
    }

    /// Validate block hash
    pub fn is_valid_hash(&self, difficulty: u64) -> bool {
        let target = "0".repeat(difficulty as usize);
        self.hash.starts_with(&target) && self.hash == self.calculate_hash()
    }

    /// Calculate merkle root of transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0".repeat(64);
        }

        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| tx.calculate_hash())
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    format!("{}{}", chunk[0], chunk[0])
                };
                
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                next_level.push(hex::encode(hasher.finalize()));
            }
            
            hashes = next_level;
        }

        hashes[0].clone()
    }

    /// Get block size in bytes
    pub fn get_size(&self) -> usize {
        bincode::serialize(self).unwrap_or_default().len()
    }

    /// Validate block structure
    pub fn validate(&self, previous_block: Option<&Block>) -> TribeResult<bool> {
        // Check hash
        if self.hash != self.calculate_hash() {
            return Ok(false);
        }

        // Check previous hash
        if let Some(prev) = previous_block {
            if self.previous_hash != prev.hash {
                return Ok(false);
            }
            if self.index != prev.index + 1 {
                return Ok(false);
            }
        }

        // Check merkle root
        let calculated_merkle = Self::calculate_merkle_root(&self.transactions);
        if self.merkle_root != calculated_merkle {
            return Ok(false);
        }

        // Validate difficulty
        if !self.is_valid_hash(self.difficulty) {
            return Ok(false);
        }

        Ok(true)
    }
} 