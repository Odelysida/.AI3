use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape, TensorData};
use tribechain_core::{TribeResult, TribeError};
use ndarray::{Array2, Array1, Axis};

/// Trait for tensor operations
pub trait TensorOp {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor>;
    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()>;
    fn get_operation_name(&self) -> &str;
    fn get_complexity_score(&self) -> u64;
}

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

/// Convolution operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Convolution {
    pub kernel_size: usize,
    pub stride: usize,
    pub padding: usize,
    pub dilation: usize,
}

impl Convolution {
    pub fn new(kernel_size: usize) -> Self {
        Self {
            kernel_size,
            stride: 1,
            padding: 0,
            dilation: 1,
        }
    }

    pub fn with_params(kernel_size: usize, stride: usize, padding: usize, dilation: usize) -> Self {
        Self {
            kernel_size,
            stride,
            padding,
            dilation,
        }
    }

    fn calculate_output_size(&self, input_size: usize) -> usize {
        let effective_kernel_size = self.dilation * (self.kernel_size - 1) + 1;
        (input_size + 2 * self.padding - effective_kernel_size) / self.stride + 1
    }
}

impl TensorOp for Convolution {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor> {
        self.validate_inputs(inputs)?;

        let input = &inputs[0];
        let kernel = &inputs[1];

        let input_data = input.data.as_f32_vec()?;
        let kernel_data = kernel.data.as_f32_vec()?;

        // Simple 1D convolution implementation
        if input.shape.rank() == 1 && kernel.shape.rank() == 1 {
            let input_size = input.shape.dimensions[0];
            let output_size = self.calculate_output_size(input_size);
            let mut output = vec![0.0; output_size];

            for i in 0..output_size {
                let mut sum = 0.0;
                for j in 0..self.kernel_size {
                    let input_idx = i * self.stride + j * self.dilation;
                    if input_idx < input_size {
                        sum += input_data[input_idx] * kernel_data[j];
                    }
                }
                output[i] = sum;
            }

            return Tensor::from_vec(output, TensorShape::vector(output_size));
        }

        // 2D convolution for matrices
        if input.shape.rank() == 2 && kernel.shape.rank() == 2 {
            let input_h = input.shape.dimensions[0];
            let input_w = input.shape.dimensions[1];
            let kernel_h = kernel.shape.dimensions[0];
            let kernel_w = kernel.shape.dimensions[1];

            let output_h = self.calculate_output_size(input_h);
            let output_w = self.calculate_output_size(input_w);
            let mut output = vec![0.0; output_h * output_w];

            for out_y in 0..output_h {
                for out_x in 0..output_w {
                    let mut sum = 0.0;
                    for ky in 0..kernel_h {
                        for kx in 0..kernel_w {
                            let in_y = out_y * self.stride + ky * self.dilation;
                            let in_x = out_x * self.stride + kx * self.dilation;
                            
                            if in_y < input_h && in_x < input_w {
                                let input_idx = in_y * input_w + in_x;
                                let kernel_idx = ky * kernel_w + kx;
                                sum += input_data[input_idx] * kernel_data[kernel_idx];
                            }
                        }
                    }
                    output[out_y * output_w + out_x] = sum;
                }
            }

            return Tensor::from_vec(output, TensorShape::matrix(output_h, output_w));
        }

        Err(TribeError::InvalidOperation("Unsupported convolution dimensions".to_string()))
    }

    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()> {
        if inputs.len() != 2 {
            return Err(TribeError::InvalidOperation("Convolution requires exactly 2 inputs (input and kernel)".to_string()));
        }

        let input = &inputs[0];
        let kernel = &inputs[1];

        if input.shape.rank() != kernel.shape.rank() {
            return Err(TribeError::InvalidOperation("Input and kernel must have same number of dimensions".to_string()));
        }

        if input.shape.rank() < 1 || input.shape.rank() > 2 {
            return Err(TribeError::InvalidOperation("Only 1D and 2D convolutions are supported".to_string()));
        }

        Ok(())
    }

    fn get_operation_name(&self) -> &str {
        "convolution"
    }

    fn get_complexity_score(&self) -> u64 {
        // Complexity depends on kernel size and output size
        (self.kernel_size * self.kernel_size) as u64 * 100
    }
}

/// Activation function types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    LeakyReLU(f32),
    Softmax,
}

/// Activation function operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationFunction {
    pub activation_type: ActivationType,
}

impl ActivationFunction {
    pub fn relu() -> Self {
        Self { activation_type: ActivationType::ReLU }
    }

    pub fn sigmoid() -> Self {
        Self { activation_type: ActivationType::Sigmoid }
    }

    pub fn tanh() -> Self {
        Self { activation_type: ActivationType::Tanh }
    }

    pub fn leaky_relu(alpha: f32) -> Self {
        Self { activation_type: ActivationType::LeakyReLU(alpha) }
    }

    pub fn softmax() -> Self {
        Self { activation_type: ActivationType::Softmax }
    }

    fn apply_activation(&self, x: f32) -> f32 {
        match self.activation_type {
            ActivationType::ReLU => x.max(0.0),
            ActivationType::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationType::Tanh => x.tanh(),
            ActivationType::LeakyReLU(alpha) => if x > 0.0 { x } else { alpha * x },
            ActivationType::Softmax => x, // Softmax is handled separately
        }
    }
}

