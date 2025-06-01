use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Virtual machine for executing smart contracts
#[derive(Debug)]
pub struct ContractVM {
    pub state: VMState,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub execution_stack: Vec<ExecutionFrame>,
    pub memory: Vec<u8>,
    pub storage: HashMap<String, Vec<u8>>,
    pub call_depth: usize,
    pub max_call_depth: usize,
    pub execution_timeout: Duration,
    pub stats: VMStats,
}

/// VM execution state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VMState {
    Ready,
    Running,
    Paused,
    Completed,
    Error(String),
    OutOfGas,
    Timeout,
}

/// Execution frame for call stack
#[derive(Debug, Clone)]
pub struct ExecutionFrame {
    pub contract_address: String,
    pub method: String,
    pub args: Vec<u8>,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub local_storage: HashMap<String, Vec<u8>>,
    pub return_data: Option<Vec<u8>>,
}

/// VM execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub error: Option<String>,
    pub logs: Vec<LogEntry>,
    pub state_changes: HashMap<String, Vec<u8>>,
    pub execution_time: Duration,
}

/// Log entry for contract events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub contract_address: String,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// VM error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VMError {
    OutOfGas,
    StackOverflow,
    InvalidOpcode,
    InvalidMemoryAccess,
    InvalidStorageAccess,
    ContractNotFound,
    MethodNotFound,
    InvalidArguments,
    ExecutionTimeout,
    SecurityViolation(String),
    InternalError(String),
}

/// VM execution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VMStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_gas_used: u64,
    pub average_execution_time: Duration,
    pub max_execution_time: Duration,
    pub contracts_deployed: u64,
    pub methods_called: HashMap<String, u64>,
}

