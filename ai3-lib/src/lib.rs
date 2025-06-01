pub mod tensor;
pub mod operations;
pub mod mining;
pub mod esp_compat;

// Re-export main types
pub use tensor::{Tensor, TensorShape, TensorData};
pub use operations::{TensorOp, MatrixMultiply, Convolution, ActivationFunction, VectorOp};
pub use mining::{AI3Miner, MiningTask, MiningResult, TaskDistributor};
pub use esp_compat::{ESP32Miner, ESP8266Miner, ESPMiningConfig};

use tribechain_core::TribeResult;

/// AI3 Engine for tensor operations and distributed computing
pub struct AI3Engine {
    pub task_distributor: TaskDistributor,
    pub active_miners: Vec<AI3Miner>,
}

impl AI3Engine {
    pub fn new() -> Self {
        Self {
            task_distributor: TaskDistributor::new(),
            active_miners: Vec::new(),
        }
    }

    pub fn register_miner(&mut self, miner: AI3Miner) -> TribeResult<()> {
        self.active_miners.push(miner);
        Ok(())
    }

    pub fn distribute_task(&mut self, task: MiningTask) -> TribeResult<Vec<String>> {
        self.task_distributor.distribute(task, &self.active_miners)
    }

    pub fn collect_results(&self) -> Vec<MiningResult> {
        self.active_miners
            .iter()
            .filter_map(|miner| miner.get_latest_result())
            .collect()
    }
}

impl Default for AI3Engine {
    fn default() -> Self {
        Self::new()
    }
} 