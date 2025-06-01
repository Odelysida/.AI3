pub mod vm;
pub mod contracts;
pub mod tokens;
pub mod staking;
pub mod liquidity;

// Re-export main types
pub use vm::{ContractVM, VMState, ExecutionResult, VMError};
pub use contracts::{Contract, ContractType, ContractCall, ContractDeployment};
pub use tokens::{TokenContract, TokenOperation, TokenInfo, TokenBalance};
pub use staking::{StakingContract, StakeInfo, ValidatorInfo, StakingRewards};
pub use liquidity::{LiquidityPool, PoolInfo, LiquidityPosition, SwapResult};

use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Smart contract execution environment
#[derive(Debug)]
pub struct ContractEngine {
    pub vm: ContractVM,
    pub deployed_contracts: HashMap<String, Contract>,
    pub token_contracts: HashMap<String, TokenContract>,
    pub staking_contracts: HashMap<String, StakingContract>,
    pub liquidity_pools: HashMap<String, LiquidityPool>,
}

impl ContractEngine {
    pub fn new() -> Self {
        Self {
            vm: ContractVM::new(),
            deployed_contracts: HashMap::new(),
            token_contracts: HashMap::new(),
            staking_contracts: HashMap::new(),
            liquidity_pools: HashMap::new(),
        }
    }

    /// Deploy a new contract
    pub fn deploy_contract(&mut self, deployment: ContractDeployment) -> TribeResult<String> {
        let contract_address = self.vm.deploy(deployment.clone())?;
        
        let contract = Contract::new(
            contract_address.clone(),
            deployment.contract_type,
            deployment.code,
            deployment.constructor_args,
            deployment.deployer,
        );

        self.deployed_contracts.insert(contract_address.clone(), contract);
        Ok(contract_address)
    }

    /// Call a contract method
    pub fn call_contract(&mut self, call: ContractCall) -> TribeResult<ExecutionResult> {
        if let Some(contract) = self.deployed_contracts.get(&call.contract_address) {
            self.vm.call(contract, call)
        } else {
            Err(TribeError::InvalidOperation("Contract not found".to_string()))
        }
    }

    /// Create a new token
    pub fn create_token(
        &mut self,
        name: String,
        symbol: String,
        total_supply: u64,
        decimals: u8,
        creator: String,
    ) -> TribeResult<String> {
        let token_contract = TokenContract::new(name, symbol, total_supply, decimals, creator)?;
        let token_id = token_contract.token_info.id.clone();
        
        self.token_contracts.insert(token_id.clone(), token_contract);
        Ok(token_id)
    }

    /// Transfer tokens
    pub fn transfer_token(
        &mut self,
        token_id: String,
        from: String,
        to: String,
        amount: u64,
    ) -> TribeResult<()> {
        if let Some(token_contract) = self.token_contracts.get_mut(&token_id) {
            token_contract.transfer(from, to, amount)
        } else {
            Err(TribeError::InvalidOperation("Token not found".to_string()))
        }
    }

    /// Create staking contract
    pub fn create_staking_contract(
        &mut self,
        token_id: String,
        validator: String,
        min_stake: u64,
        reward_rate: f64,
    ) -> TribeResult<String> {
        let staking_contract = StakingContract::new(token_id, validator, min_stake, reward_rate)?;
        let contract_id = staking_contract.id.clone();
        
        self.staking_contracts.insert(contract_id.clone(), staking_contract);
        Ok(contract_id)
    }

    /// Stake tokens
    pub fn stake_tokens(
        &mut self,
        staking_contract_id: String,
        staker: String,
        amount: u64,
        duration: u64,
    ) -> TribeResult<()> {
        if let Some(staking_contract) = self.staking_contracts.get_mut(&staking_contract_id) {
            staking_contract.stake(staker, amount, duration)
        } else {
            Err(TribeError::InvalidOperation("Staking contract not found".to_string()))
        }
    }

    /// Create liquidity pool
    pub fn create_liquidity_pool(
        &mut self,
        token_a: String,
        token_b: String,
        fee_rate: f64,
    ) -> TribeResult<String> {
        let pool = LiquidityPool::new(token_a, token_b, fee_rate)?;
        let pool_id = pool.id.clone();
        
        self.liquidity_pools.insert(pool_id.clone(), pool);
        Ok(pool_id)
    }

    /// Add liquidity to pool
    pub fn add_liquidity(
        &mut self,
        pool_id: String,
        provider: String,
        amount_a: u64,
        amount_b: u64,
    ) -> TribeResult<u64> {
        if let Some(pool) = self.liquidity_pools.get_mut(&pool_id) {
            pool.add_liquidity(provider, amount_a, amount_b)
        } else {
            Err(TribeError::InvalidOperation("Liquidity pool not found".to_string()))
        }
    }

    /// Swap tokens in pool
    pub fn swap_tokens(
        &mut self,
        pool_id: String,
        trader: String,
        token_in: String,
        amount_in: u64,
        min_amount_out: u64,
    ) -> TribeResult<SwapResult> {
        if let Some(pool) = self.liquidity_pools.get_mut(&pool_id) {
            pool.swap(trader, token_in, amount_in, min_amount_out)
        } else {
            Err(TribeError::InvalidOperation("Liquidity pool not found".to_string()))
        }
    }

