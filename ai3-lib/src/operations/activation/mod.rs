use serde::{Deserialize, Serialize};
use crate::tensor::{Tensor, TensorShape};
use crate::operations::TensorOp;
use tribechain_core::{TribeResult, TribeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationType {
    ReLU,
    Sigmoid,
    Tanh,
    LeakyReLU(f32),
    Softmax,
}

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