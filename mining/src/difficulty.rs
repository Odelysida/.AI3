use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::VecDeque;
use tribechain_core::{TribeResult, TribeError};

/// Difficulty adjustment algorithm
#[derive(Debug, Clone)]
pub struct DifficultyAdjuster {
    pub target_block_time: u64, // seconds
    pub adjustment_interval: u64, // blocks
    pub max_adjustment_factor: f64,
    pub min_difficulty: u32,
    pub max_difficulty: u32,
    pub current_difficulty: u32,
    pub block_times: VecDeque<BlockTimeRecord>,
    pub algorithm: DifficultyAlgorithm,
}

/// Different difficulty adjustment algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyAlgorithm {
    Bitcoin, // Adjust every 2016 blocks
    Ethereum, // Adjust every block
    AI3Adaptive, // AI3-specific adaptive algorithm
    Custom(CustomDifficultyParams),
}

/// Custom difficulty parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDifficultyParams {
    pub window_size: u64,
    pub target_time: u64,
    pub adjustment_factor: f64,
}

/// Block time record for difficulty calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTimeRecord {
    pub block_height: u64,
    pub timestamp: DateTime<Utc>,
    pub difficulty: u32,
    pub hash_rate: f64,
}

/// Difficulty adjustment result
#[derive(Debug, Clone)]
pub struct DifficultyAdjustment {
    pub old_difficulty: u32,
    pub new_difficulty: u32,
    pub adjustment_factor: f64,
    pub reason: String,
    pub effective_block_height: u64,
}

impl DifficultyAdjuster {
    pub fn new(algorithm: DifficultyAlgorithm) -> Self {
        let (target_time, interval) = match &algorithm {
            DifficultyAlgorithm::Bitcoin => (600, 2016), // 10 minutes, 2016 blocks
            DifficultyAlgorithm::Ethereum => (15, 1), // 15 seconds, every block
            DifficultyAlgorithm::AI3Adaptive => (30, 10), // 30 seconds, every 10 blocks
            DifficultyAlgorithm::Custom(params) => (params.target_time, params.window_size),
        };

        Self {
            target_block_time: target_time,
            adjustment_interval: interval,
            max_adjustment_factor: 4.0,
            min_difficulty: 1,
            max_difficulty: u32::MAX,
            current_difficulty: 4,
            block_times: VecDeque::new(),
            algorithm,
        }
    }

    pub fn add_block_time(&mut self, block_height: u64, timestamp: DateTime<Utc>, hash_rate: f64) {
        let record = BlockTimeRecord {
            block_height,
            timestamp,
            difficulty: self.current_difficulty,
            hash_rate,
        };

        self.block_times.push_back(record);

        // Keep only the necessary records based on algorithm
        let max_records = match &self.algorithm {
            DifficultyAlgorithm::Bitcoin => 2016,
            DifficultyAlgorithm::Ethereum => 2048, // Keep more for better calculation
            DifficultyAlgorithm::AI3Adaptive => 100,
            DifficultyAlgorithm::Custom(params) => params.window_size * 2,
        };

        while self.block_times.len() > max_records as usize {
            self.block_times.pop_front();
        }
    }

    pub fn should_adjust(&self, block_height: u64) -> bool {
        match &self.algorithm {
            DifficultyAlgorithm::Bitcoin => block_height % self.adjustment_interval == 0,
            DifficultyAlgorithm::Ethereum => true, // Adjust every block
            DifficultyAlgorithm::AI3Adaptive => block_height % self.adjustment_interval == 0,
            DifficultyAlgorithm::Custom(_) => block_height % self.adjustment_interval == 0,
        }
    }

    pub fn calculate_adjustment(&self, block_height: u64) -> TribeResult<Option<DifficultyAdjustment>> {
        if !self.should_adjust(block_height) {
            return Ok(None);
        }

        if self.block_times.len() < 2 {
            return Ok(None);
        }

        let adjustment = match &self.algorithm {
            DifficultyAlgorithm::Bitcoin => self.bitcoin_adjustment(block_height)?,
            DifficultyAlgorithm::Ethereum => self.ethereum_adjustment(block_height)?,
            DifficultyAlgorithm::AI3Adaptive => self.ai3_adaptive_adjustment(block_height)?,
            DifficultyAlgorithm::Custom(params) => self.custom_adjustment(block_height, params)?,
        };

        Ok(Some(adjustment))
    }

