use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

/// Staking contract implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    pub id: String,
    pub token_id: String,
    pub validator: String,
    pub min_stake: u64,
    pub max_stake: Option<u64>,
    pub reward_rate: f64, // Annual percentage rate
    pub total_staked: u64,
    pub total_rewards_distributed: u64,
    pub stakes: HashMap<String, StakeInfo>,
    pub validators: HashMap<String, ValidatorInfo>,
    pub reward_pool: u64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_reward_calculation: DateTime<Utc>,
    pub lock_period: Duration,
    pub early_withdrawal_penalty: f64,
}

/// Individual stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub staker: String,
    pub amount: u64,
    pub delegated_to: String,
    pub staked_at: DateTime<Utc>,
    pub lock_until: DateTime<Utc>,
    pub accumulated_rewards: u64,
    pub last_reward_claim: DateTime<Utc>,
    pub is_active: bool,
    pub auto_compound: bool,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub name: String,
    pub description: String,
    pub commission_rate: f64, // Percentage taken by validator
    pub total_delegated: u64,
    pub self_stake: u64,
    pub is_active: bool,
    pub is_jailed: bool,
    pub jail_until: Option<DateTime<Utc>>,
    pub uptime: f64,
    pub slash_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Staking rewards information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingRewards {
    pub staker: String,
    pub validator: String,
    pub amount: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub calculated_at: DateTime<Utc>,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
}

/// Delegation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationOperation {
    pub delegator: String,
    pub validator: String,
    pub amount: u64,
    pub operation_type: DelegationType,
    pub timestamp: DateTime<Utc>,
}

/// Delegation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationType {
    Delegate,
    Undelegate,
    Redelegate { from_validator: String },
    ClaimRewards,
}

/// Slashing event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub validator: String,
    pub reason: SlashingReason,
    pub amount: u64,
    pub percentage: f64,
    pub timestamp: DateTime<Utc>,
    pub block_height: u64,
}

/// Slashing reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingReason {
    DoubleSign,
    Downtime,
    InvalidBehavior,
    MissedBlocks(u32),
}

impl StakingContract {
    /// Create a new staking contract
    pub fn new(
        token_id: String,
        validator: String,
        min_stake: u64,
        reward_rate: f64,
    ) -> TribeResult<Self> {
        if token_id.is_empty() || validator.is_empty() {
            return Err(TribeError::InvalidOperation("Token ID and validator cannot be empty".to_string()));
        }

        if min_stake == 0 {
            return Err(TribeError::InvalidOperation("Minimum stake must be greater than 0".to_string()));
        }

        if reward_rate < 0.0 || reward_rate > 1.0 {
            return Err(TribeError::InvalidOperation("Reward rate must be between 0 and 1".to_string()));
        }

        let contract_id = Self::generate_contract_id(&token_id, &validator);
        let mut validators = HashMap::new();
        
        // Add the initial validator
        validators.insert(validator.clone(), ValidatorInfo {
            address: validator.clone(),
            name: "Primary Validator".to_string(),
            description: "Initial validator for this staking contract".to_string(),
            commission_rate: 0.05, // 5% default commission
            total_delegated: 0,
            self_stake: 0,
            is_active: true,
            is_jailed: false,
            jail_until: None,
            uptime: 1.0,
            slash_count: 0,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        });

        Ok(Self {
            id: contract_id,
            token_id,
            validator,
            min_stake,
            max_stake: None,
            reward_rate,
            total_staked: 0,
            total_rewards_distributed: 0,
            stakes: HashMap::new(),
            validators,
            reward_pool: 0,
            is_active: true,
            created_at: Utc::now(),
            last_reward_calculation: Utc::now(),
            lock_period: Duration::days(30), // Default 30-day lock
            early_withdrawal_penalty: 0.1, // 10% penalty
        })
    }

