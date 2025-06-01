use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape};
use crate::operations::TensorOp;
use tribechain_core::{TribeResult, TribeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorOpType {
    DotProduct,
    CrossProduct,
    Normalize,
    Add,
    Subtract,
    ElementwiseMultiply,
    ElementwiseDivide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorOp {
    pub op_type: VectorOpType,
}

impl VectorOp {
    pub fn dot_product() -> Self {
        Self { op_type: VectorOpType::DotProduct }
    }

    pub fn normalize() -> Self {
        Self { op_type: VectorOpType::Normalize }
    }

    pub fn add() -> Self {
        Self { op_type: VectorOpType::Add }
    }

    pub fn subtract() -> Self {
        Self { op_type: VectorOpType::Subtract }
    }

    pub fn elementwise_multiply() -> Self {
        Self { op_type: VectorOpType::ElementwiseMultiply }
    }
}

impl TensorOp for VectorOp {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor> {
        self.validate_inputs(inputs)?;

        match self.op_type {
            VectorOpType::DotProduct => {
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                Ok(Tensor::scalar(result))
            }
            VectorOpType::Normalize => {
                let input_data = inputs[0].data.as_f32_vec()?;
                let magnitude: f32 = input_data.iter().map(|x| x * x).sum::<f32>().sqrt();
                
                if magnitude == 0.0 {
                    return Err(TribeError::InvalidOperation("Cannot normalize zero vector".to_string()));
                }
                
                let normalized: Vec<f32> = input_data.iter().map(|x| x / magnitude).collect();
                Tensor::from_vec(normalized, inputs[0].shape.clone())
            }
            VectorOpType::Add => {
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
                Tensor::from_vec(result, inputs[0].shape.clone())
            }
            VectorOpType::Subtract => {
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
                Tensor::from_vec(result, inputs[0].shape.clone())
            }
            VectorOpType::ElementwiseMultiply => {
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| x * y).collect();
                Tensor::from_vec(result, inputs[0].shape.clone())
            }
            VectorOpType::ElementwiseDivide => {
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result: Vec<f32> = a.iter().zip(b.iter()).map(|(x, y)| {
                    if *y == 0.0 {
                        f32::INFINITY
                    } else {
                        x / y
                    }
                }).collect();
                Tensor::from_vec(result, inputs[0].shape.clone())
            }
            VectorOpType::CrossProduct => {
                // 3D cross product only
                if inputs[0].shape.total_elements() != 3 || inputs[1].shape.total_elements() != 3 {
                    return Err(TribeError::InvalidOperation("Cross product requires 3D vectors".to_string()));
                }
                
                let a = inputs[0].data.as_f32_vec()?;
                let b = inputs[1].data.as_f32_vec()?;
                
                let result = vec![
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ];
                
                Tensor::from_vec(result, TensorShape::vector(3))
            }
        }
    }

    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()> {
        match self.op_type {
            VectorOpType::Normalize => {
                if inputs.len() != 1 {
                    return Err(TribeError::InvalidOperation("Normalize requires exactly 1 input".to_string()));
                }
            }
            _ => {
                if inputs.len() != 2 {
                    return Err(TribeError::InvalidOperation("Binary vector operation requires exactly 2 inputs".to_string()));
                }
                
                // Check compatible shapes for binary operations
                if inputs[0].shape.total_elements() != inputs[1].shape.total_elements() {
                    return Err(TribeError::InvalidOperation("Input tensors must have same number of elements".to_string()));
                }
            }
        }
        Ok(())
    }

    fn get_operation_name(&self) -> &str {
        match self.op_type {
            VectorOpType::DotProduct => "dot_product",
            VectorOpType::CrossProduct => "cross_product",
            VectorOpType::Normalize => "normalize",
            VectorOpType::Add => "vector_add",
            VectorOpType::Subtract => "vector_subtract",
            VectorOpType::ElementwiseMultiply => "elementwise_multiply",
            VectorOpType::ElementwiseDivide => "elementwise_divide",
        }
    }

    fn get_complexity_score(&self) -> u64 {
        match self.op_type {
            VectorOpType::DotProduct => 50,
            VectorOpType::CrossProduct => 30,
            VectorOpType::Normalize => 100, // Includes sqrt
            VectorOpType::Add | VectorOpType::Subtract => 20,
            VectorOpType::ElementwiseMultiply | VectorOpType::ElementwiseDivide => 25,
        }
    }
} 