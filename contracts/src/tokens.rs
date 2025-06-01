use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Token contract implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenContract {
    pub token_info: TokenInfo,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>, // owner -> spender -> amount
    pub total_supply: u64,
    pub max_supply: Option<u64>,
    pub is_mintable: bool,
    pub is_burnable: bool,
    pub is_pausable: bool,
    pub is_paused: bool,
    pub owner: String,
    pub minters: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub description: String,
    pub icon_url: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
}

/// Token operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenOperation {
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
    Approve {
        owner: String,
        spender: String,
        amount: u64,
    },
    TransferFrom {
        spender: String,
        from: String,
        to: String,
        amount: u64,
    },
    Mint {
        to: String,
        amount: u64,
    },
    Burn {
        from: String,
        amount: u64,
    },
    Pause,
    Unpause,
    AddMinter {
        minter: String,
    },
    RemoveMinter {
        minter: String,
    },
    TransferOwnership {
        new_owner: String,
    },
}

/// Token balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub address: String,
    pub balance: u64,
    pub locked_balance: u64,
    pub last_updated: DateTime<Utc>,
}

/// Token transfer event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub transaction_hash: String,
}

/// Token approval event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvent {
    pub owner: String,
    pub spender: String,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub transaction_hash: String,
}

/// Token statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub holders_count: usize,
    pub total_transfers: u64,
    pub total_minted: u64,
    pub total_burned: u64,
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
}

impl TokenContract {
    /// Create a new token contract
    pub fn new(
        name: String,
        symbol: String,
        total_supply: u64,
        decimals: u8,
        creator: String,
    ) -> TribeResult<Self> {
        if name.is_empty() || symbol.is_empty() {
            return Err(TribeError::InvalidOperation("Token name and symbol cannot be empty".to_string()));
        }

        if total_supply == 0 {
            return Err(TribeError::InvalidOperation("Total supply must be greater than 0".to_string()));
        }

        let token_id = Self::generate_token_id(&name, &symbol, &creator);
        let mut balances = HashMap::new();
        balances.insert(creator.clone(), total_supply);

        Ok(Self {
            token_info: TokenInfo {
                id: token_id,
                name,
                symbol,
                decimals,
                description: String::new(),
                icon_url: None,
                website: None,
                social_links: HashMap::new(),
            },
            balances,
            allowances: HashMap::new(),
            total_supply,
            max_supply: None,
            is_mintable: false,
            is_burnable: false,
            is_pausable: false,
            is_paused: false,
            owner: creator.clone(),
            minters: vec![creator],
            created_at: Utc::now(),
            last_updated: Utc::now(),
        })
    }

    /// Create a mintable token
    pub fn new_mintable(
        name: String,
        symbol: String,
        initial_supply: u64,
        max_supply: Option<u64>,
        decimals: u8,
        creator: String,
    ) -> TribeResult<Self> {
        let mut token = Self::new(name, symbol, initial_supply, decimals, creator)?;
        token.is_mintable = true;
        token.max_supply = max_supply;
        Ok(token)
    }

    /// Create a burnable token
    pub fn new_burnable(
        name: String,
        symbol: String,
        total_supply: u64,
        decimals: u8,
        creator: String,
    ) -> TribeResult<Self> {
        let mut token = Self::new(name, symbol, total_supply, decimals, creator)?;
        token.is_burnable = true;
        Ok(token)
    }

