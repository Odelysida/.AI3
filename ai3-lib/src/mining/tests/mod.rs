#[cfg(test)]
mod tests {
    use super::super::{tasks::MiningTask, miners::AI3Miner, distributors::TaskDistributor};
    use crate::tensor::{Tensor, TensorShape};

    #[test]
    fn test_mining_task_creation() {
        let input_tensor = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input_tensor],
            4,
            100,
            60,
            "test_requester".to_string(),
        );

        assert_eq!(task.operation_type, "relu");
        assert_eq!(task.difficulty, 4);
        assert_eq!(task.reward, 100);
        assert_eq!(task.requester, "test_requester");
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_miner_capabilities() {
        let miner = AI3Miner::new("test_miner".to_string(), "127.0.0.1:8080".to_string(), false);
        
        assert_eq!(miner.id, "test_miner");
        assert_eq!(miner.address, "127.0.0.1:8080");
        assert!(!miner.capabilities.is_esp_device);
        assert!(miner.capabilities.max_tensor_size > 1024);
    }

    #[test]
    fn test_task_assignment() {
        let mut miner = AI3Miner::new("test_miner".to_string(), "127.0.0.1:8080".to_string(), false);
        let input_tensor = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input_tensor],
            4,
            100,
            60,
            "test_requester".to_string(),
        );

        assert!(miner.can_handle_task(&task));
        assert!(miner.assign_task(task).is_ok());
        assert!(miner.current_task.is_some());
    }

    #[test]
    fn test_difficulty_check() {
        let input_tensor = Tensor::vector(vec![1.0, 2.0, 3.0]);
        let task = MiningTask::new(
            "relu".to_string(),
            vec![input_tensor],
            4,
            100,
            60,
            "test_requester".to_string(),
        );

        // Hash with 4 leading zeros should meet difficulty 4
        let valid_hash = "0000abcdef123456789";
        assert!(task.meets_difficulty(valid_hash));

        // Hash with 3 leading zeros should not meet difficulty 4
        let invalid_hash = "000abcdef123456789";
        assert!(!task.meets_difficulty(invalid_hash));
    }
} 