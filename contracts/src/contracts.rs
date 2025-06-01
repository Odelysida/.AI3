use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Contract types supported by the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractType {
    Token,
    Staking,
    Liquidity,
    TensorCompute,
    Custom,
}

/// Smart contract structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub address: String,
    pub contract_type: ContractType,
    pub code: Vec<u8>,
    pub constructor_args: Vec<u8>,
    pub deployer: String,
    pub deployed_at: DateTime<Utc>,
    pub version: String,
    pub metadata: ContractMetadata,
    pub state: ContractState,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub source_url: Option<String>,
    pub abi: Vec<MethodSignature>,
    pub events: Vec<EventSignature>,
}

/// Contract state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractState {
    pub is_active: bool,
    pub is_paused: bool,
    pub owner: String,
    pub balance: u64,
    pub storage_size: usize,
    pub last_updated: DateTime<Utc>,
    pub execution_count: u64,
    pub gas_consumed: u64,
}

/// Method signature for ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub inputs: Vec<Parameter>,
    pub outputs: Vec<Parameter>,
    pub is_payable: bool,
    pub is_view: bool,
    pub gas_estimate: u64,
}

/// Event signature for ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSignature {
    pub name: String,
    pub inputs: Vec<Parameter>,
    pub anonymous: bool,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub indexed: bool, // For events
}

/// Parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    Bool,
    String,
    Bytes,
    Address,
    Array(Box<ParameterType>),
    Tuple(Vec<ParameterType>),
}

/// Contract deployment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployment {
    pub contract_type: ContractType,
    pub code: Vec<u8>,
    pub constructor_args: Vec<u8>,
    pub deployer: String,
    pub gas_limit: Option<u64>,
    pub value: u64,
    pub metadata: ContractMetadata,
    pub salt: Option<Vec<u8>>, // For deterministic addresses
}

/// Contract method call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_address: String,
    pub method: String,
    pub args: Vec<u8>,
    pub caller: String,
    pub value: u64,
    pub gas_limit: Option<u64>,
    pub nonce: u64,
}

/// Contract upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractUpgrade {
    pub contract_address: String,
    pub new_code: Vec<u8>,
    pub migration_data: Vec<u8>,
    pub proposer: String,
    pub voting_period: Duration,
    pub required_votes: u64,
    pub current_votes: u64,
    pub status: UpgradeStatus,
}

/// Upgrade status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpgradeStatus {
    Proposed,
    Voting,
    Approved,
    Rejected,
    Executed,
    Failed,
}

impl Contract {
    /// Create a new contract
    pub fn new(
        address: String,
        contract_type: ContractType,
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        deployer: String,
    ) -> Self {
        Self {
            address,
            contract_type,
            code,
            constructor_args,
            deployer: deployer.clone(),
            deployed_at: Utc::now(),
            version: "1.0.0".to_string(),
            metadata: ContractMetadata::default(),
            state: ContractState {
                is_active: true,
                is_paused: false,
                owner: deployer,
                balance: 0,
                storage_size: 0,
                last_updated: Utc::now(),
                execution_count: 0,
                gas_consumed: 0,
            },
        }
    }

    /// Create a contract with metadata
    pub fn new_with_metadata(
        address: String,
        contract_type: ContractType,
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        deployer: String,
        metadata: ContractMetadata,
    ) -> Self {
        let mut contract = Self::new(address, contract_type, code, constructor_args, deployer);
        contract.metadata = metadata;
        contract
    }

    /// Check if contract is callable
    pub fn is_callable(&self) -> bool {
        self.state.is_active && !self.state.is_paused
    }

    /// Pause contract execution
    pub fn pause(&mut self, caller: &str) -> TribeResult<()> {
        if caller != self.state.owner {
            return Err(TribeError::InvalidOperation("Only owner can pause contract".to_string()));
        }
        self.state.is_paused = true;
        self.state.last_updated = Utc::now();
        Ok(())
    }

    /// Resume contract execution
    pub fn resume(&mut self, caller: &str) -> TribeResult<()> {
        if caller != self.state.owner {
            return Err(TribeError::InvalidOperation("Only owner can resume contract".to_string()));
        }
        self.state.is_paused = false;
        self.state.last_updated = Utc::now();
        Ok(())
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, current_owner: &str, new_owner: String) -> TribeResult<()> {
        if current_owner != self.state.owner {
            return Err(TribeError::InvalidOperation("Only current owner can transfer ownership".to_string()));
        }
        self.state.owner = new_owner;
        self.state.last_updated = Utc::now();
        Ok(())
    }

