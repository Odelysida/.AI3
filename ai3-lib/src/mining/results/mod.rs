use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::tensor::Tensor;
use crate::mining::tasks::MiningTask;
use tribechain_core::TribeResult;

/// Mining result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    pub task_id: String,
    pub miner_id: String,
    pub nonce: u64,
    pub hash: String,
    pub output_tensor: Tensor,
    pub computation_time: u64, // milliseconds
    pub timestamp: DateTime<Utc>,
    pub is_valid: bool,
}

impl MiningResult {
    pub fn new(
        task_id: String,
        miner_id: String,
        nonce: u64,
        hash: String,
        output_tensor: Tensor,
        computation_time: u64,
    ) -> Self {
        Self {
            task_id,
            miner_id,
            nonce,
            hash,
            output_tensor,
            computation_time,
            timestamp: Utc::now(),
            is_valid: false, // Will be validated by network
        }
    }

    pub fn validate(&mut self, task: &MiningTask) -> TribeResult<bool> {
        // Check if hash meets difficulty
        if !task.meets_difficulty(&self.hash) {
            self.is_valid = false;
            return Ok(false);
        }

        // Verify hash calculation
        let expected_hash = task.calculate_hash(self.nonce);
        if expected_hash != self.hash {
            self.is_valid = false;
            return Ok(false);
        }

        // Verify tensor computation
        let expected_output = task.execute_operation()?;
        if !self.tensors_approximately_equal(&self.output_tensor, &expected_output) {
            self.is_valid = false;
            return Ok(false);
        }

        self.is_valid = true;
        Ok(true)
    }

    fn tensors_approximately_equal(&self, a: &Tensor, b: &Tensor) -> bool {
        if a.shape != b.shape {
            return false;
        }

        let a_data = a.data.as_f32_vec().unwrap_or_default();
        let b_data = b.data.as_f32_vec().unwrap_or_default();

        if a_data.len() != b_data.len() {
            return false;
        }

        const EPSILON: f32 = 1e-6;
        a_data.iter().zip(b_data.iter()).all(|(x, y)| (x - y).abs() < EPSILON)
    }
} 