impl ContractVM {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self {
            state: VMState::Ready,
            gas_limit: 1_000_000, // Default gas limit
            gas_used: 0,
            execution_stack: Vec::new(),
            memory: Vec::with_capacity(1024 * 1024), // 1MB initial memory
            storage: HashMap::new(),
            call_depth: 0,
            max_call_depth: 10,
            execution_timeout: Duration::from_secs(30),
            stats: VMStats::default(),
        }
    }

    /// Deploy a contract
    pub fn deploy(&mut self, deployment: super::ContractDeployment) -> TribeResult<String> {
        let start_time = Instant::now();
        self.state = VMState::Running;
        self.gas_used = 0;

        // Generate contract address
        let contract_address = self.generate_contract_address(&deployment.deployer, &deployment.code);

        // Validate deployment
        self.validate_deployment(&deployment)?;

        // Execute constructor if present
        if !deployment.constructor_args.is_empty() {
            let constructor_result = self.execute_constructor(&contract_address, &deployment)?;
            if !constructor_result.success {
                self.state = VMState::Error(constructor_result.error.unwrap_or_default());
                return Err(TribeError::InvalidOperation("Constructor execution failed".to_string()));
            }
        }

        // Store contract code
        let code_key = format!("contract:{}:code", contract_address);
        self.storage.insert(code_key, deployment.code);

        // Update statistics
        self.stats.contracts_deployed += 1;
        self.stats.total_executions += 1;
        self.stats.successful_executions += 1;
        self.stats.total_gas_used += self.gas_used;

        let execution_time = start_time.elapsed();
        self.update_execution_time_stats(execution_time);

        self.state = VMState::Completed;
        Ok(contract_address)
    }

    /// Call a contract method
    pub fn call(&mut self, contract: &super::Contract, call: super::ContractCall) -> TribeResult<ExecutionResult> {
        let start_time = Instant::now();
        self.state = VMState::Running;
        self.gas_used = 0;
        self.call_depth = 0;

        // Set gas limit for this execution
        self.gas_limit = call.gas_limit.unwrap_or(1_000_000);

        // Create execution frame
        let frame = ExecutionFrame {
            contract_address: call.contract_address.clone(),
            method: call.method.clone(),
            args: call.args.clone(),
            gas_limit: self.gas_limit,
            gas_used: 0,
            local_storage: HashMap::new(),
            return_data: None,
        };

        self.execution_stack.push(frame);

        // Execute the method
        let result = self.execute_method(contract, &call);

        // Calculate execution time
        let execution_time = start_time.elapsed();

        // Update statistics
        self.stats.total_executions += 1;
        if result.success {
            self.stats.successful_executions += 1;
        } else {
            self.stats.failed_executions += 1;
        }
        self.stats.total_gas_used += result.gas_used;
        self.update_execution_time_stats(execution_time);

        // Update method call statistics
        let method_key = format!("{}::{}", call.contract_address, call.method);
        *self.stats.methods_called.entry(method_key).or_insert(0) += 1;

        self.execution_stack.clear();
        self.state = if result.success { VMState::Completed } else { VMState::Error(result.error.clone().unwrap_or_default()) };

        Ok(result)
    }

    /// Execute a contract method
    fn execute_method(&mut self, contract: &super::Contract, call: &super::ContractCall) -> ExecutionResult {
        let mut logs = Vec::new();
        let mut state_changes = HashMap::new();

        // Check execution timeout
        let start_time = Instant::now();

        // Simulate method execution based on contract type
        match contract.contract_type {
            super::ContractType::Token => {
                self.execute_token_method(call, &mut logs, &mut state_changes)
            }
            super::ContractType::Staking => {
                self.execute_staking_method(call, &mut logs, &mut state_changes)
            }
            super::ContractType::Liquidity => {
                self.execute_liquidity_method(call, &mut logs, &mut state_changes)
            }
            super::ContractType::TensorCompute => {
                self.execute_tensor_method(call, &mut logs, &mut state_changes)
            }
            super::ContractType::Custom => {
                self.execute_custom_method(call, &mut logs, &mut state_changes)
            }
        }
    }

    /// Execute token contract method
    fn execute_token_method(
        &mut self,
        call: &super::ContractCall,
        logs: &mut Vec<LogEntry>,
        state_changes: &mut HashMap<String, Vec<u8>>,
    ) -> ExecutionResult {
        let gas_cost = match call.method.as_str() {
            "transfer" => 21000,
            "approve" => 15000,
            "mint" => 30000,
            "burn" => 25000,
            _ => 10000,
        };

        if !self.consume_gas(gas_cost) {
            return ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas".to_string()),
                logs: logs.clone(),
                state_changes: state_changes.clone(),
                execution_time: Duration::from_millis(0),
            };
        }

        // Simulate successful token operation
        let log = LogEntry {
            contract_address: call.contract_address.clone(),
            topics: vec![call.method.clone()],
            data: call.args.clone(),
            timestamp: chrono::Utc::now(),
        };
        logs.push(log);

        ExecutionResult {
            success: true,
            return_data: vec![1], // Success indicator
            gas_used: self.gas_used,
            error: None,
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            execution_time: Duration::from_millis(10),
        }
    }

    /// Execute staking contract method
    fn execute_staking_method(
        &mut self,
        call: &super::ContractCall,
        logs: &mut Vec<LogEntry>,
        state_changes: &mut HashMap<String, Vec<u8>>,
    ) -> ExecutionResult {
        let gas_cost = match call.method.as_str() {
            "stake" => 50000,
            "unstake" => 40000,
            "claim_rewards" => 30000,
            "delegate" => 35000,
            _ => 20000,
        };

        if !self.consume_gas(gas_cost) {
            return ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas".to_string()),
                logs: logs.clone(),
                state_changes: state_changes.clone(),
                execution_time: Duration::from_millis(0),
            };
        }

        // Simulate staking operation
        let log = LogEntry {
            contract_address: call.contract_address.clone(),
            topics: vec![call.method.clone(), "staking".to_string()],
            data: call.args.clone(),
            timestamp: chrono::Utc::now(),
        };
        logs.push(log);

        ExecutionResult {
            success: true,
            return_data: vec![1],
            gas_used: self.gas_used,
            error: None,
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            execution_time: Duration::from_millis(20),
        }
    }

    /// Execute liquidity contract method
    fn execute_liquidity_method(
        &mut self,
        call: &super::ContractCall,
        logs: &mut Vec<LogEntry>,
        state_changes: &mut HashMap<String, Vec<u8>>,
    ) -> ExecutionResult {
        let gas_cost = match call.method.as_str() {
            "add_liquidity" => 60000,
            "remove_liquidity" => 50000,
            "swap" => 40000,
            "get_price" => 5000,
            _ => 25000,
        };

        if !self.consume_gas(gas_cost) {
            return ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas".to_string()),
                logs: logs.clone(),
                state_changes: state_changes.clone(),
                execution_time: Duration::from_millis(0),
            };
        }

        // Simulate liquidity operation
        let log = LogEntry {
            contract_address: call.contract_address.clone(),
            topics: vec![call.method.clone(), "liquidity".to_string()],
            data: call.args.clone(),
            timestamp: chrono::Utc::now(),
        };
        logs.push(log);

        ExecutionResult {
            success: true,
            return_data: vec![1],
            gas_used: self.gas_used,
            error: None,
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            execution_time: Duration::from_millis(30),
        }
    }

    /// Execute tensor computation method
    fn execute_tensor_method(
        &mut self,
        call: &super::ContractCall,
        logs: &mut Vec<LogEntry>,
        state_changes: &mut HashMap<String, Vec<u8>>,
    ) -> ExecutionResult {
        let gas_cost = match call.method.as_str() {
            "submit_task" => 100000,
            "validate_result" => 80000,
            "distribute_rewards" => 60000,
            "get_task_status" => 10000,
            _ => 50000,
        };

        if !self.consume_gas(gas_cost) {
            return ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas".to_string()),
                logs: logs.clone(),
                state_changes: state_changes.clone(),
                execution_time: Duration::from_millis(0),
            };
        }

        // Simulate tensor computation
        let log = LogEntry {
            contract_address: call.contract_address.clone(),
            topics: vec![call.method.clone(), "tensor_compute".to_string()],
            data: call.args.clone(),
            timestamp: chrono::Utc::now(),
        };
        logs.push(log);

        ExecutionResult {
            success: true,
            return_data: vec![1],
            gas_used: self.gas_used,
            error: None,
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            execution_time: Duration::from_millis(50),
        }
    }

    /// Execute custom contract method
    fn execute_custom_method(
        &mut self,
        call: &super::ContractCall,
        logs: &mut Vec<LogEntry>,
        state_changes: &mut HashMap<String, Vec<u8>>,
    ) -> ExecutionResult {
        let base_gas_cost = 20000;
        let arg_gas_cost = call.args.len() as u64 * 100;
        let total_gas_cost = base_gas_cost + arg_gas_cost;

        if !self.consume_gas(total_gas_cost) {
            return ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas".to_string()),
                logs: logs.clone(),
                state_changes: state_changes.clone(),
                execution_time: Duration::from_millis(0),
            };
        }

        // Simulate custom method execution
        let log = LogEntry {
            contract_address: call.contract_address.clone(),
            topics: vec![call.method.clone(), "custom".to_string()],
            data: call.args.clone(),
            timestamp: chrono::Utc::now(),
        };
        logs.push(log);

        ExecutionResult {
            success: true,
            return_data: vec![1],
            gas_used: self.gas_used,
            error: None,
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            execution_time: Duration::from_millis(15),
        }
    }

    /// Execute constructor
    fn execute_constructor(
        &mut self,
        contract_address: &str,
        deployment: &super::ContractDeployment,
    ) -> TribeResult<ExecutionResult> {
        let gas_cost = 100000; // Base constructor cost

        if !self.consume_gas(gas_cost) {
            return Ok(ExecutionResult {
                success: false,
                return_data: Vec::new(),
                gas_used: self.gas_used,
                error: Some("Out of gas during constructor".to_string()),
                logs: Vec::new(),
                state_changes: HashMap::new(),
                execution_time: Duration::from_millis(0),
            });
        }

        // Simulate constructor execution
        let log = LogEntry {
            contract_address: contract_address.to_string(),
            topics: vec!["constructor".to_string()],
            data: deployment.constructor_args.clone(),
            timestamp: chrono::Utc::now(),
        };

        Ok(ExecutionResult {
            success: true,
            return_data: vec![1],
            gas_used: self.gas_used,
            error: None,
            logs: vec![log],
            state_changes: HashMap::new(),
            execution_time: Duration::from_millis(25),
        })
    }

    /// Validate contract deployment
    fn validate_deployment(&self, deployment: &super::ContractDeployment) -> TribeResult<()> {
        // Check code size limits
        if deployment.code.len() > 1024 * 1024 { // 1MB limit
            return Err(TribeError::InvalidOperation("Contract code too large".to_string()));
        }

        // Check for malicious patterns (simplified)
        if deployment.code.is_empty() {
            return Err(TribeError::InvalidOperation("Empty contract code".to_string()));
        }

        Ok(())
    }

    /// Generate contract address
    fn generate_contract_address(&self, deployer: &str, code: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(deployer.as_bytes());
        hasher.update(code);
        hasher.update(&self.stats.contracts_deployed.to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..20]) // Use first 20 bytes as address
    }

    /// Consume gas for operation
    fn consume_gas(&mut self, amount: u64) -> bool {
        if self.gas_used + amount > self.gas_limit {
            self.state = VMState::OutOfGas;
            false
        } else {
            self.gas_used += amount;
            true
        }
    }

    /// Update execution time statistics
    fn update_execution_time_stats(&mut self, execution_time: Duration) {
        if execution_time > self.stats.max_execution_time {
            self.stats.max_execution_time = execution_time;
        }

        // Update average (simplified)
        let total_time = self.stats.average_execution_time.as_millis() as u64 * (self.stats.total_executions - 1);
        let new_total = total_time + execution_time.as_millis() as u64;
        self.stats.average_execution_time = Duration::from_millis(new_total / self.stats.total_executions);
    }

    /// Get VM statistics
    pub fn get_stats(&self) -> &VMStats {
        &self.stats
    }

    /// Reset VM state
    pub fn reset(&mut self) {
        self.state = VMState::Ready;
        self.gas_used = 0;
        self.execution_stack.clear();
        self.memory.clear();
        self.call_depth = 0;
    }

    /// Set gas limit
    pub fn set_gas_limit(&mut self, limit: u64) {
        self.gas_limit = limit;
    }

    /// Set execution timeout
    pub fn set_execution_timeout(&mut self, timeout: Duration) {
        self.execution_timeout = timeout;
    }

    /// Get total gas used across all executions
    pub fn total_gas_used(&self) -> u64 {
        self.stats.total_gas_used
    }

    /// Get successful execution count
    pub fn successful_executions(&self) -> u64 {
        self.stats.successful_executions
    }

    /// Get failed execution count
    pub fn failed_executions(&self) -> u64 {
        self.stats.failed_executions
    }
}