    /// Update execution statistics
    pub fn update_execution_stats(&mut self, gas_used: u64) {
        self.state.execution_count += 1;
        self.state.gas_consumed += gas_used;
        self.state.last_updated = Utc::now();
    }

    /// Get method signature by name
    pub fn get_method_signature(&self, method_name: &str) -> Option<&MethodSignature> {
        self.metadata.abi.iter().find(|m| m.name == method_name)
    }

    /// Get event signature by name
    pub fn get_event_signature(&self, event_name: &str) -> Option<&EventSignature> {
        self.metadata.events.iter().find(|e| e.name == event_name)
    }

    /// Validate method call
    pub fn validate_method_call(&self, call: &ContractCall) -> TribeResult<()> {
        if !self.is_callable() {
            return Err(TribeError::InvalidOperation("Contract is not callable".to_string()));
        }

        if let Some(method_sig) = self.get_method_signature(&call.method) {
            // Validate gas limit
            if let Some(gas_limit) = call.gas_limit {
                if gas_limit < method_sig.gas_estimate {
                    return Err(TribeError::InvalidOperation("Insufficient gas limit".to_string()));
                }
            }

            // Validate value for payable methods
            if call.value > 0 && !method_sig.is_payable {
                return Err(TribeError::InvalidOperation("Method is not payable".to_string()));
            }

            Ok(())
        } else {
            Err(TribeError::InvalidOperation("Method not found".to_string()))
        }
    }

    /// Get contract size in bytes
    pub fn get_size(&self) -> usize {
        self.code.len() + self.constructor_args.len() + self.state.storage_size
    }

    /// Check if contract supports interface
    pub fn supports_interface(&self, interface_id: &str) -> bool {
        // Standard interface IDs
        match interface_id {
            "ERC20" => matches!(self.contract_type, ContractType::Token),
            "ERC721" => false, // NFT support could be added
            "Staking" => matches!(self.contract_type, ContractType::Staking),
            "Liquidity" => matches!(self.contract_type, ContractType::Liquidity),
            "TensorCompute" => matches!(self.contract_type, ContractType::TensorCompute),
            _ => false,
        }
    }
}

impl ContractDeployment {
    /// Create a new deployment
    pub fn new(
        contract_type: ContractType,
        code: Vec<u8>,
        deployer: String,
    ) -> Self {
        Self {
            contract_type,
            code,
            constructor_args: Vec::new(),
            deployer,
            gas_limit: None,
            value: 0,
            metadata: ContractMetadata::default(),
            salt: None,
        }
    }

    /// Set constructor arguments
    pub fn with_constructor_args(mut self, args: Vec<u8>) -> Self {
        self.constructor_args = args;
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    /// Set value
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: ContractMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set salt for deterministic address
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.salt = Some(salt);
        self
    }

    /// Validate deployment
    pub fn validate(&self) -> TribeResult<()> {
        if self.code.is_empty() {
            return Err(TribeError::InvalidOperation("Contract code cannot be empty".to_string()));
        }

        if self.deployer.is_empty() {
            return Err(TribeError::InvalidOperation("Deployer address cannot be empty".to_string()));
        }

        // Check code size limits (1MB)
        if self.code.len() > 1024 * 1024 {
            return Err(TribeError::InvalidOperation("Contract code too large".to_string()));
        }

        Ok(())
    }
}

impl ContractCall {
    /// Create a new contract call
    pub fn new(
        contract_address: String,
        method: String,
        args: Vec<u8>,
        caller: String,
    ) -> Self {
        Self {
            contract_address,
            method,
            args,
            caller,
            value: 0,
            gas_limit: None,
            nonce: 0,
        }
    }