    /// Stake tokens
    pub fn stake(
        &mut self,
        staker: String,
        amount: u64,
        duration: u64, // Duration in days
    ) -> TribeResult<()> {
        if !self.is_active {
            return Err(TribeError::InvalidOperation("Staking contract is not active".to_string()));
        }

        if amount < self.min_stake {
            return Err(TribeError::InvalidOperation("Amount below minimum stake".to_string()));
        }

        if let Some(max_stake) = self.max_stake {
            if amount > max_stake {
                return Err(TribeError::InvalidOperation("Amount exceeds maximum stake".to_string()));
            }
        }

        let lock_until = Utc::now() + Duration::days(duration as i64);
        
        // Check if staker already has a stake
        if let Some(existing_stake) = self.stakes.get_mut(&staker) {
            // Add to existing stake
            existing_stake.amount += amount;
            existing_stake.lock_until = lock_until.max(existing_stake.lock_until);
        } else {
            // Create new stake
            let stake_info = StakeInfo {
                staker: staker.clone(),
                amount,
                delegated_to: self.validator.clone(),
                staked_at: Utc::now(),
                lock_until,
                accumulated_rewards: 0,
                last_reward_claim: Utc::now(),
                is_active: true,
                auto_compound: false,
            };
            self.stakes.insert(staker, stake_info);
        }

        // Update validator delegation
        if let Some(validator) = self.validators.get_mut(&self.validator) {
            validator.total_delegated += amount;
        }

        self.total_staked += amount;
        Ok(())
    }

    /// Unstake tokens
    pub fn unstake(&mut self, staker: String, amount: u64) -> TribeResult<u64> {
        let stake = self.stakes.get_mut(&staker)
            .ok_or_else(|| TribeError::InvalidOperation("No stake found for staker".to_string()))?;

        if !stake.is_active {
            return Err(TribeError::InvalidOperation("Stake is not active".to_string()));
        }

        if amount > stake.amount {
            return Err(TribeError::InvalidOperation("Insufficient staked amount".to_string()));
        }

        let now = Utc::now();
        let mut penalty = 0u64;

        // Check if lock period has passed
        if now < stake.lock_until {
            // Apply early withdrawal penalty
            penalty = (amount as f64 * self.early_withdrawal_penalty) as u64;
        }

        // Calculate and claim any pending rewards
        self.calculate_rewards(&staker)?;

        // Update stake
        stake.amount -= amount;
        if stake.amount == 0 {
            stake.is_active = false;
        }

        // Update validator delegation
        if let Some(validator) = self.validators.get_mut(&stake.delegated_to) {
            validator.total_delegated = validator.total_delegated.saturating_sub(amount);
        }

        self.total_staked = self.total_staked.saturating_sub(amount);

        // Return amount minus penalty
        Ok(amount - penalty)
    }

    /// Delegate to a different validator
    pub fn delegate(
        &mut self,
        staker: String,
        validator: String,
        amount: u64,
    ) -> TribeResult<()> {
        if !self.validators.contains_key(&validator) {
            return Err(TribeError::InvalidOperation("Validator not found".to_string()));
        }

        let validator_info = self.validators.get(&validator).unwrap();
        if !validator_info.is_active || validator_info.is_jailed {
            return Err(TribeError::InvalidOperation("Validator is not active or jailed".to_string()));
        }

        if amount < self.min_stake {
            return Err(TribeError::InvalidOperation("Amount below minimum stake".to_string()));
        }

        // Create or update stake
        if let Some(stake) = self.stakes.get_mut(&staker) {
            // Redelegate from current validator
            if let Some(current_validator) = self.validators.get_mut(&stake.delegated_to) {
                current_validator.total_delegated = current_validator.total_delegated.saturating_sub(stake.amount);
            }

            stake.delegated_to = validator.clone();
            stake.amount = amount;
        } else {
            // New delegation
            let stake_info = StakeInfo {
                staker: staker.clone(),
                amount,
                delegated_to: validator.clone(),
                staked_at: Utc::now(),
                lock_until: Utc::now() + self.lock_period,
                accumulated_rewards: 0,
                last_reward_claim: Utc::now(),
                is_active: true,
                auto_compound: false,
            };
            self.stakes.insert(staker, stake_info);
        }

        // Update new validator delegation
        if let Some(new_validator) = self.validators.get_mut(&validator) {
            new_validator.total_delegated += amount;
        }

        Ok(())
    }