    fn bitcoin_adjustment(&self, block_height: u64) -> TribeResult<DifficultyAdjustment> {
        if self.block_times.len() < self.adjustment_interval as usize {
            return Err(TribeError::InvalidOperation("Insufficient block data".to_string()));
        }

        let recent_blocks: Vec<_> = self.block_times
            .iter()
            .rev()
            .take(self.adjustment_interval as usize)
            .collect();

        let time_taken = recent_blocks.first().unwrap().timestamp
            .signed_duration_since(recent_blocks.last().unwrap().timestamp)
            .num_seconds() as u64;

        let expected_time = self.target_block_time * self.adjustment_interval;
        let adjustment_factor = expected_time as f64 / time_taken as f64;

        // Clamp adjustment factor
        let clamped_factor = adjustment_factor.max(1.0 / self.max_adjustment_factor)
            .min(self.max_adjustment_factor);

        let new_difficulty = ((self.current_difficulty as f64 * clamped_factor) as u32)
            .max(self.min_difficulty)
            .min(self.max_difficulty);

        Ok(DifficultyAdjustment {
            old_difficulty: self.current_difficulty,
            new_difficulty,
            adjustment_factor: clamped_factor,
            reason: format!("Bitcoin algorithm: {} blocks took {}s, expected {}s", 
                self.adjustment_interval, time_taken, expected_time),
            effective_block_height: block_height,
        })
    }

    fn ethereum_adjustment(&self, block_height: u64) -> TribeResult<DifficultyAdjustment> {
        if self.block_times.len() < 2 {
            return Err(TribeError::InvalidOperation("Insufficient block data".to_string()));
        }

        let latest = self.block_times.back().unwrap();
        let previous = &self.block_times[self.block_times.len() - 2];

        let block_time = latest.timestamp
            .signed_duration_since(previous.timestamp)
            .num_seconds() as u64;

        let adjustment_factor = if block_time < self.target_block_time {
            1.0 + (self.target_block_time - block_time) as f64 / self.target_block_time as f64 * 0.1
        } else {
            1.0 - (block_time - self.target_block_time) as f64 / self.target_block_time as f64 * 0.1
        };

        let clamped_factor = adjustment_factor.max(0.9).min(1.1);
        let new_difficulty = ((self.current_difficulty as f64 * clamped_factor) as u32)
            .max(self.min_difficulty)
            .min(self.max_difficulty);

        Ok(DifficultyAdjustment {
            old_difficulty: self.current_difficulty,
            new_difficulty,
            adjustment_factor: clamped_factor,
            reason: format!("Ethereum algorithm: block time {}s, target {}s", 
                block_time, self.target_block_time),
            effective_block_height: block_height,
        })
    }

    fn ai3_adaptive_adjustment(&self, block_height: u64) -> TribeResult<DifficultyAdjustment> {
        if self.block_times.len() < self.adjustment_interval as usize {
            return Err(TribeError::InvalidOperation("Insufficient block data".to_string()));
        }

        let recent_blocks: Vec<_> = self.block_times
            .iter()
            .rev()
            .take(self.adjustment_interval as usize)
            .collect();

        // Calculate average block time
        let mut total_time = 0u64;
        for i in 0..recent_blocks.len() - 1 {
            let time_diff = recent_blocks[i].timestamp
                .signed_duration_since(recent_blocks[i + 1].timestamp)
                .num_seconds() as u64;
            total_time += time_diff;
        }

        let avg_block_time = total_time / (recent_blocks.len() - 1) as u64;

        // Calculate hash rate trend
        let avg_hash_rate: f64 = recent_blocks.iter().map(|b| b.hash_rate).sum::<f64>() 
            / recent_blocks.len() as f64;

        // AI3-specific adjustment considering tensor computation
        let base_adjustment = self.target_block_time as f64 / avg_block_time as f64;
        
        // Factor in hash rate changes (AI3 miners may have varying capabilities)
        let hash_rate_factor = if avg_hash_rate > 0.0 {
            (avg_hash_rate / 1000.0).min(2.0).max(0.5) // Normalize and clamp
        } else {
            1.0
        };

        let adjustment_factor = base_adjustment * hash_rate_factor;
        let clamped_factor = adjustment_factor.max(0.5).min(2.0); // More conservative for AI3

        let new_difficulty = ((self.current_difficulty as f64 * clamped_factor) as u32)
            .max(self.min_difficulty)
            .min(self.max_difficulty);

        Ok(DifficultyAdjustment {
            old_difficulty: self.current_difficulty,
            new_difficulty,
            adjustment_factor: clamped_factor,
            reason: format!("AI3 adaptive: avg time {}s, target {}s, hash rate {:.2}", 
                avg_block_time, self.target_block_time, avg_hash_rate),
            effective_block_height: block_height,
        })
    }

