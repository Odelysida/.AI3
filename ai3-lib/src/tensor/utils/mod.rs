use crate::tensor::{Tensor, TensorShape, TensorData};
use tribechain_core::{TribeResult, TribeError};
use ndarray::{Array, ArrayD, IxDyn};

impl Tensor {
    /// Get tensor as ndarray
    pub fn to_ndarray(&self) -> TribeResult<ArrayD<f32>> {
        let data = self.data.as_f32_vec()?;
        let shape: Vec<usize> = self.shape.dimensions.clone();
        
        Array::from_shape_vec(IxDyn(&shape), data)
            .map_err(|e| TribeError::InvalidOperation(format!("Failed to create ndarray: {}", e)))
    }

    /// Create tensor from ndarray
    pub fn from_ndarray(array: ArrayD<f32>, name: Option<String>) -> TribeResult<Self> {
        let shape = TensorShape::new(array.shape().to_vec());
        let data = TensorData::F32(array.into_raw_vec());
        Self::new(shape, data, name)
    }

    /// Reshape tensor
    pub fn reshape(&self, new_shape: TensorShape) -> TribeResult<Self> {
        if new_shape.total_elements() != self.shape.total_elements() {
            return Err(TribeError::InvalidOperation(
                format!("Cannot reshape tensor: {} elements to {} elements", 
                       self.shape.total_elements(), new_shape.total_elements())
            ));
        }

        Ok(Self {
            shape: new_shape,
            data: self.data.clone(),
            name: self.name.clone(),
        })
    }

    /// Get element at index
    pub fn get(&self, index: usize) -> TribeResult<f32> {
        let data = self.data.as_f32_vec()?;
        data.get(index)
            .copied()
            .ok_or_else(|| TribeError::InvalidOperation(format!("Index {} out of bounds", index)))
    }

    /// Get element at 2D coordinates
    pub fn get_2d(&self, row: usize, col: usize) -> TribeResult<f32> {
        if self.shape.rank() != 2 {
            return Err(TribeError::InvalidOperation("get_2d requires 2D tensor".to_string()));
        }
        
        let cols = self.shape.dimensions[1];
        let index = row * cols + col;
        self.get(index)
    }

    /// Set element at index
    pub fn set(&mut self, index: usize, value: f32) -> TribeResult<()> {
        match &mut self.data {
            TensorData::F32(ref mut vec) => {
                if index >= vec.len() {
                    return Err(TribeError::InvalidOperation(format!("Index {} out of bounds", index)));
                }
                vec[index] = value;
                Ok(())
            }
            _ => Err(TribeError::InvalidOperation("set() only supported for F32 tensors".to_string())),
        }
    }

    /// Calculate hash of tensor data
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        // Hash shape
        for dim in &self.shape.dimensions {
            hasher.update(dim.to_le_bytes());
        }
        
        // Hash data
        if let Ok(data) = self.data.as_f32_vec() {
            for value in data {
                hasher.update(value.to_le_bytes());
            }
        }
        
        hex::encode(hasher.finalize())
    }

    /// Check if tensor is compatible with ESP devices
    pub fn is_esp_compatible(&self) -> bool {
        // ESP devices have limited memory and precision
        let total_elements = self.shape.total_elements();
        total_elements <= 1024 && matches!(self.data, TensorData::F32(_) | TensorData::I32(_))
    }

    /// Convert tensor to ESP-compatible format (fixed-point)
    pub fn to_esp_format(&self) -> TribeResult<Vec<i16>> {
        let data = self.data.as_f32_vec()?;
        
        // Convert to 16-bit fixed point (Q8.8 format)
        let fixed_point: Vec<i16> = data.iter()
            .map(|&x| (x * 256.0).clamp(-32768.0, 32767.0) as i16)
            .collect();
            
        Ok(fixed_point)
    }
} 