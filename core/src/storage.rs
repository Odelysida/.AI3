use serde::{Deserialize, Serialize};
#[cfg(feature = "storage")]
use rocksdb::{DB, Options};
use crate::{TribeChain, Block, Transaction, TribeResult, TribeError};

/// Storage backend for TribeChain
#[derive(Debug, Clone)]
pub struct Storage {
    #[cfg(feature = "storage")]
    db: DB,
    #[cfg(not(feature = "storage"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Storage {
    /// Create a new storage instance
    #[cfg(feature = "storage")]
    pub fn new(path: &str) -> TribeResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        
        let db = DB::open(&opts, path)
            .map_err(|e| TribeError::Storage(format!("Failed to open database: {}", e)))?;
        
        Ok(Storage { db })
    }

    /// Create a new storage instance (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn new(_path: &str) -> TribeResult<Self> {
        Ok(Storage { _phantom: std::marker::PhantomData })
    }

    /// Save the entire blockchain
    #[cfg(feature = "storage")]
    pub fn save_blockchain(&self, blockchain: &TribeChain) -> TribeResult<()> {
        let serialized = bincode::serialize(blockchain)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize blockchain: {}", e)))?;
        
        self.db.put(b"blockchain", serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save blockchain: {}", e)))?;
        
        Ok(())
    }

    /// Save the entire blockchain (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn save_blockchain(&self, _blockchain: &TribeChain) -> TribeResult<()> {
        Ok(())
    }

    /// Load the blockchain
    #[cfg(feature = "storage")]
    pub fn load_blockchain(&self) -> TribeResult<TribeChain> {
        let data = self.db.get(b"blockchain")
            .map_err(|e| TribeError::Storage(format!("Failed to load blockchain: {}", e)))?
            .ok_or_else(|| TribeError::Storage("Blockchain not found".to_string()))?;
        
        let blockchain = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize blockchain: {}", e)))?;
        
        Ok(blockchain)
    }

    /// Load the blockchain (returns error when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn load_blockchain(&self) -> TribeResult<TribeChain> {
        Err(TribeError::Storage("Storage feature not enabled".to_string()))
    }

    /// Save a block
    #[cfg(feature = "storage")]
    pub fn save_block(&self, block: &Block, index: u64) -> TribeResult<()> {
        let key = format!("block_{}", index);
        let serialized = bincode::serialize(block)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize block: {}", e)))?;
        
        self.db.put(key.as_bytes(), serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save block: {}", e)))?;
        
        Ok(())
    }

    /// Save a block (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn save_block(&self, _block: &Block, _index: u64) -> TribeResult<()> {
        Ok(())
    }

    /// Load a block by index
    #[cfg(feature = "storage")]
    pub fn load_block(&self, index: u64) -> TribeResult<Block> {
        let key = format!("block_{}", index);
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load block: {}", e)))?
            .ok_or_else(|| TribeError::Storage(format!("Block {} not found", index)))?;
        
        let block = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize block: {}", e)))?;
        
        Ok(block)
    }

    /// Load a block by index (returns error when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn load_block(&self, _index: u64) -> TribeResult<Block> {
        Err(TribeError::Storage("Storage feature not enabled".to_string()))
    }

    /// Save a transaction
    #[cfg(feature = "storage")]
    pub fn save_transaction(&self, transaction: &Transaction) -> TribeResult<()> {
        let key = format!("tx_{}", transaction.hash);
        let serialized = bincode::serialize(transaction)
            .map_err(|e| TribeError::Storage(format!("Failed to serialize transaction: {}", e)))?;
        
        self.db.put(key.as_bytes(), serialized)
            .map_err(|e| TribeError::Storage(format!("Failed to save transaction: {}", e)))?;
        
        Ok(())
    }

    /// Save a transaction (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn save_transaction(&self, _transaction: &Transaction) -> TribeResult<()> {
        Ok(())
    }

    /// Load a transaction by hash
    #[cfg(feature = "storage")]
    pub fn load_transaction(&self, hash: &str) -> TribeResult<Transaction> {
        let key = format!("tx_{}", hash);
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load transaction: {}", e)))?
            .ok_or_else(|| TribeError::Storage(format!("Transaction {} not found", hash)))?;
        
        let transaction = bincode::deserialize(&data)
            .map_err(|e| TribeError::Storage(format!("Failed to deserialize transaction: {}", e)))?;
        
        Ok(transaction)
    }

    /// Load a transaction by hash (returns error when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn load_transaction(&self, _hash: &str) -> TribeResult<Transaction> {
        Err(TribeError::Storage("Storage feature not enabled".to_string()))
    }

    /// Save key-value pair
    #[cfg(feature = "storage")]
    pub fn save_data(&self, key: &str, value: &[u8]) -> TribeResult<()> {
        self.db.put(key.as_bytes(), value)
            .map_err(|e| TribeError::Storage(format!("Failed to save data: {}", e)))?;
        
        Ok(())
    }

    /// Save key-value pair (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn save_data(&self, _key: &str, _value: &[u8]) -> TribeResult<()> {
        Ok(())
    }

    /// Load data by key
    #[cfg(feature = "storage")]
    pub fn load_data(&self, key: &str) -> TribeResult<Option<Vec<u8>>> {
        let data = self.db.get(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to load data: {}", e)))?;
        
        Ok(data)
    }

    /// Load data by key (returns None when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn load_data(&self, _key: &str) -> TribeResult<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Delete data by key
    #[cfg(feature = "storage")]
    pub fn delete_data(&self, key: &str) -> TribeResult<()> {
        self.db.delete(key.as_bytes())
            .map_err(|e| TribeError::Storage(format!("Failed to delete data: {}", e)))?;
        
        Ok(())
    }

    /// Delete data by key (no-op when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn delete_data(&self, _key: &str) -> TribeResult<()> {
        Ok(())
    }

    /// Get database statistics
    #[cfg(feature = "storage")]
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

    /// Get database statistics (returns empty stats when storage feature is disabled)
    #[cfg(not(feature = "storage"))]
    pub fn get_stats(&self) -> TribeResult<StorageStats> {
        Ok(StorageStats {
            total_keys: 0,
            total_size: 0,
        })
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_keys: usize,
    pub total_size: usize,
} 