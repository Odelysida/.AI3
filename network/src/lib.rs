pub mod peer;
pub mod protocol;
pub mod node;
pub mod consensus;
pub mod p2p;
pub mod rpc;
pub mod sync;

pub use peer::*;
pub use protocol::*;
pub use node::*;
pub use consensus::*;
pub use p2p::*;
pub use rpc::*;
pub use sync::*;

use tribechain_core::{TribeResult, TribeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Network manager for TribeChain
#[derive(Debug)]
pub struct NetworkManager {
    pub node: node::Node,
    pub consensus: consensus::ConsensusEngine,
    pub p2p: p2p::P2PNetwork,
    pub rpc: rpc::RpcServer,
    pub sync: sync::SyncManager,
    pub is_running: bool,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub node_id: String,
    pub listen_address: String,
    pub port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
    pub consensus_type: ConsensusType,
    pub mining_enabled: bool,
    pub rpc_enabled: bool,
    pub rpc_port: u16,
}

/// Consensus types supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusType {
    ProofOfWork,
    ProofOfStake,
    DelegatedProofOfStake,
    TensorProofOfWork, // AI3-specific consensus
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub total_transactions: u64,
    pub blocks_processed: u64,
    pub network_hash_rate: f64,
    pub uptime: chrono::Duration,
    pub sync_status: SyncStatus,
}

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Syncing { current_block: u64, target_block: u64 },
    NotSynced,
    Error(String),
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> TribeResult<Self> {
        let node = node::Node::new(config.clone())?;
        let consensus = consensus::ConsensusEngine::new(config.consensus_type.clone())?;
        let p2p = p2p::P2PNetwork::new(config.clone())?;
        let rpc = rpc::RpcServer::new(config.rpc_port)?;
        let sync = sync::SyncManager::new()?;

        Ok(Self {
            node,
            consensus,
            p2p,
            rpc,
            sync,
            is_running: false,
        })
    }

    /// Start the network
    pub async fn start(&mut self) -> TribeResult<()> {
        if self.is_running {
            return Err(TribeError::InvalidOperation("Network is already running".to_string()));
        }

        // Start P2P network
        self.p2p.start().await?;
        
        // Start consensus engine
        self.consensus.start().await?;
        
        // Start RPC server if enabled
        if self.node.config.rpc_enabled {
            self.rpc.start().await?;
        }
        
        // Start sync manager
        self.sync.start().await?;
        
        // Start the node
        self.node.start().await?;

        self.is_running = true;
        Ok(())
    }

    /// Stop the network
    pub async fn stop(&mut self) -> TribeResult<()> {
        if !self.is_running {
            return Ok(());
        }

        // Stop components in reverse order
        self.node.stop().await?;
        self.sync.stop().await?;
        self.rpc.stop().await?;
        self.consensus.stop().await?;
        self.p2p.stop().await?;

        self.is_running = false;
        Ok(())
    }

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.p2p.get_peer_count(),
            total_transactions: self.node.get_transaction_count(),
            blocks_processed: self.node.get_block_count(),
            network_hash_rate: self.consensus.get_hash_rate(),
            uptime: self.node.get_uptime(),
            sync_status: self.sync.get_status(),
        }
    }

    /// Broadcast a transaction to the network
    pub async fn broadcast_transaction(&mut self, transaction: tribechain_core::Transaction) -> TribeResult<()> {
        // Validate transaction
        transaction.validate()?;
        
        // Add to local mempool
        self.node.add_transaction(transaction.clone())?;
        
        // Broadcast to peers
        self.p2p.broadcast_transaction(transaction).await?;
        
        Ok(())
    }

    /// Broadcast a block to the network
    pub async fn broadcast_block(&mut self, block: tribechain_core::Block) -> TribeResult<()> {
        // Validate block
        block.validate()?;
        
        // Add to local blockchain
        self.node.add_block(block.clone())?;
        
        // Broadcast to peers
        self.p2p.broadcast_block(block).await?;
        
        Ok(())
    }

    /// Connect to a peer
    pub async fn connect_peer(&mut self, address: String) -> TribeResult<()> {
        self.p2p.connect_peer(address).await
    }

    /// Disconnect from a peer
    pub async fn disconnect_peer(&mut self, peer_id: String) -> TribeResult<()> {
        self.p2p.disconnect_peer(peer_id).await
    }

    /// Get connected peers
    pub fn get_peers(&self) -> Vec<p2p::PeerInfo> {
        self.p2p.get_peers()
    }

    /// Process incoming message
    pub async fn handle_message(&mut self, message: p2p::NetworkMessage) -> TribeResult<()> {
        match message.message_type {
            p2p::MessageType::Transaction => {
                let transaction: tribechain_core::Transaction = serde_json::from_slice(&message.data)?;
                self.node.add_transaction(transaction)?;
            }
            p2p::MessageType::Block => {
                let block: tribechain_core::Block = serde_json::from_slice(&message.data)?;
                self.node.add_block(block)?;
            }
            p2p::MessageType::Ping => {
                // Respond with pong
                let pong = p2p::NetworkMessage::new_pong(self.node.config.node_id.clone());
                self.p2p.send_message(message.sender, pong).await?;
            }
            p2p::MessageType::Pong => {
                // Update peer last seen
                self.p2p.update_peer_activity(message.sender)?;
            }
            p2p::MessageType::SyncRequest => {
                // Handle sync request
                self.sync.handle_sync_request(message).await?;
            }
            p2p::MessageType::SyncResponse => {
                // Handle sync response
                self.sync.handle_sync_response(message).await?;
            }
        }
        Ok(())
    }

    /// Mine a new block (if mining is enabled)
    pub async fn mine_block(&mut self) -> TribeResult<Option<tribechain_core::Block>> {
        if !self.node.config.mining_enabled {
            return Ok(None);
        }

        // Get pending transactions
        let transactions = self.node.get_pending_transactions()?;
        
        if transactions.is_empty() {
            return Ok(None);
        }

        // Mine block using consensus engine
        let block = self.consensus.mine_block(transactions).await?;
        
        // Add to blockchain and broadcast
        self.node.add_block(block.clone())?;
        self.p2p.broadcast_block(block.clone()).await?;
        
        Ok(Some(block))
    }

    /// Sync with the network
    pub async fn sync(&mut self) -> TribeResult<()> {
        self.sync.start_sync(&mut self.p2p).await
    }

    /// Get blockchain info
    pub fn get_blockchain_info(&self) -> node::BlockchainInfo {
        self.node.get_blockchain_info()
    }

    /// Get mempool info
    pub fn get_mempool_info(&self) -> node::MempoolInfo {
        self.node.get_mempool_info()
    }

    /// Validate the entire blockchain
    pub fn validate_blockchain(&self) -> TribeResult<bool> {
        self.node.validate_blockchain()
    }

    /// Get block by hash
    pub fn get_block(&self, hash: String) -> Option<tribechain_core::Block> {
        self.node.get_block(hash)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: String) -> Option<tribechain_core::Transaction> {
        self.node.get_transaction(hash)
    }

    /// Get account balance
    pub fn get_balance(&self, address: String) -> u64 {
        self.node.get_balance(address)
    }

    /// Create and send a transaction
    pub async fn send_transaction(
        &mut self,
        from: String,
        to: String,
        amount: u64,
        private_key: String,
    ) -> TribeResult<String> {
        // Create transaction
        let transaction = self.node.create_transaction(from, to, amount, private_key)?;
        
        // Broadcast transaction
        self.broadcast_transaction(transaction.clone()).await?;
        
        Ok(transaction.hash)
    }

    /// Deploy a smart contract
    pub async fn deploy_contract(
        &mut self,
        deployer: String,
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        private_key: String,
    ) -> TribeResult<String> {
        let transaction = self.node.create_contract_deployment(
            deployer,
            code,
            constructor_args,
            private_key,
        )?;
        
        self.broadcast_transaction(transaction.clone()).await?;
        
        Ok(transaction.hash)
    }

    /// Call a smart contract
    pub async fn call_contract(
        &mut self,
        caller: String,
        contract_address: String,
        method: String,
        args: Vec<u8>,
        private_key: String,
    ) -> TribeResult<String> {
        let transaction = self.node.create_contract_call(
            caller,
            contract_address,
            method,
            args,
            private_key,
        )?;
        
        self.broadcast_transaction(transaction.clone()).await?;
        
        Ok(transaction.hash)
    }

    /// Get network health status
    pub fn get_health(&self) -> NetworkHealth {
        NetworkHealth {
            is_healthy: self.is_running && self.p2p.get_peer_count() > 0,
            connected_peers: self.p2p.get_peer_count(),
            sync_status: self.sync.get_status(),
            last_block_time: self.node.get_last_block_time(),
            mempool_size: self.node.get_mempool_size(),
        }
    }
}