    /// Get contract state
    pub fn get_contract_state(&self, contract_address: &str) -> Option<&Contract> {
        self.deployed_contracts.get(contract_address)
    }

    /// Get token info
    pub fn get_token_info(&self, token_id: &str) -> Option<&TokenInfo> {
        self.token_contracts.get(token_id).map(|c| &c.token_info)
    }

    /// Get token balance
    pub fn get_token_balance(&self, token_id: &str, address: &str) -> u64 {
        self.token_contracts
            .get(token_id)
            .and_then(|c| c.balances.get(address))
            .copied()
            .unwrap_or(0)
    }

    /// Get staking info
    pub fn get_staking_info(&self, contract_id: &str, staker: &str) -> Option<&StakeInfo> {
        self.staking_contracts
            .get(contract_id)
            .and_then(|c| c.stakes.get(staker))
    }

    /// Get liquidity pool info
    pub fn get_pool_info(&self, pool_id: &str) -> Option<&PoolInfo> {
        self.liquidity_pools.get(pool_id).map(|p| &p.info)
    }

    /// Process tensor computation contract
    pub fn process_tensor_computation(
        &mut self,
        contract_address: String,
        operation: String,
        input_data: Vec<f32>,
        requester: String,
        reward: u64,
    ) -> TribeResult<String> {
        // Create a tensor computation task
        let task_id = uuid::Uuid::new_v4().to_string();
        
        // In a real implementation, this would:
        // 1. Validate the contract exists and has sufficient funds
        // 2. Create a mining task for the AI3 engine
        // 3. Distribute the task to miners
        // 4. Handle result validation and reward distribution
        
        // For now, we'll simulate this process
        if let Some(_contract) = self.deployed_contracts.get(&contract_address) {
            // Simulate task creation and distribution
            Ok(task_id)
        } else {
            Err(TribeError::InvalidOperation("Tensor computation contract not found".to_string()))
        }
    }

    /// Calculate rewards for completed tensor computations
    pub fn distribute_tensor_rewards(
        &mut self,
        task_id: String,
        miner: String,
        computation_result: Vec<f32>,
    ) -> TribeResult<u64> {
        // In a real implementation, this would:
        // 1. Validate the computation result
        // 2. Calculate rewards based on task complexity and accuracy
        // 3. Transfer rewards to the miner
        // 4. Update contract state
        
        // For now, return a simulated reward
        let base_reward = 100;
        let complexity_bonus = computation_result.len() as u64;
        let total_reward = base_reward + complexity_bonus;
        
        Ok(total_reward)
    }

    /// Get contract execution statistics
    pub fn get_execution_stats(&self) -> ContractExecutionStats {
        ContractExecutionStats {
            total_contracts: self.deployed_contracts.len(),
            total_tokens: self.token_contracts.len(),
            total_staking_contracts: self.staking_contracts.len(),
            total_liquidity_pools: self.liquidity_pools.len(),
            total_gas_used: self.vm.total_gas_used(),
            successful_executions: self.vm.successful_executions(),
            failed_executions: self.vm.failed_executions(),
        }
    }
}

impl Default for ContractEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecutionStats {
    pub total_contracts: usize,
    pub total_tokens: usize,
    pub total_staking_contracts: usize,
    pub total_liquidity_pools: usize,
    pub total_gas_used: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_engine_creation() {
        let engine = ContractEngine::new();
        assert_eq!(engine.deployed_contracts.len(), 0);
        assert_eq!(engine.token_contracts.len(), 0);
    }

    #[test]
    fn test_token_creation() {
        let mut engine = ContractEngine::new();
        let token_id = engine.create_token(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        assert!(!token_id.is_empty());
        assert!(engine.token_contracts.contains_key(&token_id));
    }

    #[test]
    fn test_token_transfer() {
        let mut engine = ContractEngine::new();
        let token_id = engine.create_token(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        // Transfer should work
        assert!(engine.transfer_token(
            token_id.clone(),
            "creator".to_string(),
            "recipient".to_string(),
            1000,
        ).is_ok());

        // Check balances
        assert_eq!(engine.get_token_balance(&token_id, "creator"), 999000);
        assert_eq!(engine.get_token_balance(&token_id, "recipient"), 1000);
    }

    #[test]
    fn test_staking_contract() {
        let mut engine = ContractEngine::new();
        let token_id = engine.create_token(
            "Stake Token".to_string(),
            "STAKE".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        let staking_id = engine.create_staking_contract(
            token_id,
            "validator".to_string(),
            1000,
            0.1, // 10% APR
        ).unwrap();

        assert!(!staking_id.is_empty());
        assert!(engine.staking_contracts.contains_key(&staking_id));
    }

    #[test]
    fn test_liquidity_pool() {
        let mut engine = ContractEngine::new();
        let token_a = engine.create_token(
            "Token A".to_string(),
            "TOKA".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        let token_b = engine.create_token(
            "Token B".to_string(),
            "TOKB".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        let pool_id = engine.create_liquidity_pool(
            token_a,
            token_b,
            0.003, // 0.3% fee
        ).unwrap();

        assert!(!pool_id.is_empty());
        assert!(engine.liquidity_pools.contains_key(&pool_id));
    }
} 