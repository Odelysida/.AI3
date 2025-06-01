pub mod tasks;
pub mod miners;
pub mod distributors;
pub mod results;
pub mod tests;

// Re-export main types for convenience
pub use tasks::MiningTask;
pub use miners::{AI3Miner, MinerCapabilities, MinerStats};
pub use distributors::TaskDistributor;
pub use results::MiningResult; 