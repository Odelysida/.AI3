use serde::{Deserialize, Serialize};
use std::path::Path;
use rocksdb::{DB, Options};
use crate::{TribeChain, Block, Transaction, TribeResult, TribeError};

/// Storage backend for TribeChain
pub struct Storage {
    db: DB,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(path: &str) -> TribeResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        
        let db = DB::open(&opts, path)
            .map_err(|e| TribeError::Storage(format!("Failed to open database: {}", e)))?;
        
        Ok(Storage { db })
    }

    /// Save the entire blockchain
    pub fn save_blockchain(&self, blockchain: &TribeChain) -> TribeResult<()> {
        let serialized = bincode::serialize(blockchain)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize blockchain: {}", e)))?;
        
        self.db.put(b"blockchain", serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save blockchain: {}", e)))?;
        
        Ok(())
    }

    /// Load the blockchain
    pub fn load_blockchain(&self) -> TribeResult<TribeChain> {
        let data = self.db.get(b"blockchain")
            .map_err(|e| TribeError::Storage(format!("Failed to load blockchain: {}", e)))?
            .ok_or_else(|| TribeError::Storage("Blockchain not found".to_string()))?;
        
        let blockchain = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize blockchain: {}", e)))?;
        
        Ok(blockchain)
    }

    /// Save a block
    pub fn save_block(&self, block: &Block, index: u64) -> TribeResult<()> {
        let key = format!("block_{}", index);
        let serialized = bincode::serialize(block)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize block: {}", e)))?;
        
        self.db.put(key.as_bytes(), serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save block: {}", e)))?;
        
        Ok(())
    }

    /// Load a block by index
    pub fn load_block(&self, index: u64) -> TribeResult<Block> {
        let key = format!("block_{}", index);
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load block: {}", e)))?
            .ok_or_else(|| TribeError::Storage(format!("Block {} not found", index)))?;
        
        let block = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize block: {}", e)))?;
        
        Ok(block)
    }

    /// Save a transaction
    pub fn save_transaction(&self, transaction: &Transaction) -> TribeResult<()> {
        let key = format!("tx_{}", transaction.hash);
        let serialized = bincode::serialize(transaction)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize transaction: {}", e)))?;
        
        self.db.put(key.as_bytes(), serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save transaction: {}", e)))?;
        
        Ok(())
    }

    /// Load a transaction by hash
    pub fn load_transaction(&self, hash: &str) -> TribeResult<Transaction> {
        let key = format!("tx_{}", hash);
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load transaction: {}", e)))?
            .ok_or_else(|| TribeError::Storage(format!("Transaction {} not found", hash)))?;
        
        let transaction = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize transaction: {}", e)))?;
        
        Ok(transaction)
    }

    /// Save key-value pair
    pub fn save_data(&self, key: &str, value: &[u8]) -> TribeResult<()> {
        self.db.put(key.as_bytes(), value)
            .map_err(|e| TribeError::Storage(format!("Failed to save data: {}", e)))?;
        
        Ok(())
    }

    /// Load data by key
    pub fn load_data(&self, key: &str) -> TribeResult<Option<Vec<u8>>> {
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load data: {}", e)))?;
        
        Ok(data)
    }

    /// Delete data by key
    pub fn delete_data(&self, key: &str) -> TribeResult<()> {
        self.db.delete(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to delete data: {}", e)))?;
        
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> TribeResult<StorageStats> {
        // Get approximate number of keys
        let mut iter = self.db.iterator(rocksdb::IteratorMode::Start);
        let mut total_keys = 0;
        let mut total_size = 0;
        
        while let Some(Ok((key, value))) = iter.next() {
            total_keys += 1;
            total_size += key.len() + value.len();
        }
        
        Ok(StorageStats {
            total_keys,
            total_size,
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_keys: usize,
    pub total_size: usize,
} 