impl TensorOp for ActivationFunction {
    fn execute(&self, inputs: &[Tensor]) -> TribeResult<Tensor> {
        self.validate_inputs(inputs)?;

        let input = &inputs[0];
        let input_data = input.data.as_f32_vec()?;

        let output_data = match self.activation_type {
            ActivationType::Softmax => {
                // Softmax requires special handling
                let max_val = input_data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let exp_values: Vec<f32> = input_data.iter().map(|&x| (x - max_val).exp()).collect();
                let sum_exp: f32 = exp_values.iter().sum();
                exp_values.iter().map(|&x| x / sum_exp).collect()
            }
            _ => {
                input_data.iter().map(|&x| self.apply_activation(x)).collect()
            }
        };

        Tensor::from_vec(output_data, input.shape.clone())
    }

    fn validate_inputs(&self, inputs: &[Tensor]) -> TribeResult<()> {
        if inputs.len() != 1 {
            return Err(TribeError::InvalidOperation("Activation function requires exactly 1 input".to_string()));
        }
        Ok(())
    }

    fn get_operation_name(&self) -> &str {
        match self.activation_type {
            ActivationType::ReLU => "relu",
            ActivationType::Sigmoid => "sigmoid",
            ActivationType::Tanh => "tanh",
            ActivationType::LeakyReLU(_) => "leaky_relu",
            ActivationType::Softmax => "softmax",
        }
    }

    fn get_complexity_score(&self) -> u64 {
        match self.activation_type {
            ActivationType::ReLU => 10,
            ActivationType::Sigmoid => 50,
            ActivationType::Tanh => 50,
            ActivationType::LeakyReLU(_) => 15,
            ActivationType::Softmax => 100,
        }
    }
}

/// Vector operations
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

/// Vector operation
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
                let input = inputs[0].data.as_f32_vec()?;
                let magnitude: f32 = input.iter().map(|x| x * x).sum::<f32>().sqrt();
                if magnitude == 0.0 {
                    return Err(TribeError::InvalidOperation("Cannot normalize zero vector".to_string()));
                }
                let normalized: Vec<f32> = input.iter().map(|x| x / magnitude).collect();
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
                    if *y == 0.0 { f32::INFINITY } else { x / y }
                }).collect();
                Tensor::from_vec(result, inputs[0].shape.clone())
            }
            VectorOpType::CrossProduct => {
                if inputs[0].shape.dimensions != vec![3] || inputs[1].shape.dimensions != vec![3] {
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
                if inputs[0].shape.rank() != 1 {
                    return Err(TribeError::InvalidOperation("Normalize requires vector input".to_string()));
                }
            }
            VectorOpType::DotProduct | VectorOpType::CrossProduct | 
            VectorOpType::Add | VectorOpType::Subtract | 
            VectorOpType::ElementwiseMultiply | VectorOpType::ElementwiseDivide => {
                if inputs.len() != 2 {
                    return Err(TribeError::InvalidOperation("Binary operations require exactly 2 inputs".to_string()));
                }
                if inputs[0].shape != inputs[1].shape {
                    return Err(TribeError::InvalidOperation("Input tensors must have same shape".to_string()));
                }
                if inputs[0].shape.rank() != 1 {
                    return Err(TribeError::InvalidOperation("Vector operations require vector inputs".to_string()));
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
            VectorOpType::Normalize => 100,
            VectorOpType::Add | VectorOpType::Subtract => 10,
            VectorOpType::ElementwiseMultiply | VectorOpType::ElementwiseDivide => 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() {
        let a = Tensor::matrix(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
        let b = Tensor::matrix(vec![5.0, 6.0, 7.0, 8.0], 2, 2).unwrap();
        
        let op = MatrixMultiply::new();
        let result = op.execute(&[a, b]).unwrap();
        
        assert_eq!(result.shape.dimensions, vec![2, 2]);
        // Expected result: [[19, 22], [43, 50]]
        assert_eq!(result.get_2d(0, 0).unwrap(), 19.0);
        assert_eq!(result.get_2d(0, 1).unwrap(), 22.0);
        assert_eq!(result.get_2d(1, 0).unwrap(), 43.0);
        assert_eq!(result.get_2d(1, 1).unwrap(), 50.0);
    }

    #[test]
    fn test_relu_activation() {
        let input = Tensor::vector(vec![-1.0, 0.0, 1.0, 2.0]);
        let op = ActivationFunction::relu();
        let result = op.execute(&[input]).unwrap();
        
        let expected = vec![0.0, 0.0, 1.0, 2.0];
        let result_data = result.data.as_f32_vec().unwrap();
        assert_eq!(result_data, expected);
    }

    #[test]
    fn test_vector_dot_product() {
        let a = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let b = Tensor::vector(vec![4.0, 5.0, 6.0]);
        
        let op = VectorOp::dot_product();
        let result = op.execute(&[a, b]).unwrap();
        
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert_eq!(result.get(0).unwrap(), 32.0);
    }

    #[test]
    fn test_convolution_1d() {
        let input = Tensor::vector(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let kernel = Tensor::vector(vec![1.0, 0.0, -1.0]);
        
        let op = Convolution::new(3);
        let result = op.execute(&[input, kernel]).unwrap();
        
        assert_eq!(result.shape.dimensions, vec![3]);
        // Expected: [1*1 + 2*0 + 3*(-1), 2*1 + 3*0 + 4*(-1), 3*1 + 4*0 + 5*(-1)]
        // = [-2, -2, -2]
        assert_eq!(result.get(0).unwrap(), -2.0);
        assert_eq!(result.get(1).unwrap(), -2.0);
        assert_eq!(result.get(2).unwrap(), -2.0);
    }
} 