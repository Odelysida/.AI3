use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape};
use crate::operations::TensorOp;
use tribechain_core::{TribeResult, TribeError};

/// Matrix multiplication operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixMultiply {
    pub transpose_a: bool,
    pub transpose_b: bool,
}

impl MatrixMultiply {
    pub fn new() -> Self {
        Self {
            transpose_a: false,
            transpose_b: false,
        }
    }

    pub fn with_transpose(transpose_a: bool, transpose_b: bool) -> Self {
        Self {
            transpose_a,
            transpose_b,
        }
    }
}

impl Default for MatrixMultiply {
    fn default() -> Self {
        Self::new()
    }
}

impl TensorOp for MatrixMultiply {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor> {
        self.validate_inputs(inputs)?;

        let a = &inputs[0];
        let b = &inputs[1];

        let a_array = a.to_ndarray()?;
        let b_array = b.to_ndarray()?;

        // Convert to 2D arrays
        let a_2d = a_array.into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| TribeError::InvalidOperation(format!("Failed to convert tensor A to 2D: {}", e)))?;
        let b_2d = b_array.into_dimensionality::<ndarray::Ix2>()
            .map_err(|e| TribeError::InvalidOperation(format!("Failed to convert tensor B to 2D: {}", e)))?;

        // Apply transpose if needed
        let a_final = if self.transpose_a { a_2d.t().to_owned() } else { a_2d };
        let b_final = if self.transpose_b { b_2d.t().to_owned() } else { b_2d };

        // Perform matrix multiplication
        let result = a_final.dot(&b_final);

        // Convert back to tensor
        let result_shape = TensorShape::matrix(result.nrows(), result.ncols());
        let result_data = result.into_raw_vec();

        Tensor::from_vec(result_data, result_shape)
    }

    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()> {
        if inputs.len() != 2 {
            return Err(TribeError::InvalidOperation("Matrix multiply requires exactly 2 inputs".to_string()));
        }

        let a = &inputs[0];
        let b = &inputs[1];

        if a.shape.rank() != 2 || b.shape.rank() != 2 {
            return Err(TribeError::InvalidOperation("Both inputs must be 2D matrices".to_string()));
        }

        let a_cols = if self.transpose_a { a.shape.dimensions[0] } else { a.shape.dimensions[1] };
        let b_rows = if self.transpose_b { b.shape.dimensions[1] } else { b.shape.dimensions[0] };

        if a_cols != b_rows {
            return Err(TribeError::InvalidOperation(
                format!("Matrix dimensions incompatible: {} vs {}", a_cols, b_rows)
            ));
        }

        Ok(())
    }

    fn get_operation_name(&self) -> &str {
        "matrix_multiply"
    }

    fn get_complexity_score(&self) -> u64 {
        // O(n^3) complexity for matrix multiplication
        1000
    }
} 