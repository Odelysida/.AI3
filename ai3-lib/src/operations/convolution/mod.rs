use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape};
use crate::operations::TensorOp;
use tribechain_core::{TribeResult, TribeError};

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

        Err(TribeError::InvalidOperation("Unsupported tensor dimensions for convolution".to_string()))
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
            return Err(TribeError::InvalidOperation("Convolution only supports 1D and 2D tensors".to_string()));
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