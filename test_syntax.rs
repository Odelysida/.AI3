// Simple syntax test to verify our fixes
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBlock {
    pub index: u64,
    pub timestamp: u64,
    pub hash: String,
}

impl TestBlock {
    pub fn new(index: u64) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        let mut block = TestBlock {
            index,
            timestamp,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!("{}{}", self.index, self.timestamp);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct TestStorage {
    _phantom: std::marker::PhantomData<()>,
}

impl TestStorage {
    pub fn new() -> Self {
        TestStorage { _phantom: std::marker::PhantomData }
    }
}

fn main() {
    let block = TestBlock::new(1);
    println!("Block created: {:?}", block);
    
    let storage = TestStorage::new();
    println!("Storage created: {:?}", storage);
    
    // Test serde_json usage
    let json = serde_json::to_string(&block).unwrap();
    println!("Block as JSON: {}", json);
} 