// Re-export all workspace members
pub use tribechain_core::*;
pub use tribechain_mining::*;
pub use tribechain_contracts::*;
pub use tribechain_network::*;
pub use ai3_lib::*;

// Legacy modules for backward compatibility
pub mod blockchain;
pub mod block;
pub mod transaction;
pub mod wallet;
pub mod mining;
pub mod network;
pub mod storage;
pub mod contracts;
pub mod ai3;
pub mod tokens;
pub mod esp32_miner;

/// TribeChain version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default network port
pub const DEFAULT_PORT: u16 = 8333;

/// Block time target (in seconds)
pub const BLOCK_TIME_TARGET: u64 = 60;

/// Maximum block size
pub const MAX_BLOCK_SIZE: usize = 1_000_000;

/// AI3 tensor operation difficulty adjustment
pub const AI3_DIFFICULTY_ADJUSTMENT: u64 = 2016;

/// Supported token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    TribeChain,
    STOMP,
    AUM,
    AI3,
} 