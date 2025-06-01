use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Liquidity pool contract implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub id: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_liquidity: u64,
    pub liquidity_providers: HashMap<String, LiquidityPosition>,
    pub fee_rate: f64, // Trading fee percentage (e.g., 0.003 for 0.3%)
    pub protocol_fee_rate: f64, // Protocol fee percentage
    pub accumulated_fees_a: u64,
    pub accumulated_fees_b: u64,
    pub total_volume_a: u64,
    pub total_volume_b: u64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_trade: Option<DateTime<Utc>>,
    pub price_oracle: PriceOracle,
}

/// Individual liquidity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub provider: String,
    pub liquidity_tokens: u64,
    pub initial_a: u64,
    pub initial_b: u64,
    pub added_at: DateTime<Utc>,
    pub last_fee_claim: DateTime<Utc>,
    pub unclaimed_fees_a: u64,
    pub unclaimed_fees_b: u64,
    pub is_active: bool,
}

/// Price oracle for tracking price history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceOracle {
    pub current_price: f64, // Price of token A in terms of token B
    pub price_history: Vec<PricePoint>,
    pub twap_24h: f64, // Time-weighted average price over 24 hours
    pub last_update: DateTime<Utc>,
}

/// Price point for historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub volume_a: u64,
    pub volume_b: u64,
}

/// Swap operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapOperation {
    pub trader: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_paid: u64,
    pub price_impact: f64,
    pub timestamp: DateTime<Utc>,
}

/// Liquidity operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityOperation {
    pub provider: String,
    pub operation_type: LiquidityOperationType,
    pub amount_a: u64,
    pub amount_b: u64,
    pub liquidity_tokens: u64,
    pub timestamp: DateTime<Utc>,
}

/// Types of liquidity operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiquidityOperationType {
    Add,
    Remove,
    ClaimFees,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_value_locked: u64, // In terms of token A
    pub volume_24h: u64,
    pub fees_24h: u64,
    pub transactions_24h: u32,
    pub liquidity_providers_count: usize,
    pub current_apy: f64,
}

impl LiquidityPool {
    /// Create a new liquidity pool
    pub fn new(
        token_a: String,
        token_b: String,
        initial_a: u64,
        initial_b: u64,
        provider: String,
        fee_rate: f64,
    ) -> TribeResult<Self> {
        if token_a == token_b {
            return Err(TribeError::InvalidOperation("Cannot create pool with same tokens".to_string()));
        }

        if initial_a == 0 || initial_b == 0 {
            return Err(TribeError::InvalidOperation("Initial liquidity cannot be zero".to_string()));
        }

        if fee_rate < 0.0 || fee_rate > 0.1 {
            return Err(TribeError::InvalidOperation("Fee rate must be between 0 and 10%".to_string()));
        }

        let pool_id = Self::generate_pool_id(&token_a, &token_b);
        
        // Calculate initial liquidity tokens (geometric mean)
        let initial_liquidity = ((initial_a as f64 * initial_b as f64).sqrt()) as u64;
        
        let mut liquidity_providers = HashMap::new();
        liquidity_providers.insert(provider.clone(), LiquidityPosition {
            provider: provider.clone(),
            liquidity_tokens: initial_liquidity,
            initial_a,
            initial_b,
            added_at: Utc::now(),
            last_fee_claim: Utc::now(),
            unclaimed_fees_a: 0,
            unclaimed_fees_b: 0,
            is_active: true,
        });

        let initial_price = initial_b as f64 / initial_a as f64;
        let price_oracle = PriceOracle {
            current_price: initial_price,
            price_history: vec![PricePoint {
                price: initial_price,
                timestamp: Utc::now(),
                volume_a: 0,
                volume_b: 0,
            }],
            twap_24h: initial_price,
            last_update: Utc::now(),
        };

        Ok(Self {
            id: pool_id,
            token_a,
            token_b,
            reserve_a: initial_a,
            reserve_b: initial_b,
            total_liquidity: initial_liquidity,
            liquidity_providers,
            fee_rate,
            protocol_fee_rate: fee_rate * 0.1, // 10% of trading fees go to protocol
            accumulated_fees_a: 0,
            accumulated_fees_b: 0,
            total_volume_a: 0,
            total_volume_b: 0,
            is_active: true,
            created_at: Utc::now(),
            last_trade: None,
            price_oracle,
        })
    }

