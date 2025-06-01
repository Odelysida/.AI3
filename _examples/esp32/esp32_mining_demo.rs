use tribechain::esp32_miner::{ESP32Miner, ESP32Config};
use tribechain_core::{TribeChain, Transaction, TransactionType};
use tribechain_mining::AI3Miner;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TribeChain ESP32 Mining Demo");
    println!("================================");

    // Create ESP32 configuration
    let esp32_config = ESP32Config {
        device_id: "ESP32_DEMO_001".to_string(),
        wifi_ssid: "TribeChain_Network".to_string(),
        wifi_password: "secure_password".to_string(),
        node_url: "http://localhost:8333".to_string(),
        mining_threads: 2,
        ai3_enabled: true,
        power_limit: 3.0,
    };

    println!("📱 ESP32 Configuration:");
    println!("   Device ID: {}", esp32_config.device_id);
    println!("   WiFi SSID: {}", esp32_config.wifi_ssid);
    println!("   Mining Threads: {}", esp32_config.mining_threads);
    println!("   AI3 Enabled: {}", esp32_config.ai3_enabled);
    println!("   Power Limit: {} watts", esp32_config.power_limit);
    println!();

    // Initialize ESP32 miner
    let mut esp32_miner = ESP32Miner::new(esp32_config);
    
    println!("🔧 Initializing ESP32 miner...");
    esp32_miner.initialize()?;
    println!("✅ ESP32 miner initialized successfully!");
    println!();

    // Create some demo transactions for the blockchain
    println!("💰 Creating demo transactions...");
    let demo_transactions = create_demo_transactions();
    println!("✅ Created {} demo transactions", demo_transactions.len());
    println!();

    // Add transactions to the blockchain
    if let Some(blockchain) = &mut esp32_miner.blockchain {
        for transaction in demo_transactions {
            blockchain.add_transaction(transaction)?;
        }
        println!("✅ Added transactions to blockchain");
    }
    println!();

    // Create and add some AI3 tensor tasks
    println!("🧠 Creating AI3 tensor tasks...");
    if let Some(blockchain) = &mut esp32_miner.blockchain {
        let ai3_miner = AI3Miner::new("ai3_demo".to_string(), "ai3_address".to_string());
        
        for i in 0..3 {
            let task = ai3_miner.generate_random_task();
            blockchain.add_tensor_task(task.clone())?;
            println!("   📋 Added tensor task {}: {}", i + 1, task.operation);
        }
    }
    println!("✅ Added AI3 tensor tasks to blockchain");
    println!();

    // Demonstrate mining a few blocks
    println!("⛏️  Starting mining demonstration...");
    for round in 1..=3 {
        println!("🔄 Mining round {}/3", round);
        
        match esp32_miner.mine_block()? {
            Some(result) => {
                println!("   ✅ Block mined successfully!");
                println!("   📦 Block hash: {}", result.hash);
                println!("   🔢 Nonce: {}", result.nonce);
                println!("   ⏱️  Mining time: {} seconds", result.mining_time);
                
                if let Some(ai3_proof) = &result.ai3_proof {
                    println!("   🧠 AI3 Proof included!");
                    println!("   📋 Task ID: {}", ai3_proof.task_id);
                    println!("   🔗 Computation hash: {}", ai3_proof.computation_hash);
                }
                
                // Submit the block
                esp32_miner.submit_block(result)?;
                println!("   📡 Block submitted to network");
            }
            None => {
                println!("   ⏸️  No block mined this round");
            }
        }
        
        // Show ESP32 statistics
        let stats = esp32_miner.get_stats();
        println!("   📊 ESP32 Stats:");
        println!("      🌡️  Temperature: {:.1}°C", stats.temperature);
        println!("      ⚡ Power: {:.1}W", stats.power_consumption);
        println!("      💾 Memory: {}KB", stats.memory_usage);
        println!("      📶 WiFi Signal: {}dBm", stats.wifi_signal);
        println!();
        
        // Small delay between rounds
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // Show final blockchain statistics
    if let Some(blockchain) = &esp32_miner.blockchain {
        let stats = blockchain.get_stats();
        println!("📈 Final Blockchain Statistics:");
        println!("   🧱 Total Blocks: {}", stats.block_count);
        println!("   💸 Total Transactions: {}", stats.transaction_count);
        println!("   ⏳ Pending Transactions: {}", stats.pending_transactions);
        println!("   🎯 Current Difficulty: {}", stats.difficulty);
        println!("   💰 Mining Reward: {}", stats.mining_reward);
        println!("   👥 Active Miners: {}", stats.active_miners);
        println!("   🧠 Tensor Tasks: {}", stats.tensor_tasks);
        println!("   🔢 AI3 Difficulty Multiplier: {}", stats.ai3_difficulty_multiplier);
    }
    println!();

    println!("🎉 ESP32 Mining Demo completed successfully!");
    println!("   The ESP32 has successfully:");
    println!("   ✅ Connected to WiFi");
    println!("   ✅ Synchronized with TribeChain");
    println!("   ✅ Mined blocks with traditional PoW");
    println!("   ✅ Processed AI3 tensor computations");
    println!("   ✅ Managed power consumption");
    println!("   ✅ Monitored system health");

    Ok(())
}

fn create_demo_transactions() -> Vec<Transaction> {
    let mut transactions = Vec::new();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create a simple transfer transaction
    let transfer_tx = Transaction::new(
        "alice".to_string(),
        TransactionType::Transfer {
            to: "bob".to_string(),
            amount: 100.0,
        },
        1.0, // fee
        current_time,
        1, // nonce
    );
    transactions.push(transfer_tx);

    // Create a token creation transaction
    let token_tx = Transaction::new(
        "creator".to_string(),
        TransactionType::TokenCreate {
            name: "DemoToken".to_string(),
            symbol: "DEMO".to_string(),
            total_supply: 1000000.0,
            decimals: 18,
        },
        5.0, // fee
        current_time + 1,
        1, // nonce
    );
    transactions.push(token_tx);

    // Create a tensor computation transaction
    let tensor_tx = Transaction::new(
        "ai_user".to_string(),
        TransactionType::TensorCompute {
            operation: "matrix_multiply".to_string(),
            input_data: vec![1.0, 2.0, 3.0, 4.0],
            max_compute_time: 5000,
            reward: 10.0,
        },
        2.0, // fee
        current_time + 2,
        1, // nonce
    );
    transactions.push(tensor_tx);

    // Create a staking transaction
    let stake_tx = Transaction::new(
        "staker".to_string(),
        TransactionType::Stake {
            amount: 1000.0,
            duration: 86400, // 1 day
        },
        1.5, // fee
        current_time + 3,
        1, // nonce
    );
    transactions.push(stake_tx);

    transactions
} 