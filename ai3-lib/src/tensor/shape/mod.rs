use serde::{Deserialize, Serialize};
use std::fmt;

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