    /// Add liquidity to the pool
    pub fn add_liquidity(
        &mut self,
        provider: String,
        amount_a: u64,
        amount_b: u64,
        min_liquidity: u64,
    ) -> TribeResult<u64> {
        if !self.is_active {
            return Err(TribeError::InvalidOperation("Pool is not active".to_string()));
        }

        if amount_a == 0 || amount_b == 0 {
            return Err(TribeError::InvalidOperation("Amounts cannot be zero".to_string()));
        }

        // Calculate optimal amounts based on current ratio
        let ratio = self.reserve_b as f64 / self.reserve_a as f64;
        let optimal_b = (amount_a as f64 * ratio) as u64;
        let optimal_a = (amount_b as f64 / ratio) as u64;

        let (final_a, final_b) = if optimal_b <= amount_b {
            (amount_a, optimal_b)
        } else {
            (optimal_a, amount_b)
        };

        // Calculate liquidity tokens to mint
        let liquidity_tokens = if self.total_liquidity == 0 {
            ((final_a as f64 * final_b as f64).sqrt()) as u64
        } else {
            std::cmp::min(
                (final_a * self.total_liquidity) / self.reserve_a,
                (final_b * self.total_liquidity) / self.reserve_b,
            )
        };

        if liquidity_tokens < min_liquidity {
            return Err(TribeError::InvalidOperation("Insufficient liquidity tokens".to_string()));
        }

        // Update reserves
        self.reserve_a += final_a;
        self.reserve_b += final_b;
        self.total_liquidity += liquidity_tokens;

        // Update or create liquidity position
        if let Some(position) = self.liquidity_providers.get_mut(&provider) {
            position.liquidity_tokens += liquidity_tokens;
            position.initial_a += final_a;
            position.initial_b += final_b;
        } else {
            let position = LiquidityPosition {
                provider: provider.clone(),
                liquidity_tokens,
                initial_a: final_a,
                initial_b: final_b,
                added_at: Utc::now(),
                last_fee_claim: Utc::now(),
                unclaimed_fees_a: 0,
                unclaimed_fees_b: 0,
                is_active: true,
            };
            self.liquidity_providers.insert(provider, position);
        }

        Ok(liquidity_tokens)
    }

