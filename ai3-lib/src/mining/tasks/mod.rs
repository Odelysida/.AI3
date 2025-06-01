use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::tensor::Tensor;
use crate::operations::{TensorOp, MatrixMultiply, Convolution, ActivationFunction, VectorOp};
use tribechain_core::{TribeResult, TribeError};

/// Mining task for tensor operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningTask {
    pub id: String,
    pub operation_type: String,
    pub input_tensors: Vec<Tensor>,
    pub expected_output_shape: Option<Vec<usize>>,
    pub difficulty: u64,
    pub reward: u64,
    pub max_computation_time: u64, // seconds
    pub created_at: DateTime<Utc>,
    pub requester: String,
    pub nonce_range: (u64, u64), // Range for mining nonce
}

impl MiningTask {
    pub fn new(
        operation_type: String,
        input_tensors: Vec<Tensor>,
        difficulty: u64,
        reward: u64,
        max_computation_time: u64,
        requester: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation_type,
            input_tensors,
            expected_output_shape: None,
            difficulty,
            reward,
            max_computation_time,
            created_at: Utc::now(),
            requester,
            nonce_range: (0, u64::MAX),
        }
    }

    pub fn with_expected_output(mut self, shape: Vec<usize>) -> Self {
        self.expected_output_shape = Some(shape);
        self
    }

    pub fn with_nonce_range(mut self, start: u64, end: u64) -> Self {
        self.nonce_range = (start, end);
        self
    }

    /// Calculate task hash for mining
    pub fn calculate_hash(&self, nonce: u64) -> String {
        use sha2::{Digest, Sha256};
        
        let task_data = format!(
            "{}{}{}{}{}{}",
            self.id,
            self.operation_type,
            self.difficulty,
            self.reward,
            self.requester,
            nonce
        );
        
        let mut hasher = Sha256::new();
        hasher.update(task_data.as_bytes());
        
        // Include tensor data in hash
        for tensor in &self.input_tensors {
            hasher.update(tensor.calculate_hash().as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }

    /// Check if hash meets difficulty target
    pub fn meets_difficulty(&self, hash: &str) -> bool {
        let leading_zeros = hash.chars().take_while(|&c| c == '0').count();
        leading_zeros >= (self.difficulty as usize)
    }

    /// Get operation instance
    pub fn get_operation(&self) -> TribeResult<Box<dyn TensorOp>> {
        match self.operation_type.as_str() {
            "matrix_multiply" => Ok(Box::new(MatrixMultiply::new())),
            "convolution" => Ok(Box::new(Convolution::new(3))), // Default kernel size
            "relu" => Ok(Box::new(ActivationFunction::relu())),
            "sigmoid" => Ok(Box::new(ActivationFunction::sigmoid())),
            "tanh" => Ok(Box::new(ActivationFunction::tanh())),
            "softmax" => Ok(Box::new(ActivationFunction::softmax())),
            "dot_product" => Ok(Box::new(VectorOp::dot_product())),
            "normalize" => Ok(Box::new(VectorOp::normalize())),
            "vector_add" => Ok(Box::new(VectorOp::add())),
            _ => Err(TribeError::InvalidOperation(format!("Unknown operation type: {}", self.operation_type))),
        }
    }

    /// Execute the tensor operation
    pub fn execute_operation(&self) -> TribeResult<Tensor> {
        let operation = self.get_operation()?;
        operation.execute(&self.input_tensors)
    }

    /// Check if task is expired
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.created_at);
        elapsed.num_seconds() > self.max_computation_time as i64
    }
} 