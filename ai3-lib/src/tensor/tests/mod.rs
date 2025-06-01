#[cfg(test)]
mod tests {
    use super::super::{Tensor, TensorShape, TensorData};

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
        assert_eq!(tensor.shape.total_elements(), 4);
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