/// Network health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub is_healthy: bool,
    pub connected_peers: usize,
    pub sync_status: SyncStatus,
    pub last_block_time: Option<DateTime<Utc>>,
    pub mempool_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            listen_address: "0.0.0.0".to_string(),
            port: 8333,
            bootstrap_nodes: vec![],
            max_peers: 50,
            consensus_type: ConsensusType::ProofOfWork,
            mining_enabled: false,
            rpc_enabled: true,
            rpc_port: 8334,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let network = NetworkManager::new(config);
        assert!(network.is_ok());
    }

    #[tokio::test]
    async fn test_network_start_stop() {
        let config = NetworkConfig::default();
        let mut network = NetworkManager::new(config).unwrap();
        
        // Start network
        assert!(network.start().await.is_ok());
        assert!(network.is_running);
        
        // Stop network
        assert!(network.stop().await.is_ok());
        assert!(!network.is_running);
    }

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.port, 8333);
        assert_eq!(config.rpc_port, 8334);
        assert_eq!(config.max_peers, 50);
        assert!(config.rpc_enabled);
        assert!(!config.mining_enabled);
    }

    #[test]
    fn test_consensus_types() {
        let pow = ConsensusType::ProofOfWork;
        let pos = ConsensusType::ProofOfStake;
        let dpos = ConsensusType::DelegatedProofOfStake;
        let tpow = ConsensusType::TensorProofOfWork;
        
        // Test serialization
        let pow_json = serde_json::to_string(&pow).unwrap();
        let pos_json = serde_json::to_string(&pos).unwrap();
        let dpos_json = serde_json::to_string(&dpos).unwrap();
        let tpow_json = serde_json::to_string(&tpow).unwrap();
        
        assert!(pow_json.contains("ProofOfWork"));
        assert!(pos_json.contains("ProofOfStake"));
        assert!(dpos_json.contains("DelegatedProofOfStake"));
        assert!(tpow_json.contains("TensorProofOfWork"));
    }
} 