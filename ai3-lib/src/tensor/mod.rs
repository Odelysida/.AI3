use serde::{Deserialize, Serialize};
use std::fmt;
use tribechain_core::{TribeResult, TribeError};

pub mod shape;
pub mod data;
pub mod utils;
pub mod tests;

// Re-export main types
pub use shape::TensorShape;
pub use data::TensorData;

/// Main tensor structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub shape: TensorShape,
    pub data: TensorData,
    pub name: Option<String>,
}

impl Tensor {
    /// Create a new tensor
    pub fn new(shape: TensorShape, data: TensorData, name: Option<String>) -> TribeResult<Self> {
        if shape.total_elements() != data.len() {
            return Err(TribeError::InvalidOperation(
                format!("Shape {} doesn't match data length {}", shape, data.len())
            ));
        }

        Ok(Self { shape, data, name })
    }

    /// Create tensor from f32 vector
    pub fn from_vec(data: Vec<f32>, shape: TensorShape) -> TribeResult<Self> {
        Self::new(shape, TensorData::F32(data), None)
    }

    /// Create scalar tensor
    pub fn scalar(value: f32) -> Self {
        Self {
            shape: TensorShape::scalar(),
            data: TensorData::F32(vec![value]),
            name: None,
        }
    }

    /// Create vector tensor
    pub fn vector(data: Vec<f32>) -> Self {
        let len = data.len();
        Self {
            shape: TensorShape::vector(len),
            data: TensorData::F32(data),
            name: None,
        }
    }

    /// Create matrix tensor
    pub fn matrix(data: Vec<f32>, rows: usize, cols: usize) -> TribeResult<Self> {
        if data.len() != rows * cols {
            return Err(TribeError::InvalidOperation(
                format!("Data length {} doesn't match matrix dimensions {}x{}", data.len(), rows, cols)
            ));
        }

        Ok(Self {
            shape: TensorShape::matrix(rows, cols),
            data: TensorData::F32(data),
            name: None,
        })
    }

    /// Create zeros tensor
    pub fn zeros(shape: TensorShape) -> Self {
        let total_elements = shape.total_elements();
        Self {
            shape,
            data: TensorData::F32(vec![0.0; total_elements]),
            name: None,
        }
    }

    /// Create ones tensor
    pub fn ones(shape: TensorShape) -> Self {
        let total_elements = shape.total_elements();
        Self {
            shape,
            data: TensorData::F32(vec![1.0; total_elements]),
            name: None,
        }
    }

    /// Create random tensor
    pub fn random(shape: TensorShape) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let total_elements = shape.total_elements();
        let data: Vec<f32> = (0..total_elements).map(|_| rng.gen_range(-1.0..1.0)).collect();
        
        Self {
            shape,
            data: TensorData::F32(data),
            name: None,
        }
    }
}

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor(shape: {}, name: {:?})", self.shape, self.name)
    }
} 