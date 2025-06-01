use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Block, Transaction, TransactionType, Storage, TribeResult, TribeError, AI3Proof};

/// Miner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerInfo {
    pub id: String,
    pub address: String,
    pub hash_rate: u64,
    pub last_seen: u64,
    pub blocks_mined: u64,
    pub ai3_capability: bool,
}

/// Tensor computation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorTask {
    pub id: String,
    pub operation: String,
    pub input_data: Vec<f32>,
    pub expected_output_size: usize,
    pub max_computation_time: u64,
    pub reward: u64,
    pub requester: String,
    pub completed: bool,
    pub result: Option<Vec<f32>>,
    pub assigned_miner: Option<String>,
    pub created_at: u64,
}

/// Main TribeChain blockchain structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TribeChain {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub difficulty: u64,
    pub mining_reward: u64,
    pub balances: HashMap<String, u64>,
    #[serde(skip)]
    pub storage: Option<Storage>,
    pub tensor_tasks: Vec<TensorTask>,
    pub active_miners: HashMap<String, MinerInfo>,
    pub ai3_difficulty_multiplier: f32,
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub block_count: u64,
    pub transaction_count: u64,
    pub pending_transactions: u64,
    pub difficulty: u64,
    pub mining_reward: u64,
    pub total_supply: u64,
    pub active_addresses: u64,
    pub avg_block_time: u64,
    pub active_miners: u64,
    pub tensor_tasks: u64,
    pub ai3_difficulty_multiplier: f32,
}

impl TribeChain {
    /// Create a new TribeChain
    pub fn new(storage_path: &str) -> TribeResult<Self> {
        let storage = Storage::new(storage_path)?;
        
        // Try to load existing blockchain
        match storage.load_blockchain() {
            Ok(mut blockchain) => {
                blockchain.storage = Some(storage);
                Ok(blockchain)
            }
            Err(_) => {
                // Create new blockchain with genesis block
                let mut blockchain = TribeChain {
                    blocks: Vec::new(),
                    pending_transactions: Vec::new(),
                    difficulty: 4, // Starting difficulty
                    mining_reward: 50_000_000, // 50 TRIBE tokens (with 6 decimals)
                    balances: HashMap::new(),
                    storage: Some(storage),
                    tensor_tasks: Vec::new(),
                    active_miners: HashMap::new(),
                    ai3_difficulty_multiplier: 1.5, // AI3 mining is 50% more difficult
                };
                
                // Create genesis block
                blockchain.create_genesis_block()?;
                Ok(blockchain)
            }
        }
    }

    /// Create the genesis block
    fn create_genesis_block(&mut self) -> TribeResult<()> {
        let genesis_block = Block::genesis();
        self.blocks.push(genesis_block);
        
        // Initialize genesis balances
        self.balances.insert("genesis".to_string(), 1_000_000_000_000); // 1M TRIBE tokens
        
        // Save to storage
        if let Some(storage) = &self.storage {
            storage.save_blockchain(self)?;
        }
        
        Ok(())
    }

    /// Add a new transaction to the pending pool
    pub fn add_transaction(&mut self, transaction: Transaction) -> TribeResult<()> {
        // Validate transaction
        if !self.validate_transaction(&transaction)? {
            return Err(TribeError::InvalidTransaction("Transaction validation failed".to_string()));
        }
        
        // Add to pending transactions
        self.pending_transactions.push(transaction.clone());
        
        // Save transaction to storage
        if let Some(storage) = &self.storage {
            storage.save_transaction(&transaction)?;
        }
        
        Ok(())
    }