    /// Remove liquidity from the pool
    pub fn remove_liquidity(
        &mut self,
        provider: String,
        liquidity_tokens: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> TribeResult<(u64, u64)> {
        let position = self.liquidity_providers.get_mut(&provider)
            .ok_or_else(|| TribeError::InvalidOperation("No liquidity position found".to_string()))?;

        if !position.is_active {
            return Err(TribeError::InvalidOperation("Position is not active".to_string()));
        }

        if liquidity_tokens > position.liquidity_tokens {
            return Err(TribeError::InvalidOperation("Insufficient liquidity tokens".to_string()));
        }

        // Calculate amounts to return
        let amount_a = (liquidity_tokens * self.reserve_a) / self.total_liquidity;
        let amount_b = (liquidity_tokens * self.reserve_b) / self.total_liquidity;

        if amount_a < min_amount_a || amount_b < min_amount_b {
            return Err(TribeError::InvalidOperation("Amounts below minimum".to_string()));
        }

        // Update reserves
        self.reserve_a -= amount_a;
        self.reserve_b -= amount_b;
        self.total_liquidity -= liquidity_tokens;

        // Update position
        position.liquidity_tokens -= liquidity_tokens;
        if position.liquidity_tokens == 0 {
            position.is_active = false;
        }

        // Calculate and distribute fees before removal
        self.calculate_fees(&provider)?;

        Ok((amount_a, amount_b))
    }

    /// Swap tokens using constant product formula (x * y = k)
    pub fn swap(
        &mut self,
        trader: String,
        token_in: String,
        amount_in: u64,
        min_amount_out: u64,
    ) -> TribeResult<u64> {
        if !self.is_active {
            return Err(TribeError::InvalidOperation("Pool is not active".to_string()));
        }

        if amount_in == 0 {
            return Err(TribeError::InvalidOperation("Amount in cannot be zero".to_string()));
        }

        let (reserve_in, reserve_out, is_a_to_b) = if token_in == self.token_a {
            (self.reserve_a, self.reserve_b, true)
        } else if token_in == self.token_b {
            (self.reserve_b, self.reserve_a, false)
        } else {
            return Err(TribeError::InvalidOperation("Invalid token".to_string()));
        };

        // Calculate fee
        let fee = (amount_in as f64 * self.fee_rate) as u64;
        let amount_in_after_fee = amount_in - fee;

        // Calculate amount out using constant product formula
        // amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee)
        let amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee);

        if amount_out < min_amount_out {
            return Err(TribeError::InvalidOperation("Amount out below minimum".to_string()));
        }

        // Calculate price impact
        let price_before = reserve_out as f64 / reserve_in as f64;
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = reserve_out - amount_out;
        let price_after = new_reserve_out as f64 / new_reserve_in as f64;
        let price_impact = ((price_after - price_before) / price_before).abs();

        // Update reserves
        if is_a_to_b {
            self.reserve_a += amount_in;
            self.reserve_b -= amount_out;
            self.accumulated_fees_a += fee;
            self.total_volume_a += amount_in;
        } else {
            self.reserve_b += amount_in;
            self.reserve_a -= amount_out;
            self.accumulated_fees_b += fee;
            self.total_volume_b += amount_in;
        }

        // Update price oracle
        self.update_price_oracle()?;

        // Record trade
        self.last_trade = Some(Utc::now());

        // Distribute fees to liquidity providers
        self.distribute_fees(fee, is_a_to_b)?;

        Ok(amount_out)
    }

    /// Calculate the output amount for a given input (for price quotes)
    pub fn get_amount_out(&self, amount_in: u64, token_in: String) -> TribeResult<u64> {
        if amount_in == 0 {
            return Ok(0);
        }

        let (reserve_in, reserve_out) = if token_in == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else if token_in == self.token_b {
            (self.reserve_b, self.reserve_a)
        } else {
            return Err(TribeError::InvalidOperation("Invalid token".to_string()));
        };

        let fee = (amount_in as f64 * self.fee_rate) as u64;
        let amount_in_after_fee = amount_in - fee;
        let amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee);

