use crate::tensor::Tensor;
use tribechain_core::TribeResult;

pub mod matrix;
pub mod convolution;
pub mod activation;
pub mod vector;
pub mod tests;

/// Trait for tensor operations
pub trait TensorOp {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor>;
    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()>;
    fn get_operation_name(&self) -> &str;
    fn get_complexity_score(&self) -> u64;
}

// Re-export main types for convenience
pub use matrix::MatrixMultiply;
pub use convolution::Convolution;
pub use activation::{ActivationFunction, ActivationType};
pub use vector::{VectorOp, VectorOpType}; 