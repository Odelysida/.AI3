#[cfg(test)]
mod tests {
    use super::super::{
        TensorOp,
        matrix::MatrixMultiply,
        activation::ActivationFunction,
        vector::VectorOp,
        convolution::Convolution,
    };
    use crate::tensor::{Tensor, TensorShape};

    #[test]
    fn test_matrix_multiply() {
        let a = Tensor::matrix(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
        let b = Tensor::matrix(vec![5.0, 6.0, 7.0, 8.0], 2, 2).unwrap();
        
        let matmul = MatrixMultiply::new();
        let result = matmul.execute(&[a, b]).unwrap();
        
        assert_eq!(result.shape.dimensions, vec![2, 2]);
        let result_data = result.data.as_f32_vec().unwrap();
        assert_eq!(result_data, vec![19.0, 22.0, 43.0, 50.0]);
    }

    #[test]
    fn test_relu_activation() {
        let input = Tensor::vector(vec![-1.0, 0.0, 1.0, 2.0]);
        let relu = ActivationFunction::relu();
        let result = relu.execute(&[input]).unwrap();
        
        let result_data = result.data.as_f32_vec().unwrap();
        assert_eq!(result_data, vec![0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_vector_dot_product() {
        let a = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let b = Tensor::vector(vec![4.0, 5.0, 6.0]);
        
        let dot_op = VectorOp::dot_product();
        let result = dot_op.execute(&[a, b]).unwrap();
        
        let result_data = result.data.as_f32_vec().unwrap();
        assert_eq!(result_data, vec![32.0]); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_convolution_1d() {
        let input = Tensor::vector(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let kernel = Tensor::vector(vec![1.0, 0.0, -1.0]);
        
        let conv = Convolution::new(3);
        let result = conv.execute(&[input, kernel]).unwrap();
        
        let result_data = result.data.as_f32_vec().unwrap();
        assert_eq!(result_data.len(), 3); // Output size should be 3
    }
} 