    /// Transfer tokens
    pub fn transfer(&mut self, from: String, to: String, amount: u64) -> TribeResult<()> {
        if self.is_paused {
            return Err(TribeError::InvalidOperation("Token transfers are paused".to_string()));
        }

        if from == to {
            return Err(TribeError::InvalidOperation("Cannot transfer to self".to_string()));
        }

        if amount == 0 {
            return Err(TribeError::InvalidOperation("Transfer amount must be greater than 0".to_string()));
        }

        let from_balance = self.balances.get(&from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err(TribeError::InvalidOperation("Insufficient balance".to_string()));
        }

        // Update balances
        self.balances.insert(from.clone(), from_balance - amount);
        let to_balance = self.balances.get(&to).copied().unwrap_or(0);
        self.balances.insert(to.clone(), to_balance + amount);

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Approve spender to spend tokens
    pub fn approve(&mut self, owner: String, spender: String, amount: u64) -> TribeResult<()> {
        if self.is_paused {
            return Err(TribeError::InvalidOperation("Token operations are paused".to_string()));
        }

        if owner == spender {
            return Err(TribeError::InvalidOperation("Cannot approve self".to_string()));
        }

        self.allowances
            .entry(owner)
            .or_insert_with(HashMap::new)
            .insert(spender, amount);

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Transfer tokens from one account to another using allowance
    pub fn transfer_from(
        &mut self,
        spender: String,
        from: String,
        to: String,
        amount: u64,
    ) -> TribeResult<()> {
        if self.is_paused {
            return Err(TribeError::InvalidOperation("Token transfers are paused".to_string()));
        }

        // Check allowance
        let allowance = self.allowances
            .get(&from)
            .and_then(|allowances| allowances.get(&spender))
            .copied()
            .unwrap_or(0);

        if allowance < amount {
            return Err(TribeError::InvalidOperation("Insufficient allowance".to_string()));
        }

        // Perform transfer
        self.transfer(from.clone(), to, amount)?;

        // Update allowance
        if allowance != u64::MAX { // u64::MAX represents unlimited allowance
            self.allowances
                .get_mut(&from)
                .unwrap()
                .insert(spender, allowance - amount);
        }

        Ok(())
    }

    /// Mint new tokens
    pub fn mint(&mut self, to: String, amount: u64, minter: String) -> TribeResult<()> {
        if !self.is_mintable {
            return Err(TribeError::InvalidOperation("Token is not mintable".to_string()));
        }

        if !self.minters.contains(&minter) {
            return Err(TribeError::InvalidOperation("Caller is not authorized to mint".to_string()));
        }

        if self.is_paused {
            return Err(TribeError::InvalidOperation("Token operations are paused".to_string()));
        }

        if amount == 0 {
            return Err(TribeError::InvalidOperation("Mint amount must be greater than 0".to_string()));
        }

        // Check max supply
        if let Some(max_supply) = self.max_supply {
            if self.total_supply + amount > max_supply {
                return Err(TribeError::InvalidOperation("Minting would exceed max supply".to_string()));
            }
        }

        // Update balances and total supply
        let to_balance = self.balances.get(&to).copied().unwrap_or(0);
        self.balances.insert(to, to_balance + amount);
        self.total_supply += amount;

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Burn tokens
    pub fn burn(&mut self, from: String, amount: u64) -> TribeResult<()> {
        if !self.is_burnable {
            return Err(TribeError::InvalidOperation("Token is not burnable".to_string()));
        }

        if self.is_paused {
            return Err(TribeError::InvalidOperation("Token operations are paused".to_string()));
        }

        if amount == 0 {
            return Err(TribeError::InvalidOperation("Burn amount must be greater than 0".to_string()));
        }

        let from_balance = self.balances.get(&from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err(TribeError::InvalidOperation("Insufficient balance to burn".to_string()));
        }

        // Update balance and total supply
        self.balances.insert(from, from_balance - amount);
        self.total_supply -= amount;

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Pause token operations
    pub fn pause(&mut self, caller: String) -> TribeResult<()> {
        if !self.is_pausable {
            return Err(TribeError::InvalidOperation("Token is not pausable".to_string()));
        }

        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can pause token".to_string()));
        }

        self.is_paused = true;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Unpause token operations
    pub fn unpause(&mut self, caller: String) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can unpause token".to_string()));
        }

        self.is_paused = false;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Add a minter
    pub fn add_minter(&mut self, minter: String, caller: String) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can add minters".to_string()));
        }

        if !self.minters.contains(&minter) {
            self.minters.push(minter);
            self.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Remove a minter
    pub fn remove_minter(&mut self, minter: String, caller: String) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can remove minters".to_string()));
        }

        self.minters.retain(|m| m != &minter);
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Transfer ownership
    pub fn transfer_ownership(&mut self, new_owner: String, caller: String) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can transfer ownership".to_string()));
        }

        self.owner = new_owner;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Get balance of an address
    pub fn balance_of(&self, address: &str) -> u64 {
        self.balances.get(address).copied().unwrap_or(0)
    }

    /// Get allowance
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0)
    }

    /// Get token statistics
    pub fn get_stats(&self) -> TokenStats {
        let holders_count = self.balances.iter().filter(|(_, &balance)| balance > 0).count();
        let circulating_supply = self.total_supply; // Could be adjusted for locked tokens

        TokenStats {
            total_supply: self.total_supply,
            circulating_supply,
            holders_count,
            total_transfers: 0, // Would need to track this
            total_minted: 0,    // Would need to track this
            total_burned: 0,    // Would need to track this
            market_cap: None,
            price: None,
        }
    }

    /// Update token metadata
    pub fn update_metadata(
        &mut self,
        description: Option<String>,
        icon_url: Option<String>,
        website: Option<String>,
        caller: String,
    ) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can update metadata".to_string()));
        }

        if let Some(desc) = description {
            self.token_info.description = desc;
        }

        if let Some(icon) = icon_url {
            self.token_info.icon_url = Some(icon);
        }

        if let Some(site) = website {
            self.token_info.website = Some(site);
        }

        self.last_updated = Utc::now();
        Ok(())
    }

    /// Add social link
    pub fn add_social_link(&mut self, platform: String, url: String, caller: String) -> TribeResult<()> {
        if caller != self.owner {
            return Err(TribeError::InvalidOperation("Only owner can add social links".to_string()));
        }

        self.token_info.social_links.insert(platform, url);
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Generate token ID
    fn generate_token_id(name: &str, symbol: &str, creator: &str) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        hasher.update(symbol.as_bytes());
        hasher.update(creator.as_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..16]) // Use first 16 bytes as token ID
    }

    /// Validate token operation
    pub fn validate_operation(&self, operation: &TokenOperation, caller: &str) -> TribeResult<()> {
        match operation {
            TokenOperation::Transfer { from, amount, .. } => {
                if from != caller {
                    return Err(TribeError::InvalidOperation("Can only transfer own tokens".to_string()));
                }
                if self.balance_of(from) < *amount {
                    return Err(TribeError::InvalidOperation("Insufficient balance".to_string()));
                }
            }
            TokenOperation::Approve { owner, .. } => {
                if owner != caller {
                    return Err(TribeError::InvalidOperation("Can only approve own tokens".to_string()));
                }
            }
            TokenOperation::TransferFrom { spender, from, amount, .. } => {
                if spender != caller {
                    return Err(TribeError::InvalidOperation("Invalid spender".to_string()));
                }
                if self.allowance(from, spender) < *amount {
                    return Err(TribeError::InvalidOperation("Insufficient allowance".to_string()));
                }
            }
            TokenOperation::Mint { .. } => {
                if !self.minters.contains(&caller.to_string()) {
                    return Err(TribeError::InvalidOperation("Not authorized to mint".to_string()));
                }
            }
            TokenOperation::Burn { from, .. } => {
                if from != caller {
                    return Err(TribeError::InvalidOperation("Can only burn own tokens".to_string()));
                }
            }
            TokenOperation::Pause | TokenOperation::Unpause => {
                if caller != self.owner {
                    return Err(TribeError::InvalidOperation("Only owner can pause/unpause".to_string()));
                }
            }
            TokenOperation::AddMinter { .. } | TokenOperation::RemoveMinter { .. } => {
                if caller != self.owner {
                    return Err(TribeError::InvalidOperation("Only owner can manage minters".to_string()));
                }
            }
            TokenOperation::TransferOwnership { .. } => {
                if caller != self.owner {
                    return Err(TribeError::InvalidOperation("Only owner can transfer ownership".to_string()));
                }
            }
        }

        Ok(())
    }

    /// Execute token operation
    pub fn execute_operation(&mut self, operation: TokenOperation, caller: String) -> TribeResult<()> {
        self.validate_operation(&operation, &caller)?;

        match operation {
            TokenOperation::Transfer { from, to, amount } => {
                self.transfer(from, to, amount)
            }
            TokenOperation::Approve { owner, spender, amount } => {
                self.approve(owner, spender, amount)
            }
            TokenOperation::TransferFrom { spender, from, to, amount } => {
                self.transfer_from(spender, from, to, amount)
            }
            TokenOperation::Mint { to, amount } => {
                self.mint(to, amount, caller)
            }
            TokenOperation::Burn { from, amount } => {
                self.burn(from, amount)
            }
            TokenOperation::Pause => {
                self.pause(caller)
            }
            TokenOperation::Unpause => {
                self.unpause(caller)
            }
            TokenOperation::AddMinter { minter } => {
                self.add_minter(minter, caller)
            }
            TokenOperation::RemoveMinter { minter } => {
                self.remove_minter(minter, caller)
            }
            TokenOperation::TransferOwnership { new_owner } => {
                self.transfer_ownership(new_owner, caller)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = TokenContract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        assert_eq!(token.token_info.name, "Test Token");
        assert_eq!(token.token_info.symbol, "TEST");
        assert_eq!(token.total_supply, 1000000);
        assert_eq!(token.balance_of("creator"), 1000000);
    }

    #[test]
    fn test_token_transfer() {
        let mut token = TokenContract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        // Transfer tokens
        assert!(token.transfer("creator".to_string(), "recipient".to_string(), 1000).is_ok());
        assert_eq!(token.balance_of("creator"), 999000);
        assert_eq!(token.balance_of("recipient"), 1000);

        // Insufficient balance
        assert!(token.transfer("recipient".to_string(), "other".to_string(), 2000).is_err());
    }

    #[test]
    fn test_token_approval() {
        let mut token = TokenContract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        // Approve spender
        assert!(token.approve("creator".to_string(), "spender".to_string(), 1000).is_ok());
        assert_eq!(token.allowance("creator", "spender"), 1000);

        // Transfer from
        assert!(token.transfer_from(
            "spender".to_string(),
            "creator".to_string(),
            "recipient".to_string(),
            500
        ).is_ok());

        assert_eq!(token.balance_of("creator"), 999500);
        assert_eq!(token.balance_of("recipient"), 500);
        assert_eq!(token.allowance("creator", "spender"), 500);
    }

    #[test]
    fn test_mintable_token() {
        let mut token = TokenContract::new_mintable(
            "Mintable Token".to_string(),
            "MINT".to_string(),
            1000,
            Some(10000),
            6,
            "creator".to_string(),
        ).unwrap();

        // Mint tokens
        assert!(token.mint("recipient".to_string(), 500, "creator".to_string()).is_ok());
        assert_eq!(token.total_supply, 1500);
        assert_eq!(token.balance_of("recipient"), 500);

        // Exceed max supply
        assert!(token.mint("recipient".to_string(), 10000, "creator".to_string()).is_err());
    }

    #[test]
    fn test_burnable_token() {
        let mut token = TokenContract::new_burnable(
            "Burnable Token".to_string(),
            "BURN".to_string(),
            1000,
            6,
            "creator".to_string(),
        ).unwrap();

        // Burn tokens
        assert!(token.burn("creator".to_string(), 100).is_ok());
        assert_eq!(token.total_supply, 900);
        assert_eq!(token.balance_of("creator"), 900);

        // Insufficient balance
        assert!(token.burn("creator".to_string(), 1000).is_err());
    }

    #[test]
    fn test_token_pause() {
        let mut token = TokenContract::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            1000000,
            6,
            "creator".to_string(),
        ).unwrap();

        token.is_pausable = true;

        // Pause token
        assert!(token.pause("creator".to_string()).is_ok());
        assert!(token.is_paused);

        // Transfer should fail when paused
        assert!(token.transfer("creator".to_string(), "recipient".to_string(), 1000).is_err());

        // Unpause token
        assert!(token.unpause("creator".to_string()).is_ok());
        assert!(!token.is_paused);

        // Transfer should work after unpause
        assert!(token.transfer("creator".to_string(), "recipient".to_string(), 1000).is_ok());
    }
} 