    /// Calculate rewards for a staker
    pub fn calculate_rewards(&mut self, staker: &str) -> TribeResult<u64> {
        let stake = self.stakes.get_mut(staker)
            .ok_or_else(|| TribeError::InvalidOperation("No stake found for staker".to_string()))?;

        if !stake.is_active {
            return Ok(0);
        }

        let now = Utc::now();
        let time_diff = now.signed_duration_since(stake.last_reward_claim);
        let days = time_diff.num_days() as f64;

        if days <= 0.0 {
            return Ok(0);
        }

        // Get validator commission rate
        let commission_rate = self.validators.get(&stake.delegated_to)
            .map(|v| v.commission_rate)
            .unwrap_or(0.0);

        // Calculate base rewards (annual rate / 365 * days * amount)
        let base_reward = (self.reward_rate / 365.0 * days * stake.amount as f64) as u64;
        
        // Apply validator commission
        let commission = (base_reward as f64 * commission_rate) as u64;
        let net_reward = base_reward - commission;

        stake.accumulated_rewards += net_reward;
        stake.last_reward_claim = now;

        Ok(net_reward)
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, staker: String) -> TribeResult<u64> {
        // Calculate latest rewards
        let new_rewards = self.calculate_rewards(&staker)?;
        
        let stake = self.stakes.get_mut(&staker)
            .ok_or_else(|| TribeError::InvalidOperation("No stake found for staker".to_string()))?;

        let total_rewards = stake.accumulated_rewards;
        
        if total_rewards == 0 {
            return Err(TribeError::InvalidOperation("No rewards to claim".to_string()));
        }

        // Reset accumulated rewards
        stake.accumulated_rewards = 0;
        self.total_rewards_distributed += total_rewards;

        Ok(total_rewards)
    }

