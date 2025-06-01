pub mod error;
pub mod block;
pub mod transaction;
pub mod blockchain;
pub mod storage;

// Re-export main types
pub use error::{TribeError, TribeResult};
pub use block::{Block, AI3Proof};
pub use transaction::{Transaction, TransactionType};
pub use blockchain::{TribeChain, MinerInfo, TensorTask, BlockchainStats};
pub use storage::{Storage, StorageStats}; 