impl Default for ContractVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let vm = ContractVM::new();
        assert_eq!(vm.state, VMState::Ready);
        assert_eq!(vm.gas_used, 0);
        assert_eq!(vm.call_depth, 0);
    }

    #[test]
    fn test_gas_consumption() {
        let mut vm = ContractVM::new();
        vm.gas_limit = 1000;
        
        assert!(vm.consume_gas(500));
        assert_eq!(vm.gas_used, 500);
        
        assert!(vm.consume_gas(400));
        assert_eq!(vm.gas_used, 900);
        
        assert!(!vm.consume_gas(200)); // Should fail
        assert_eq!(vm.state, VMState::OutOfGas);
    }

    #[test]
    fn test_contract_address_generation() {
        let vm = ContractVM::new();
        let address1 = vm.generate_contract_address("deployer1", b"code1");
        let address2 = vm.generate_contract_address("deployer1", b"code2");
        let address3 = vm.generate_contract_address("deployer2", b"code1");
        
        assert_ne!(address1, address2);
        assert_ne!(address1, address3);
        assert_ne!(address2, address3);
        assert_eq!(address1.len(), 40); // 20 bytes in hex
    }

    #[test]
    fn test_vm_reset() {
        let mut vm = ContractVM::new();
        vm.gas_used = 1000;
        vm.state = VMState::Running;
        vm.call_depth = 5;
        
        vm.reset();
        
        assert_eq!(vm.state, VMState::Ready);
        assert_eq!(vm.gas_used, 0);
        assert_eq!(vm.call_depth, 0);
        assert!(vm.execution_stack.is_empty());
    }
} 