    /// Add a new validator
    pub fn add_validator(
        &mut self,
        validator_address: String,
        name: String,
        description: String,
        commission_rate: f64,
        self_stake: u64,
    ) -> TribeResult<()> {
        if self.validators.contains_key(&validator_address) {
            return Err(TribeError::InvalidOperation("Validator already exists".to_string()));
        }

        if commission_rate < 0.0 || commission_rate > 1.0 {
            return Err(TribeError::InvalidOperation("Commission rate must be between 0 and 1".to_string()));
        }

        let validator_info = ValidatorInfo {
            address: validator_address.clone(),
            name,
            description,
            commission_rate,
            total_delegated: 0,
            self_stake,
            is_active: true,
            is_jailed: false,
            jail_until: None,
            uptime: 1.0,
            slash_count: 0,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        self.validators.insert(validator_address, validator_info);
        Ok(())
    }

    /// Slash a validator
    pub fn slash_validator(
        &mut self,
        validator: String,
        reason: SlashingReason,
        percentage: f64,
    ) -> TribeResult<u64> {
        let validator_info = self.validators.get_mut(&validator)
            .ok_or_else(|| TribeError::InvalidOperation("Validator not found".to_string()))?;

        if percentage < 0.0 || percentage > 1.0 {
            return Err(TribeError::InvalidOperation("Slash percentage must be between 0 and 1".to_string()));
        }

        let slash_amount = (validator_info.total_delegated as f64 * percentage) as u64;
        
        // Apply slashing to all delegators
        for stake in self.stakes.values_mut() {
            if stake.delegated_to == validator && stake.is_active {
                let stake_slash = (stake.amount as f64 * percentage) as u64;
                stake.amount = stake.amount.saturating_sub(stake_slash);
                
                if stake.amount == 0 {
                    stake.is_active = false;
                }
            }
        }

        // Update validator info
        validator_info.total_delegated = validator_info.total_delegated.saturating_sub(slash_amount);
        validator_info.slash_count += 1;
        validator_info.is_jailed = true;
        validator_info.jail_until = Some(Utc::now() + Duration::days(7)); // 7-day jail

        self.total_staked = self.total_staked.saturating_sub(slash_amount);

        Ok(slash_amount)
    }

    /// Unjail a validator
    pub fn unjail_validator(&mut self, validator: String) -> TribeResult<()> {
        let validator_info = self.validators.get_mut(&validator)
            .ok_or_else(|| TribeError::InvalidOperation("Validator not found".to_string()))?;

        if !validator_info.is_jailed {
            return Err(TribeError::InvalidOperation("Validator is not jailed".to_string()));
        }

        if let Some(jail_until) = validator_info.jail_until {
            if Utc::now() < jail_until {
                return Err(TribeError::InvalidOperation("Jail period has not ended".to_string()));
            }
        }

        validator_info.is_jailed = false;
        validator_info.jail_until = None;
        validator_info.last_activity = Utc::now();

        Ok(())
    }

    /// Get staking statistics
    pub fn get_stats(&self) -> StakingStats {
        let active_stakes = self.stakes.values().filter(|s| s.is_active).count();
        let active_validators = self.validators.values().filter(|v| v.is_active && !v.is_jailed).count();
        
        let avg_stake = if active_stakes > 0 {
            self.total_staked / active_stakes as u64
        } else {
            0
        };

        StakingStats {
            total_staked: self.total_staked,
            total_stakers: active_stakes,
            total_validators: active_validators,
            total_rewards_distributed: self.total_rewards_distributed,
            average_stake: avg_stake,
            current_apy: self.reward_rate,
        }
    }

    /// Generate contract ID
    fn generate_contract_id(token_id: &str, validator: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(token_id.as_bytes());
        hasher.update(validator.as_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..16])
    }

    /// Get stake info for a staker
    pub fn get_stake_info(&self, staker: &str) -> Option<&StakeInfo> {
        self.stakes.get(staker)
    }

    /// Get validator info
    pub fn get_validator_info(&self, validator: &str) -> Option<&ValidatorInfo> {
        self.validators.get(validator)
    }

    /// Update validator uptime
    pub fn update_validator_uptime(&mut self, validator: String, uptime: f64) -> TribeResult<()> {
        let validator_info = self.validators.get_mut(&validator)
            .ok_or_else(|| TribeError::InvalidOperation("Validator not found".to_string()))?;

        validator_info.uptime = uptime.clamp(0.0, 1.0);
        validator_info.last_activity = Utc::now();

        // Auto-jail validator if uptime is too low
        if uptime < 0.5 && !validator_info.is_jailed {
            validator_info.is_jailed = true;
            validator_info.jail_until = Some(Utc::now() + Duration::days(1));
        }

        Ok(())
    }

    /// Set auto-compound for a stake
    pub fn set_auto_compound(&mut self, staker: String, auto_compound: bool) -> TribeResult<()> {
        let stake = self.stakes.get_mut(&staker)
            .ok_or_else(|| TribeError::InvalidOperation("No stake found for staker".to_string()))?;

        stake.auto_compound = auto_compound;
        Ok(())
    }

