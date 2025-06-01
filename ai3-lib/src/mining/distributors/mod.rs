use std::collections::HashMap;
use crate::mining::tasks::MiningTask;
use crate::mining::results::MiningResult;
use crate::mining::miners::AI3Miner;
use tribechain_core::{TribeResult, TribeError};

/// Task distributor for managing mining tasks
#[derive(Debug)]
pub struct TaskDistributor {
    pub pending_tasks: HashMap<String, MiningTask>,
    pub active_tasks: HashMap<String, (MiningTask, String)>, // task_id -> (task, miner_id)
    pub completed_tasks: HashMap<String, MiningResult>,
}

impl TaskDistributor {
    pub fn new() -> Self {
        Self {
            pending_tasks: HashMap::new(),
            active_tasks: HashMap::new(),
            completed_tasks: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: MiningTask) {
        self.pending_tasks.insert(task.id.clone(), task);
    }

    pub fn distribute(&mut self, task: MiningTask, miners: &[AI3Miner]) -> TribeResult<Vec<String>> {
        let mut assigned_miners = Vec::new();

        // Find suitable miners
        for miner in miners {
            if miner.can_handle_task(&task) {
                self.active_tasks.insert(task.id.clone(), (task.clone(), miner.id.clone()));
                assigned_miners.push(miner.id.clone());
                break; // For now, assign to first suitable miner
            }
        }

        if assigned_miners.is_empty() {
            // No suitable miners found, keep in pending
            self.pending_tasks.insert(task.id.clone(), task);
            return Err(TribeError::InvalidOperation("No suitable miners available".to_string()));
        }

        // Remove from pending if it was there
        self.pending_tasks.remove(&task.id);

        Ok(assigned_miners)
    }

    pub fn submit_result(&mut self, result: MiningResult) -> TribeResult<()> {
        // Validate that this task was actually assigned
        if let Some((task, _miner_id)) = self.active_tasks.remove(&result.task_id) {
            // Validate the result
            let mut validated_result = result;
            validated_result.validate(&task)?;
            
            self.completed_tasks.insert(task.id.clone(), validated_result);
            Ok(())
        } else {
            Err(TribeError::InvalidOperation("Task not found in active tasks".to_string()))
        }
    }

    pub fn get_pending_tasks(&self) -> Vec<&MiningTask> {
        self.pending_tasks.values().collect()
    }

    pub fn get_completed_results(&self) -> Vec<&MiningResult> {
        self.completed_tasks.values().collect()
    }

    pub fn cleanup_expired_tasks(&mut self) {
        self.pending_tasks.retain(|_, task| !task.is_expired());
        self.active_tasks.retain(|_, (task, _)| !task.is_expired());
    }
}

impl Default for TaskDistributor {
    fn default() -> Self {
        Self::new()
    }
} 