    /// Validate a transaction
    fn validate_transaction(&self, transaction: &Transaction) -> TribeResult<bool> {
        // Basic validation
        if !transaction.validate()? {
            return Ok(false);
        }

        // Check if sender has sufficient balance
        let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
        
        match &transaction.transaction_type {
            TransactionType::Transfer { amount, .. } => {
                if sender_balance < &(amount + transaction.fee) {
                    return Ok(false);
                }
            }
            TransactionType::TokenCreate { .. } => {
                // Token creation requires minimum balance
                if *sender_balance < 1_000_000 + transaction.fee { // 1 TRIBE token + fee
                    return Ok(false);
                }
            }
            TransactionType::TokenTransfer { amount, .. } => {
                if sender_balance < &(amount + transaction.fee) {
                    return Ok(false);
                }
            }
            TransactionType::Stake { amount, .. } => {
                if sender_balance < &(amount + transaction.fee) {
                    return Ok(false);
                }
            }
            TransactionType::TensorCompute { reward, .. } => {
                // Tensor compute requires balance for reward + fee
                if sender_balance < &(reward + transaction.fee) {
                    return Ok(false);
                }
            }
            TransactionType::ContractDeploy { .. } => {
                if *sender_balance < transaction.fee {
                    return Ok(false);
                }
            }
            TransactionType::ContractCall { value, .. } => {
                if sender_balance < &(value + transaction.fee) {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }

    /// Mine a new block (traditional mining)
    pub fn mine_block(&mut self, miner_address: String) -> TribeResult<Block> {
        if self.pending_transactions.is_empty() {
            return Err(TribeError::Mining("No pending transactions to mine".to_string()));
        }
        
        // Get previous block hash
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            last_block.hash.clone()
        } else {
            "0".repeat(64)
        };
        
        // Create new block
        let mut block = Block::new(
            self.blocks.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
            miner_address.clone(),
        );
        
        // Mine the block (find valid nonce)
        block.mine_block(self.difficulty)?;
        
        // Add block to chain
        self.add_block(block.clone())?;
        
        // Reward miner
        let current_balance = self.balances.get(&miner_address).unwrap_or(&0);
        self.balances.insert(miner_address, current_balance + self.mining_reward);
        
        // Clear pending transactions
        self.pending_transactions.clear();
        
        // Save to storage
        if let Some(storage) = &self.storage {
            storage.save_blockchain(self)?;
        }
        
        Ok(block)
    }

    /// Mine a block with AI3 proof (enhanced mining)
    pub fn mine_block_with_ai3(&mut self, miner_address: String, ai3_proof: AI3Proof) -> TribeResult<Block> {
        if self.pending_transactions.is_empty() {
            return Err(TribeError::Mining("No pending transactions to mine".to_string()));
        }
        
        // Validate AI3 proof
        if !self.validate_ai3_proof(&ai3_proof)? {
            return Err(TribeError::AI3("Invalid AI3 proof".to_string()));
        }
        
        // Get previous block hash
        let previous_hash = if let Some(last_block) = self.blocks.last() {
            last_block.hash.clone()
        } else {
            "0".repeat(64)
        };
        
        // Create new block
        let mut block = Block::new(
            self.blocks.len() as u64,
            previous_hash,
            self.pending_transactions.clone(),
            miner_address.clone(),
        );
        
        // Calculate AI3 adjusted difficulty
        let ai3_difficulty = (self.difficulty as f32 * self.ai3_difficulty_multiplier) as u64;
        
        // Mine the block with AI3 proof
        block.mine_with_ai3_proof(ai3_difficulty, ai3_proof.clone())?;
        
        // Add block to chain
        self.add_block(block.clone())?;
        
        // Enhanced reward for AI3 mining
        let ai3_bonus = (self.mining_reward as f32 * ai3_proof.optimization_factor) as u64;
        let total_reward = self.mining_reward + ai3_bonus;
        
        let current_balance = self.balances.get(&miner_address).unwrap_or(&0);
        self.balances.insert(miner_address.clone(), current_balance + total_reward);
        
        // Mark tensor task as completed if applicable
        if let Some(task) = self.tensor_tasks.iter_mut().find(|t| t.id == ai3_proof.task_id) {
            task.completed = true;
            task.assigned_miner = Some(miner_address);
        }
        
        // Clear pending transactions
        self.pending_transactions.clear();
        
        // Save to storage
        if let Some(storage) = &self.storage {
            storage.save_blockchain(self)?;
        }
        
        Ok(block)
    }

    /// Validate AI3 proof
    fn validate_ai3_proof(&self, proof: &AI3Proof) -> TribeResult<bool> {
        // Check if task exists
        let task = self.tensor_tasks.iter()
            .find(|t| t.id == proof.task_id)
            .ok_or_else(|| TribeError::AI3("Task not found".to_string()))?;
        
        // Check if task is not already completed
        if task.completed {
            return Ok(false);
        }
        
        // Validate optimization factor (should be between 0.1 and 2.0)
        if proof.optimization_factor < 0.1 || proof.optimization_factor > 2.0 {
            return Ok(false);
        }
        
        // Validate computation time
        if proof.computation_time > task.max_computation_time {
            return Ok(false);
        }
        
        // In a real implementation, we would verify the tensor computation
        // For now, we'll accept the proof if basic checks pass
        Ok(true)
    }

    /// Add a block to the chain
    pub fn add_block(&mut self, block: Block) -> TribeResult<()> {
        // Validate block
        let previous_block = self.blocks.last();
        if !block.validate(previous_block)? {
            return Err(TribeError::InvalidBlock("Block validation failed".to_string()));
        }
        
        // Process transactions in the block
        for transaction in &block.transactions {
            self.process_transaction(transaction)?;
        }
        
        // Add block to chain
        self.blocks.push(block.clone());
        
        // Adjust difficulty if needed
        self.adjust_difficulty();
        
        // Save block to storage
        if let Some(storage) = &self.storage {
            storage.save_block(&block, self.blocks.len() as u64 - 1)?;
        }
        
        Ok(())
    }

    /// Process a transaction (update balances)
    fn process_transaction(&mut self, transaction: &Transaction) -> TribeResult<()> {
        match &transaction.transaction_type {
            TransactionType::Transfer { to, amount } => {
                // Deduct from sender
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - amount - transaction.fee);
                
                // Add to receiver
                let receiver_balance = self.balances.get(to).unwrap_or(&0);
                self.balances.insert(to.clone(), receiver_balance + amount);
            }
            TransactionType::TokenCreate { .. } => {
                // Token creation fee
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - 1_000_000 - transaction.fee);
            }
            TransactionType::TokenTransfer { to, amount, .. } => {
                // Similar to regular transfer but for tokens
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - amount - transaction.fee);
                
                let receiver_balance = self.balances.get(to).unwrap_or(&0);
                self.balances.insert(to.clone(), receiver_balance + amount);
            }
            TransactionType::Stake { amount, .. } => {
                // Deduct staked amount from balance
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - amount - transaction.fee);
            }
            TransactionType::TensorCompute { reward, .. } => {
                // Deduct computation fee and reward
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - reward - transaction.fee);
            }
            TransactionType::ContractDeploy { .. } => {
                // Deduct deployment fee
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - transaction.fee);
            }
            TransactionType::ContractCall { value, .. } => {
                // Deduct call value and fee
                let sender_balance = self.balances.get(&transaction.from).unwrap_or(&0);
                self.balances.insert(transaction.from.clone(), sender_balance - value - transaction.fee);
            }
        }
        
        Ok(())
    }

    /// Adjust mining difficulty based on block time
    fn adjust_difficulty(&mut self) {
        if self.blocks.len() < 10 {
            return; // Not enough blocks to adjust
        }
        
        let recent_blocks = &self.blocks[self.blocks.len() - 10..];
        let time_diff = recent_blocks.last().unwrap().timestamp - recent_blocks.first().unwrap().timestamp;
        let target_time = 10 * 60; // 10 minutes per block target
        
        if time_diff < target_time / 2 {
            self.difficulty += 1; // Increase difficulty
        } else if time_diff > target_time * 2 && self.difficulty > 1 {
            self.difficulty -= 1; // Decrease difficulty
        }
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0)
    }

    /// Get blockchain statistics
    pub fn get_stats(&self) -> BlockchainStats {
        let total_supply: u64 = self.balances.values().sum();
        let avg_block_time = if self.blocks.len() > 1 {
            let time_diff = self.blocks.last().unwrap().timestamp - self.blocks.first().unwrap().timestamp;
            time_diff / (self.blocks.len() as u64 - 1)
        } else {
            0
        };
        
        BlockchainStats {
            block_count: self.blocks.len() as u64,
            transaction_count: self.blocks.iter().map(|b| b.transactions.len()).sum::<usize>() as u64,
            pending_transactions: self.pending_transactions.len() as u64,
            difficulty: self.difficulty,
            mining_reward: self.mining_reward,
            total_supply,
            active_addresses: self.balances.len() as u64,
            avg_block_time,
            active_miners: self.active_miners.len() as u64,
            tensor_tasks: self.tensor_tasks.len() as u64,
            ai3_difficulty_multiplier: self.ai3_difficulty_multiplier,
        }
    }

    /// Add a tensor computation task
    pub fn add_tensor_task(&mut self, task: TensorTask) -> TribeResult<()> {
        self.tensor_tasks.push(task);
        if let Some(storage) = &self.storage {
            storage.save_blockchain(self)?;
        }
        Ok(())
    }

    /// Register a miner
    pub fn register_miner(&mut self, miner_info: MinerInfo) -> TribeResult<()> {
        self.active_miners.insert(miner_info.id.clone(), miner_info);
        if let Some(storage) = &self.storage {
            storage.save_blockchain(self)?;
        }
        Ok(())
    }

    /// Get pending tensor tasks for mining
    pub fn get_pending_tensor_tasks(&self) -> Vec<&TensorTask> {
        self.tensor_tasks.iter()
            .filter(|task| !task.completed)
            .collect()
    }

    /// Complete a tensor task
    pub fn complete_tensor_task(&mut self, task_id: &str, result: Vec<f32>) -> TribeResult<()> {
        if let Some(task) = self.tensor_tasks.iter_mut().find(|t| t.id == task_id) {
            task.completed = true;
            task.result = Some(result);
            if let Some(storage) = &self.storage {
                storage.save_blockchain(self)?;
            }
            Ok(())
        } else {
            Err(TribeError::AI3(format!("Task {} not found", task_id)))
        }
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Get block by index
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.blocks.get(index as usize)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &str) -> Option<&Transaction> {
        for block in &self.blocks {
            for tx in &block.transactions {
                if tx.hash == hash {
                    return Some(tx);
                }
            }
        }
        None
    }
} 