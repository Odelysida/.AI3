use serde::{Deserialize, Serialize};
use tribechain_core::{TribeResult, TribeError};

/// Tensor data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TensorData {
    F32(Vec<f32>),
    F64(Vec<f64>),
    I32(Vec<i32>),
    I64(Vec<i64>),
    Bool(Vec<bool>),
}

impl TensorData {
    pub fn len(&self) -> usize {
        match self {
            TensorData::F32(v) => v.len(),
            TensorData::F64(v) => v.len(),
            TensorData::I32(v) => v.len(),
            TensorData::I64(v) => v.len(),
            TensorData::Bool(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_f32_slice(&self) -> TribeResult<&[f32]> {
        match self {
            TensorData::F32(v) => Ok(v),
            _ => Err(TribeError::InvalidOperation("Expected F32 tensor data".to_string())),
        }
    }

    pub fn as_f32_vec(&self) -> TribeResult<Vec<f32>> {
        match self {
            TensorData::F32(v) => Ok(v.clone()),
            TensorData::F64(v) => Ok(v.iter().map(|&x| x as f32).collect()),
            TensorData::I32(v) => Ok(v.iter().map(|&x| x as f32).collect()),
            TensorData::I64(v) => Ok(v.iter().map(|&x| x as f32).collect()),
            TensorData::Bool(v) => Ok(v.iter().map(|&x| if x { 1.0 } else { 0.0 }).collect()),
        }
    }
} 