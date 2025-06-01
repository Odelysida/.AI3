use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;
use uuid::Uuid;
use crate::{TribeResult, TribeError};

/// Transaction types supported by TribeChain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Regular token transfer
    Transfer { to: String, amount: u64 },
    /// Token creation
    TokenCreate { 
        name: String, 
        symbol: String, 
        total_supply: u64,
        decimals: u8,
    },
    /// Token transfer
    TokenTransfer { 
        to: String, 
        amount: u64, 
        token_id: String 
    },
    /// Staking transaction
    Stake { 
        amount: u64, 
        validator: String,
        duration: u64,
    },
    /// AI3 tensor computation
    TensorCompute { 
        operation: String, 
        input_data: Vec<f32>,
        expected_output_size: usize,
        max_computation_time: u64,
        reward: u64,
    },
    /// Contract deployment
    ContractDeploy {
        code: Vec<u8>,
        constructor_args: Vec<u8>,
    },
    /// Contract call
    ContractCall {
        contract_address: String,
        method: String,
        args: Vec<u8>,
        value: u64,
    },
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub transaction_type: TransactionType,
    pub fee: u64,
    pub timestamp: u64,
    pub nonce: u64,
    pub signature: String,
    pub hash: String,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        from: String,
        transaction_type: TransactionType,
        fee: u64,
        nonce: u64,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().timestamp() as u64;
        
        let mut tx = Transaction {
            id,
            from,
            transaction_type,
            fee,
            timestamp,
            nonce,
            signature: String::new(),
            hash: String::new(),
        };
        
        tx.hash = tx.calculate_hash();
        tx
    }

    /// Calculate transaction hash
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}{}",
            self.id,
            self.from,
            serde_json::to_string(&self.transaction_type).unwrap_or_default(),
            self.fee,
            self.timestamp,
            self.nonce
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Sign transaction (simplified - in real implementation would use proper cryptography)
    pub fn sign(&mut self, private_key: &str) -> TribeResult<()> {
        let message = format!("{}{}", self.hash, private_key);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        self.signature = hex::encode(hasher.finalize());
        Ok(())
    }

    /// Verify transaction signature
    pub fn verify_signature(&self, public_key: &str) -> bool {
        let message = format!("{}{}", self.hash, public_key);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let expected_signature = hex::encode(hasher.finalize());
        self.signature == expected_signature
    }

    /// Validate transaction
    pub fn validate(&self) -> TribeResult<bool> {
        // Check if hash is correct
        if self.hash != self.calculate_hash() {
            return Ok(false);
        }

        // Check timestamp (not too far in future)
        let now = Utc::now().timestamp() as u64;
        if self.timestamp > now + 300 { // 5 minutes tolerance
            return Ok(false);
        }

        // Validate transaction type specific rules
        match &self.transaction_type {
            TransactionType::Transfer { amount, .. } => {
                if *amount == 0 {
                    return Ok(false);
                }
            }
            TransactionType::TokenCreate { total_supply, decimals, .. } => {
                if *total_supply == 0 || *decimals > 18 {
                    return Ok(false);
                }
            }
            TransactionType::TokenTransfer { amount, .. } => {
                if *amount == 0 {
                    return Ok(false);
                }
            }
            TransactionType::Stake { amount, duration, .. } => {
                if *amount == 0 || *duration == 0 {
                    return Ok(false);
                }
            }
            TransactionType::TensorCompute { input_data, expected_output_size, .. } => {
                if input_data.is_empty() || *expected_output_size == 0 {
                    return Ok(false);
                }
            }
            TransactionType::ContractDeploy { code, .. } => {
                if code.is_empty() {
                    return Ok(false);
                }
            }
            TransactionType::ContractCall { contract_address, method, .. } => {
                if contract_address.is_empty() || method.is_empty() {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Get transaction size in bytes
    pub fn get_size(&self) -> usize {
        bincode::serialize(self).unwrap_or_default().len()
    }

    /// Get transaction fee per byte
    pub fn get_fee_per_byte(&self) -> f64 {
        let size = self.get_size();
        if size == 0 {
            0.0
        } else {
            self.fee as f64 / size as f64
        }
    }

    /// Check if transaction is a tensor computation
    pub fn is_tensor_compute(&self) -> bool {
        matches!(self.transaction_type, TransactionType::TensorCompute { .. })
    }

    /// Get tensor computation details if applicable
    pub fn get_tensor_compute_details(&self) -> Option<(&String, &Vec<f32>, usize, u64, u64)> {
        if let TransactionType::TensorCompute { 
            operation, 
            input_data, 
            expected_output_size, 
            max_computation_time, 
            reward 
        } = &self.transaction_type {
            Some((operation, input_data, *expected_output_size, *max_computation_time, *reward))
        } else {
            None
        }
    }
} 