    fn custom_adjustment(&self, block_height: u64, params: &CustomDifficultyParams) -> TribeResult<DifficultyAdjustment> {
        if self.block_times.len() < params.window_size as usize {
            return Err(TribeError::InvalidOperation("Insufficient block data".to_string()));
        }

        let recent_blocks: Vec<_> = self.block_times
            .iter()
            .rev()
            .take(params.window_size as usize)
            .collect();

        let mut total_time = 0u64;
        for i in 0..recent_blocks.len() - 1 {
            let time_diff = recent_blocks[i].timestamp
                .signed_duration_since(recent_blocks[i + 1].timestamp)
                .num_seconds() as u64;
            total_time += time_diff;
        }

        let avg_block_time = total_time / (recent_blocks.len() - 1) as u64;
        let adjustment_factor = (params.target_time as f64 / avg_block_time as f64) * params.adjustment_factor;

        let clamped_factor = adjustment_factor.max(1.0 / self.max_adjustment_factor)
            .min(self.max_adjustment_factor);

        let new_difficulty = ((self.current_difficulty as f64 * clamped_factor) as u32)
            .max(self.min_difficulty)
            .min(self.max_difficulty);

        Ok(DifficultyAdjustment {
            old_difficulty: self.current_difficulty,
            new_difficulty,
            adjustment_factor: clamped_factor,
            reason: format!("Custom algorithm: avg time {}s, target {}s", 
                avg_block_time, params.target_time),
            effective_block_height: block_height,
        })
    }

    pub fn apply_adjustment(&mut self, adjustment: DifficultyAdjustment) {
        self.current_difficulty = adjustment.new_difficulty;
    }

    pub fn get_current_difficulty(&self) -> u32 {
        self.current_difficulty
    }

    pub fn get_target_block_time(&self) -> u64 {
        self.target_block_time
    }

    pub fn get_average_block_time(&self, window: usize) -> Option<f64> {
        if self.block_times.len() < 2 {
            return None;
        }

        let blocks_to_check = window.min(self.block_times.len() - 1);
        let recent_blocks: Vec<_> = self.block_times
            .iter()
            .rev()
            .take(blocks_to_check + 1)
            .collect();

        let mut total_time = 0u64;
        for i in 0..recent_blocks.len() - 1 {
            let time_diff = recent_blocks[i].timestamp
                .signed_duration_since(recent_blocks[i + 1].timestamp)
                .num_seconds() as u64;
            total_time += time_diff;
        }

        Some(total_time as f64 / blocks_to_check as f64)
    }

    pub fn estimate_next_difficulty(&self, block_height: u64) -> TribeResult<u32> {
        if let Some(adjustment) = self.calculate_adjustment(block_height)? {
            Ok(adjustment.new_difficulty)
        } else {
            Ok(self.current_difficulty)
        }
    }
}

impl Default for DifficultyAdjuster {
    fn default() -> Self {
        Self::new(DifficultyAlgorithm::AI3Adaptive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_adjuster_creation() {
        let adjuster = DifficultyAdjuster::new(DifficultyAlgorithm::Bitcoin);
        assert_eq!(adjuster.target_block_time, 600);
        assert_eq!(adjuster.adjustment_interval, 2016);
        assert_eq!(adjuster.current_difficulty, 4);
    }

    #[test]
    fn test_should_adjust() {
        let adjuster = DifficultyAdjuster::new(DifficultyAlgorithm::Bitcoin);
        assert!(adjuster.should_adjust(2016));
        assert!(!adjuster.should_adjust(2015));
        assert!(adjuster.should_adjust(4032));
    }

    #[test]
    fn test_add_block_time() {
        let mut adjuster = DifficultyAdjuster::new(DifficultyAlgorithm::AI3Adaptive);
        let now = Utc::now();
        
        adjuster.add_block_time(1, now, 1000.0);
        assert_eq!(adjuster.block_times.len(), 1);
        
        adjuster.add_block_time(2, now + Duration::seconds(30), 1100.0);
        assert_eq!(adjuster.block_times.len(), 2);
    }

    #[test]
    fn test_average_block_time() {
        let mut adjuster = DifficultyAdjuster::new(DifficultyAlgorithm::AI3Adaptive);
        let now = Utc::now();
        
        adjuster.add_block_time(1, now, 1000.0);
        adjuster.add_block_time(2, now + Duration::seconds(30), 1000.0);
        adjuster.add_block_time(3, now + Duration::seconds(60), 1000.0);
        
        let avg = adjuster.get_average_block_time(2).unwrap();
        assert_eq!(avg, 30.0);
    }

    #[tokio::test]
    async fn test_ai3_adaptive_adjustment() {
        let mut adjuster = DifficultyAdjuster::new(DifficultyAlgorithm::AI3Adaptive);
        let now = Utc::now();
        
        // Add blocks with consistent 30-second intervals (target time)
        for i in 0..10 {
            adjuster.add_block_time(i + 1, now + Duration::seconds(i * 30), 1000.0);
        }
        
        let adjustment = adjuster.calculate_adjustment(10).unwrap();
        assert!(adjustment.is_some());
        
        let adj = adjustment.unwrap();
        // Should maintain difficulty since we're hitting target time
        assert!((adj.adjustment_factor - 1.0).abs() < 0.1);
    }
} 