        Ok(amount_out)
    }

    /// Calculate fees for a liquidity provider
    pub fn calculate_fees(&mut self, provider: &str) -> TribeResult<(u64, u64)> {
        let position = self.liquidity_providers.get_mut(provider)
            .ok_or_else(|| TribeError::InvalidOperation("No liquidity position found".to_string()))?;

        if !position.is_active {
            return Ok((0, 0));
        }

        // Calculate share of accumulated fees
        let share = position.liquidity_tokens as f64 / self.total_liquidity as f64;
        let fees_a = (self.accumulated_fees_a as f64 * share) as u64;
        let fees_b = (self.accumulated_fees_b as f64 * share) as u64;

        position.unclaimed_fees_a += fees_a;
        position.unclaimed_fees_b += fees_b;
        position.last_fee_claim = Utc::now();

        Ok((fees_a, fees_b))
    }

    /// Claim accumulated fees
    pub fn claim_fees(&mut self, provider: String) -> TribeResult<(u64, u64)> {
        // Calculate latest fees
        self.calculate_fees(&provider)?;

        let position = self.liquidity_providers.get_mut(&provider)
            .ok_or_else(|| TribeError::InvalidOperation("No liquidity position found".to_string()))?;

        let fees_a = position.unclaimed_fees_a;
        let fees_b = position.unclaimed_fees_b;

        if fees_a == 0 && fees_b == 0 {
            return Err(TribeError::InvalidOperation("No fees to claim".to_string()));
        }

        // Reset unclaimed fees
        position.unclaimed_fees_a = 0;
        position.unclaimed_fees_b = 0;

        Ok((fees_a, fees_b))
    }

    /// Distribute trading fees to liquidity providers
    fn distribute_fees(&mut self, fee: u64, is_token_a: bool) -> TribeResult<()> {
        let protocol_fee = (fee as f64 * self.protocol_fee_rate) as u64;
        let lp_fee = fee - protocol_fee;

        // Add LP fees to accumulated fees
        if is_token_a {
            self.accumulated_fees_a += lp_fee;
        } else {
            self.accumulated_fees_b += lp_fee;
        }

        Ok(())
    }

    /// Update price oracle with current price
    fn update_price_oracle(&mut self) -> TribeResult<()> {
        let current_price = self.reserve_b as f64 / self.reserve_a as f64;
        let now = Utc::now();

        self.price_oracle.current_price = current_price;
        self.price_oracle.last_update = now;

        // Add to price history
        self.price_oracle.price_history.push(PricePoint {
            price: current_price,
            timestamp: now,
            volume_a: self.total_volume_a,
            volume_b: self.total_volume_b,
        });

        // Keep only last 24 hours of price history
        let cutoff = now - chrono::Duration::hours(24);
        self.price_oracle.price_history.retain(|p| p.timestamp > cutoff);

        // Calculate TWAP
        self.calculate_twap()?;

        Ok(())
    }

    /// Calculate time-weighted average price
    fn calculate_twap(&mut self) -> TribeResult<()> {
        if self.price_oracle.price_history.len() < 2 {
            self.price_oracle.twap_24h = self.price_oracle.current_price;
            return Ok(());
        }

        let mut weighted_sum = 0.0;
        let mut total_time = 0.0;

        for i in 1..self.price_oracle.price_history.len() {
            let prev = &self.price_oracle.price_history[i - 1];
            let curr = &self.price_oracle.price_history[i];
            
            let time_diff = curr.timestamp.signed_duration_since(prev.timestamp).num_seconds() as f64;
            weighted_sum += prev.price * time_diff;
            total_time += time_diff;
        }

        if total_time > 0.0 {
            self.price_oracle.twap_24h = weighted_sum / total_time;
        }

        Ok(())
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let now = Utc::now();
        let cutoff_24h = now - chrono::Duration::hours(24);

        // Calculate 24h volume (simplified - would need more detailed tracking in production)
        let volume_24h = self.total_volume_a + self.total_volume_b;
        let fees_24h = self.accumulated_fees_a + self.accumulated_fees_b;

        // Calculate TVL in terms of token A
        let tvl = self.reserve_a + (self.reserve_b as f64 / self.price_oracle.current_price) as u64;

        // Calculate APY (simplified)
        let daily_fees = fees_24h as f64;
        let daily_yield = if tvl > 0 { daily_fees / tvl as f64 } else { 0.0 };
        let apy = daily_yield * 365.0;

        PoolStats {
            total_value_locked: tvl,
            volume_24h,
            fees_24h,
            transactions_24h: 0, // Would need transaction tracking
            liquidity_providers_count: self.liquidity_providers.len(),
            current_apy: apy,
        }
    }

    /// Generate pool ID
    fn generate_pool_id(token_a: &str, token_b: &str) -> String {
        use sha2::{Sha256, Digest};
        
        // Ensure consistent ordering
        let (first, second) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };
        
        let mut hasher = Sha256::new();
        hasher.update(first.as_bytes());
        hasher.update(second.as_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..16])
    }

    /// Get liquidity position for a provider
    pub fn get_position(&self, provider: &str) -> Option<&LiquidityPosition> {
        self.liquidity_providers.get(provider)
    }

    /// Get current price
    pub fn get_price(&self) -> f64 {
        self.price_oracle.current_price
    }

    /// Get price impact for a trade
    pub fn get_price_impact(&self, amount_in: u64, token_in: String) -> TribeResult<f64> {
        let (reserve_in, reserve_out) = if token_in == self.token_a {
            (self.reserve_a, self.reserve_b)
        } else if token_in == self.token_b {
            (self.reserve_b, self.reserve_a)
        } else {
            return Err(TribeError::InvalidOperation("Invalid token".to_string()));
        };

        let price_before = reserve_out as f64 / reserve_in as f64;
        let fee = (amount_in as f64 * self.fee_rate) as u64;
        let amount_in_after_fee = amount_in - fee;
        let amount_out = (amount_in_after_fee * reserve_out) / (reserve_in + amount_in_after_fee);
        
        let new_reserve_in = reserve_in + amount_in;
        let new_reserve_out = reserve_out - amount_out;
        let price_after = new_reserve_out as f64 / new_reserve_in as f64;
        
        let price_impact = ((price_after - price_before) / price_before).abs();
        Ok(price_impact)
    }

    /// Check if pool has sufficient liquidity for a trade
    pub fn has_sufficient_liquidity(&self, amount_out: u64, token_out: String) -> bool {
        if token_out == self.token_a {
            amount_out < self.reserve_a
        } else if token_out == self.token_b {
            amount_out < self.reserve_b
        } else {
            false
        }
    }

    /// Pause/unpause the pool
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Update fee rate (governance function)
    pub fn update_fee_rate(&mut self, new_fee_rate: f64) -> TribeResult<()> {
        if new_fee_rate < 0.0 || new_fee_rate > 0.1 {
            return Err(TribeError::InvalidOperation("Fee rate must be between 0 and 10%".to_string()));
        }
        
        self.fee_rate = new_fee_rate;
        self.protocol_fee_rate = new_fee_rate * 0.1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = LiquidityPool::new(
            "TRIBE".to_string(),
            "USDC".to_string(),
            10000,
            20000,
            "provider1".to_string(),
            0.003,
        ).unwrap();

        assert_eq!(pool.token_a, "TRIBE");
        assert_eq!(pool.token_b, "USDC");
        assert_eq!(pool.reserve_a, 10000);
        assert_eq!(pool.reserve_b, 20000);
        assert!(pool.is_active);
    }

    #[test]
    fn test_add_liquidity() {
        let mut pool = LiquidityPool::new(
            "TRIBE".to_string(),
            "USDC".to_string(),
            10000,
            20000,
            "provider1".to_string(),
            0.003,
        ).unwrap();

        let liquidity_tokens = pool.add_liquidity(
            "provider2".to_string(),
            5000,
            10000,
            0,
        ).unwrap();

        assert!(liquidity_tokens > 0);
        assert_eq!(pool.reserve_a, 15000);
        assert_eq!(pool.reserve_b, 30000);
    }

    #[test]
    fn test_swap() {
        let mut pool = LiquidityPool::new(
            "TRIBE".to_string(),
            "USDC".to_string(),
            10000,
            20000,
            "provider1".to_string(),
            0.003,
        ).unwrap();

        let amount_out = pool.swap(
            "trader1".to_string(),
            "TRIBE".to_string(),
            1000,
            0,
        ).unwrap();

        assert!(amount_out > 0);
        assert_eq!(pool.reserve_a, 11000);
        assert!(pool.reserve_b < 20000);
    }

    #[test]
    fn test_price_calculation() {
        let pool = LiquidityPool::new(
            "TRIBE".to_string(),
            "USDC".to_string(),
            10000,
            20000,
            "provider1".to_string(),
            0.003,
        ).unwrap();

        let price = pool.get_price();
        assert_eq!(price, 2.0); // 20000 / 10000
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pool = LiquidityPool::new(
            "TRIBE".to_string(),
            "USDC".to_string(),
            10000,
            20000,
            "provider1".to_string(),
            0.003,
        ).unwrap();

        let initial_liquidity = pool.liquidity_providers.get("provider1").unwrap().liquidity_tokens;
        let half_liquidity = initial_liquidity / 2;

        let (amount_a, amount_b) = pool.remove_liquidity(
            "provider1".to_string(),
            half_liquidity,
            0,
            0,
        ).unwrap();

        assert!(amount_a > 0);
        assert!(amount_b > 0);
        assert_eq!(pool.reserve_a, 10000 - amount_a);
        assert_eq!(pool.reserve_b, 20000 - amount_b);
    }
} 