    /// Compound rewards (add to stake)
    pub fn compound_rewards(&mut self, staker: String) -> TribeResult<u64> {
        let rewards = self.calculate_rewards(&staker)?;
        
        if rewards == 0 {
            return Ok(0);
        }

        let stake = self.stakes.get_mut(&staker).unwrap();
        let total_rewards = stake.accumulated_rewards;
        
        // Add rewards to stake amount
        stake.amount += total_rewards;
        stake.accumulated_rewards = 0;

        // Update validator delegation
        if let Some(validator) = self.validators.get_mut(&stake.delegated_to) {
            validator.total_delegated += total_rewards;
        }

        self.total_staked += total_rewards;

        Ok(total_rewards)
    }
}

/// Staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_staked: u64,
    pub total_stakers: usize,
    pub total_validators: usize,
    pub total_rewards_distributed: u64,
    pub average_stake: u64,
    pub current_apy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staking_contract_creation() {
        let contract = StakingContract::new(
            "token123".to_string(),
            "validator1".to_string(),
            1000,
            0.1, // 10% APR
        ).unwrap();

        assert_eq!(contract.token_id, "token123");
        assert_eq!(contract.validator, "validator1");
        assert_eq!(contract.min_stake, 1000);
        assert_eq!(contract.reward_rate, 0.1);
        assert!(contract.is_active);
    }

    #[test]
    fn test_staking() {
        let mut contract = StakingContract::new(
            "token123".to_string(),
            "validator1".to_string(),
            1000,
            0.1,
        ).unwrap();

        // Stake tokens
        assert!(contract.stake("staker1".to_string(), 5000, 30).is_ok());
        assert_eq!(contract.total_staked, 5000);
        
        let stake_info = contract.get_stake_info("staker1").unwrap();
        assert_eq!(stake_info.amount, 5000);
        assert_eq!(stake_info.delegated_to, "validator1");
    }

    #[test]
    fn test_unstaking() {
        let mut contract = StakingContract::new(
            "token123".to_string(),
            "validator1".to_string(),
            1000,
            0.1,
        ).unwrap();

        // Stake and then unstake
        contract.stake("staker1".to_string(), 5000, 30).unwrap();
        
        // Immediate unstaking should apply penalty
        let returned = contract.unstake("staker1".to_string(), 2000).unwrap();
        assert!(returned < 2000); // Should be less due to penalty
        
        let stake_info = contract.get_stake_info("staker1").unwrap();
        assert_eq!(stake_info.amount, 3000);
    }

    #[test]
    fn test_validator_management() {
        let mut contract = StakingContract::new(
            "token123".to_string(),
            "validator1".to_string(),
            1000,
            0.1,
        ).unwrap();

        // Add new validator
        assert!(contract.add_validator(
            "validator2".to_string(),
            "Validator 2".to_string(),
            "Second validator".to_string(),
            0.05,
            10000,
        ).is_ok());

        assert!(contract.validators.contains_key("validator2"));
        
        // Slash validator
        contract.stake("staker1".to_string(), 5000, 30).unwrap();
        contract.delegate("staker1".to_string(), "validator2".to_string(), 5000).unwrap();
        
        let slashed = contract.slash_validator(
            "validator2".to_string(),
            SlashingReason::DoubleSign,
            0.1, // 10% slash
        ).unwrap();
        
        assert_eq!(slashed, 500); // 10% of 5000
        
        let validator_info = contract.get_validator_info("validator2").unwrap();
        assert!(validator_info.is_jailed);
    }

    #[test]
    fn test_rewards_calculation() {
        let mut contract = StakingContract::new(
            "token123".to_string(),
            "validator1".to_string(),
            1000,
            0.365, // 36.5% APR for easy calculation
        ).unwrap();

        contract.stake("staker1".to_string(), 10000, 30).unwrap();
        
        // Manually set last reward claim to 1 day ago for testing
        let stake = contract.stakes.get_mut("staker1").unwrap();
        stake.last_reward_claim = Utc::now() - Duration::days(1);
        
        let rewards = contract.calculate_rewards("staker1").unwrap();
        // Should be approximately 10 tokens (10000 * 0.365 / 365)
        assert!(rewards >= 9 && rewards <= 11);
    }
} 