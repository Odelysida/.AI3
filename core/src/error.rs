use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type for TribeChain operations
pub type TribeResult<T> = Result<T, TribeError>;

/// Error types for TribeChain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TribeError {
    /// Invalid block error
    InvalidBlock(String),
    /// Invalid transaction error
    InvalidTransaction(String),
    /// Mining error
    Mining(String),
    /// Storage error
    Storage(String),
    /// Network error
    Network(String),
    /// Token error
    Token(String),
    /// AI3 engine error
    AI3(String),
    /// Blockchain error
    Blockchain(String),
    /// Contract error
    Contract(String),
    /// Generic error
    Generic(String),
}

impl fmt::Display for TribeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TribeError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            TribeError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            TribeError::Mining(msg) => write!(f, "Mining error: {}", msg),
            TribeError::Storage(msg) => write!(f, "Storage error: {}", msg),
            TribeError::Network(msg) => write!(f, "Network error: {}", msg),
            TribeError::Token(msg) => write!(f, "Token error: {}", msg),
            TribeError::AI3(msg) => write!(f, "AI3 error: {}", msg),
            TribeError::Blockchain(msg) => write!(f, "Blockchain error: {}", msg),
            TribeError::Contract(msg) => write!(f, "Contract error: {}", msg),
            TribeError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TribeError {} 