    /// Set value
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    /// Set gas limit
    pub fn with_gas_limit(mut self, gas_limit: u64) -> Self {
        self.gas_limit = Some(gas_limit);
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Validate call
    pub fn validate(&self) -> TribeResult<()> {
        if self.contract_address.is_empty() {
            return Err(TribeError::InvalidOperation("Contract address cannot be empty".to_string()));
        }

        if self.method.is_empty() {
            return Err(TribeError::InvalidOperation("Method name cannot be empty".to_string()));
        }

        if self.caller.is_empty() {
            return Err(TribeError::InvalidOperation("Caller address cannot be empty".to_string()));
        }

        Ok(())
    }
}

impl Default for ContractMetadata {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            description: "No description provided".to_string(),
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            license: "MIT".to_string(),
            source_url: None,
            abi: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl MethodSignature {
    /// Create a new method signature
    pub fn new(name: String, inputs: Vec<Parameter>, outputs: Vec<Parameter>) -> Self {
        Self {
            name,
            inputs,
            outputs,
            is_payable: false,
            is_view: false,
            gas_estimate: 21000, // Default gas estimate
        }
    }

    /// Mark as payable
    pub fn payable(mut self) -> Self {
        self.is_payable = true;
        self
    }

    /// Mark as view (read-only)
    pub fn view(mut self) -> Self {
        self.is_view = true;
        self
    }

    /// Set gas estimate
    pub fn with_gas_estimate(mut self, gas: u64) -> Self {
        self.gas_estimate = gas;
        self
    }
}

impl EventSignature {
    /// Create a new event signature
    pub fn new(name: String, inputs: Vec<Parameter>) -> Self {
        Self {
            name,
            inputs,
            anonymous: false,
        }
    }

    /// Mark as anonymous
    pub fn anonymous(mut self) -> Self {
        self.anonymous = true;
        self
    }
}

impl Parameter {
    /// Create a new parameter
    pub fn new(name: String, param_type: ParameterType) -> Self {
        Self {
            name,
            param_type,
            indexed: false,
        }
    }

    /// Mark as indexed (for events)
    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }
}

use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let contract = Contract::new(
            "0x123".to_string(),
            ContractType::Token,
            vec![1, 2, 3],
            vec![],
            "deployer".to_string(),
        );

        assert_eq!(contract.address, "0x123");
        assert_eq!(contract.contract_type, ContractType::Token);
        assert_eq!(contract.deployer, "deployer");
        assert!(contract.is_callable());
    }

    #[test]
    fn test_contract_pause_resume() {
        let mut contract = Contract::new(
            "0x123".to_string(),
            ContractType::Token,
            vec![1, 2, 3],
            vec![],
            "owner".to_string(),
        );

        // Pause contract
        assert!(contract.pause("owner").is_ok());
        assert!(!contract.is_callable());

        // Resume contract
        assert!(contract.resume("owner").is_ok());
        assert!(contract.is_callable());

        // Non-owner cannot pause
        assert!(contract.pause("other").is_err());
    }

    #[test]
    fn test_contract_deployment() {
        let deployment = ContractDeployment::new(
            ContractType::Token,
            vec![1, 2, 3],
            "deployer".to_string(),
        )
        .with_gas_limit(1000000)
        .with_value(100);

        assert_eq!(deployment.contract_type, ContractType::Token);
        assert_eq!(deployment.gas_limit, Some(1000000));
        assert_eq!(deployment.value, 100);
        assert!(deployment.validate().is_ok());
    }

    #[test]
    fn test_contract_call() {
        let call = ContractCall::new(
            "0x123".to_string(),
            "transfer".to_string(),
            vec![1, 2, 3],
            "caller".to_string(),
        )
        .with_value(100)
        .with_gas_limit(50000);

        assert_eq!(call.contract_address, "0x123");
        assert_eq!(call.method, "transfer");
        assert_eq!(call.value, 100);
        assert_eq!(call.gas_limit, Some(50000));
        assert!(call.validate().is_ok());
    }

    #[test]
    fn test_method_signature() {
        let method = MethodSignature::new(
            "transfer".to_string(),
            vec![
                Parameter::new("to".to_string(), ParameterType::Address),
                Parameter::new("amount".to_string(), ParameterType::Uint256),
            ],
            vec![Parameter::new("success".to_string(), ParameterType::Bool)],
        )
        .payable()
        .with_gas_estimate(21000);

        assert_eq!(method.name, "transfer");
        assert!(method.is_payable);
        assert!(!method.is_view);
        assert_eq!(method.gas_estimate, 21000);
    }

    #[test]
    fn test_contract_interface_support() {
        let token_contract = Contract::new(
            "0x123".to_string(),
            ContractType::Token,
            vec![1, 2, 3],
            vec![],
            "deployer".to_string(),
        );

        assert!(token_contract.supports_interface("ERC20"));
        assert!(!token_contract.supports_interface("ERC721"));
        assert!(!token_contract.supports_interface("Staking"));
    }
} 