use serde::{Deserialize, Serialize};
use ndarray::{Array, ArrayD, IxDyn};
use std::fmt;
use tribechain_core::{TribeResult, TribeError};

/// Tensor shape representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TensorShape {
    pub dimensions: Vec<usize>,
}

impl TensorShape {
    pub fn new(dimensions: Vec<usize>) -> Self {
        Self { dimensions }
    }

    pub fn scalar() -> Self {
        Self { dimensions: vec![] }
    }

    pub fn vector(size: usize) -> Self {
        Self { dimensions: vec![size] }
    }

    pub fn matrix(rows: usize, cols: usize) -> Self {
        Self { dimensions: vec![rows, cols] }
    }

    pub fn tensor_3d(d1: usize, d2: usize, d3: usize) -> Self {
        Self { dimensions: vec![d1, d2, d3] }
    }

    pub fn total_elements(&self) -> usize {
        self.dimensions.iter().product()
    }

    pub fn rank(&self) -> usize {
        self.dimensions.len()
    }

    pub fn is_compatible_for_matmul(&self, other: &TensorShape) -> bool {
        if self.rank() != 2 || other.rank() != 2 {
            return false;
        }
        self.dimensions[1] == other.dimensions[0]
    }
}

impl fmt::Display for TensorShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.dimensions.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "))
    }
}

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

    /// Get tensor as ndarray
    pub fn to_ndarray(&self) -> TribeResult<ArrayD<f32>> {
        let data = self.data.as_f32_vec()?;
        let shape: Vec<usize> = self.shape.dimensions.clone();
        
        Array::from_shape_vec(IxDyn(&shape), data)
            .map_err(|e| TribeError::InvalidOperation(format!("Failed to create ndarray: {}", e)))
    }

    /// Create tensor from ndarray
    pub fn from_ndarray(array: ArrayD<f32>) -> Self {
        let shape = TensorShape::new(array.shape().to_vec());
        let data = array.into_raw_vec();
        
        Self {
            shape,
            data: TensorData::F32(data),
            name: None,
        }
    }

    /// Reshape tensor
    pub fn reshape(&self, new_shape: TensorShape) -> TribeResult<Self> {
        if new_shape.total_elements() != self.shape.total_elements() {
            return Err(TribeError::InvalidOperation(
                "New shape must have same total elements".to_string()
            ));
        }

        Ok(Self {
            shape: new_shape,
            data: self.data.clone(),
            name: self.name.clone(),
        })
    }

    /// Get element at index (for vectors)
    pub fn get(&self, index: usize) -> TribeResult<f32> {
        let data = self.data.as_f32_slice()?;
        data.get(index)
            .copied()
            .ok_or_else(|| TribeError::InvalidOperation("Index out of bounds".to_string()))
    }

    /// Get element at 2D index (for matrices)
    pub fn get_2d(&self, row: usize, col: usize) -> TribeResult<f32> {
        if self.shape.rank() != 2 {
            return Err(TribeError::InvalidOperation("Tensor is not 2D".to_string()));
        }

        let cols = self.shape.dimensions[1];
        let index = row * cols + col;
        self.get(index)
    }

    /// Set element at index (for vectors)
    pub fn set(&mut self, index: usize, value: f32) -> TribeResult<()> {
        match &mut self.data {
            TensorData::F32(v) => {
                if index < v.len() {
                    v[index] = value;
                    Ok(())
                } else {
                    Err(TribeError::InvalidOperation("Index out of bounds".to_string()))
                }
            }
            _ => Err(TribeError::InvalidOperation("Expected F32 tensor data".to_string())),
        }
    }

    /// Calculate tensor hash for mining
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        
        let serialized = bincode::serialize(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hex::encode(hasher.finalize())
    }

    /// Check if tensor is valid for ESP32/ESP8266 (small size constraints)
    pub fn is_esp_compatible(&self) -> bool {
        // ESP32 has limited memory, so we limit tensor size
        const MAX_ESP_ELEMENTS: usize = 1024; // Adjust based on available memory
        self.shape.total_elements() <= MAX_ESP_ELEMENTS
    }

    /// Convert to ESP-compatible format (quantized if needed)
    pub fn to_esp_format(&self) -> TribeResult<Vec<i16>> {
        let data = self.data.as_f32_vec()?;
        
        // Quantize to 16-bit integers for ESP compatibility
        let quantized: Vec<i16> = data.iter()
            .map(|&x| (x * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();
            
        Ok(quantized)
    }
}

impl fmt::Display for Tensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tensor{}", self.shape)?;
        if let Some(name) = &self.name {
            write!(f, " ({})", name)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let tensor = Tensor::vector(vec![1.0, 2.0, 3.0]);
        assert_eq!(tensor.shape.dimensions, vec![3]);
        assert_eq!(tensor.data.len(), 3);
    }

    #[test]
    fn test_matrix_creation() {
        let tensor = Tensor::matrix(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
        assert_eq!(tensor.shape.dimensions, vec![2, 2]);
        assert_eq!(tensor.get_2d(0, 1).unwrap(), 2.0);
        assert_eq!(tensor.get_2d(1, 0).unwrap(), 3.0);
    }

    #[test]
    fn test_tensor_reshape() {
        let tensor = Tensor::vector(vec![1.0, 2.0, 3.0, 4.0]);
        let reshaped = tensor.reshape(TensorShape::matrix(2, 2)).unwrap();
        assert_eq!(reshaped.shape.dimensions, vec![2, 2]);
    }

    #[test]
    fn test_esp_compatibility() {
        let small_tensor = Tensor::vector(vec![1.0; 100]);
        assert!(small_tensor.is_esp_compatible());

        let large_tensor = Tensor::vector(vec![1.0; 2000]);
        assert!(!large_tensor.is_esp_compatible());
    }
} 