use clap::{Arg, Command};
use std::net::SocketAddr;
use tokio;
use tribechain::{
    TribeChain, NetworkNode, Transaction, TransactionType, TensorTask, MinerInfo,
    AI3Engine, TokenManager, TokenInfo, TokenType, TribeResult, TribeError
};
use std::process;

mod esp32_miner;
use esp32_miner::{ESP32Miner, ESP32Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("TribeChain")
        .version("1.0.0")
        .author("BitTribe")
        .about("TribeChain - AI-Powered Blockchain with Tensor Mining")
        .subcommand(
            Command::new("node")
                .about("Start a TribeChain node")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .value_name("PORT")
                        .help("Port to listen on")
                        .default_value("8333")
                )
                .arg(
                    Arg::new("data-dir")
                        .short('d')
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Data directory for blockchain storage")
                        .default_value("./data")
                )
                .arg(
                    Arg::new("connect")
                        .short('c')
                        .long("connect")
                        .value_name("ADDRESS")
                        .help("Connect to peer (format: ip:port)")
                        .action(clap::ArgAction::Append)
                )
        )
        .subcommand(
            Command::new("wallet")
                .about("Wallet operations")
                .subcommand(
                    Command::new("balance")
                        .about("Check balance")
                        .arg(
                            Arg::new("address")
                                .help("Address to check")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("send")
                        .about("Send tokens")
                        .arg(
                            Arg::new("from")
                                .help("Sender address")
                                .required(true)
                        )
                        .arg(
                            Arg::new("to")
                                .help("Recipient address")
                                .required(true)
                        )
                        .arg(
                            Arg::new("amount")
                                .help("Amount to send")
                                .required(true)
                        )
                )
        )
        .subcommand(
            Command::new("mine")
                .about("Mining operations")
                .arg(
                    Arg::new("address")
                        .help("Miner address")
                        .required(true)
                )
                .arg(
                    Arg::new("data-dir")
                        .short('d')
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Data directory for blockchain storage")
                        .default_value("./data")
                )
        )
        .subcommand(
            Command::new("stats")
                .about("Show blockchain statistics")
                .arg(
                    Arg::new("data-dir")
                        .short('d')
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Data directory for blockchain storage")
                        .default_value("./data")
                )
        )
        .subcommand(
            Command::new("token")
                .about("Token operations")
                .subcommand(
                    Command::new("create")
                        .about("Create a new token")
                        .arg(Arg::new("name").help("Token name").required(true))
                        .arg(Arg::new("symbol").help("Token symbol").required(true))
                        .arg(Arg::new("supply").help("Initial supply").required(true))
                        .arg(Arg::new("creator").help("Creator address").required(true))
                )
        )
        .subcommand(
            Command::new("ai3")
                .about("AI3 tensor operations")
                .subcommand(
                    Command::new("compute")
                        .about("Submit tensor computation task")
                        .arg(Arg::new("operation").help("Operation type").required(true))
                        .arg(Arg::new("data").help("Input data (comma-separated)").required(true))
                        .arg(Arg::new("requester").help("Requester address").required(true))
                )
        )
        .subcommand(
            Command::new("esp32")
                .about("ESP32 mining operations")
                .subcommand(
                    Command::new("mine")
                        .about("Start ESP32 mining")
                        .arg(Arg::new("device-id")
                            .short('d')
                            .long("device-id")
                            .value_name("ID")
                            .help("ESP32 device ID")
                            .required(true))
                        .arg(Arg::new("wifi-ssid")
                            .short('s')
                            .long("wifi-ssid")
                            .value_name("SSID")
                            .help("WiFi network SSID")
                            .required(true))
                        .arg(Arg::new("wifi-password")
                            .short('w')
                            .long("wifi-password")
                            .value_name("PASSWORD")
                            .help("WiFi network password")
                            .required(true))
                        .arg(Arg::new("node-url")
                            .short('n')
                            .long("node-url")
                            .value_name("URL")
                            .help("TribeChain node URL")
                            .default_value("http://localhost:8333"))
                        .arg(Arg::new("threads")
                            .short('t')
                            .long("threads")
                            .value_name("THREADS")
                            .help("Number of mining threads")
                            .default_value("2"))
                        .arg(Arg::new("ai3")
                            .long("ai3")
                            .help("Enable AI3 tensor mining")
                            .action(clap::ArgAction::SetTrue))
                        .arg(Arg::new("power-limit")
                            .short('p')
                            .long("power-limit")
                            .value_name("WATTS")
                            .help("Power consumption limit in watts")
                            .default_value("3.0"))
                )
                .subcommand(
                    Command::new("stats")
                        .about("Show ESP32 mining statistics")
                        .arg(Arg::new("device-id")
                            .short('d')
                            .long("device-id")
                            .value_name("ID")
                            .help("ESP32 device ID")
                            .required(true))
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("node", sub_matches)) => {
            start_node(sub_matches).await?;
        }
        Some(("wallet", sub_matches)) => {
            handle_wallet_commands(sub_matches).await?;
        }
        Some(("mine", sub_matches)) => {
            start_mining(sub_matches).await?;
        }
        Some(("stats", sub_matches)) => {
            show_stats(sub_matches).await?;
        }
        Some(("token", sub_matches)) => {
            handle_token_commands(sub_matches).await?;
        }
        Some(("ai3", sub_matches)) => {
            handle_ai3_commands(sub_matches).await?;
        }
        Some(("esp32", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("mine", esp32_matches)) => {
                    let config = ESP32Config {
                        device_id: esp32_matches.get_one::<String>("device-id").unwrap().clone(),
                        wifi_ssid: esp32_matches.get_one::<String>("wifi-ssid").unwrap().clone(),
                        wifi_password: esp32_matches.get_one::<String>("wifi-password").unwrap().clone(),
                        node_url: esp32_matches.get_one::<String>("node-url").unwrap().clone(),
                        mining_threads: esp32_matches.get_one::<String>("threads").unwrap().parse().unwrap_or(2),
                        ai3_enabled: esp32_matches.get_flag("ai3"),
                        power_limit: esp32_matches.get_one::<String>("power-limit").unwrap().parse().unwrap_or(3.0),
                    };
                    
                    if let Err(e) = start_esp32_mining(config).await {
                        eprintln!("ESP32 mining failed: {}", e);
                        process::exit(1);
                    }
                }
                Some(("stats", esp32_matches)) => {
                    let device_id = esp32_matches.get_one::<String>("device-id").unwrap();
                    println!("ESP32 Mining Statistics for device: {}", device_id);
                    // Stats implementation would go here
                }
                _ => println!("Invalid ESP32 command"),
            }
        }
        _ => {
            println!("TribeChain v1.0.0 - AI-Powered Blockchain");
            println!("Use --help for available commands");
        }
    }

    Ok(())
}

async fn start_node(matches: &clap::ArgMatches) -> TribeResult<()> {
    let port: u16 = matches.get_one::<String>("port")
        .unwrap()
        .parse()
        .map_err(|_| TribeError::Generic("Invalid port number".to_string()))?;
    
    let data_dir = matches.get_one::<String>("data-dir").unwrap();
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", port).parse()
        .map_err(|_| TribeError::Network("Invalid listen address".to_string()))?;

    println!("Starting TribeChain node...");
    println!("Data directory: {}", data_dir);
    println!("Listening on: {}", listen_addr);

    // Initialize blockchain
    let blockchain = TribeChain::new(data_dir)?;
    let node_id = format!("node_{}", port);
    
    // Create network node
    let network_node = NetworkNode::new(node_id, listen_addr, blockchain);

    // Connect to peers if specified
    if let Some(peers) = matches.get_many::<String>("connect") {
        for peer_addr in peers {
            let addr: SocketAddr = peer_addr.parse()
                .map_err(|_| TribeError::Network(format!("Invalid peer address: {}", peer_addr)))?;
            
            println!("Connecting to peer: {}", addr);
            if let Err(e) = network_node.connect_to_peer(addr).await {
                eprintln!("Failed to connect to peer {}: {}", addr, e);
            }
        }
    }

    // Start the node
    network_node.start().await?;

    Ok(())
}

async fn handle_wallet_commands(matches: &clap::ArgMatches) -> TribeResult<()> {
    match matches.subcommand() {
        Some(("balance", sub_matches)) => {
            let address = sub_matches.get_one::<String>("address").unwrap();
            let blockchain = TribeChain::new("./data")?;
            let balance = blockchain.get_balance(address);
            println!("Balance for {}: {} TRIBE", address, balance as f64 / 1_000_000.0);
        }
        Some(("send", sub_matches)) => {
            let from = sub_matches.get_one::<String>("from").unwrap();
            let to = sub_matches.get_one::<String>("to").unwrap();
            let amount: u64 = sub_matches.get_one::<String>("amount")
                .unwrap()
                .parse::<f64>()
                .map_err(|_| TribeError::Generic("Invalid amount".to_string()))?
                as u64 * 1_000_000; // Convert to smallest unit

            let mut blockchain = TribeChain::new("./data")?;
            
            let transaction = Transaction::new(
                from.clone(),
                TransactionType::Transfer {
                    to: to.clone(),
                    amount,
                }
            );

            blockchain.add_transaction(transaction)?;
            println!("Transaction added to pending pool");
            println!("From: {}", from);
            println!("To: {}", to);
            println!("Amount: {} TRIBE", amount as f64 / 1_000_000.0);
        }
        _ => {
            println!("Available wallet commands: balance, send");
        }
    }

    Ok(())
}

async fn start_mining(matches: &clap::ArgMatches) -> TribeResult<()> {
    let miner_address = matches.get_one::<String>("address").unwrap();
    let data_dir = matches.get_one::<String>("data-dir").unwrap();

    println!("Starting mining for address: {}", miner_address);
    
    let mut blockchain = TribeChain::new(data_dir)?;
    
    // Register miner
    let miner_info = MinerInfo {
        id: miner_address.clone(),
        device_type: "CPU".to_string(),
        compute_power: 1000,
        last_seen: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    blockchain.register_miner(miner_info)?;

    loop {
        if !blockchain.pending_transactions.is_empty() {
            println!("Mining new block...");
            match blockchain.mine_block(miner_address.clone()) {
                Ok(block) => {
                    println!("Successfully mined block #{}", block.index);
                    println!("Block hash: {}", block.hash);
                    println!("Transactions: {}", block.transactions.len());
                    println!("Mining reward: {} TRIBE", blockchain.mining_reward as f64 / 1_000_000.0);
                }
                Err(e) => {
                    eprintln!("Mining failed: {}", e);
                }
            }
        } else {
            println!("No pending transactions. Waiting...");
        }

        // Check for tensor tasks
        let pending_tasks = blockchain.get_pending_tensor_tasks();
        if !pending_tasks.is_empty() {
            println!("Processing {} tensor tasks...", pending_tasks.len());
            
            let mut ai3_engine = AI3Engine::new();
            for task in pending_tasks {
                match ai3_engine.execute_tensor_operation(&task.operation, &task.input_data) {
                    Ok(result) => {
                        blockchain.complete_tensor_task(&task.id, result)?;
                        println!("Completed tensor task: {}", task.id);
                    }
                    Err(e) => {
                        eprintln!("Failed to execute tensor task {}: {}", task.id, e);
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

async fn show_stats(matches: &clap::ArgMatches) -> TribeResult<()> {
    let data_dir = matches.get_one::<String>("data-dir").unwrap();
    let blockchain = TribeChain::new(data_dir)?;
    let stats = blockchain.get_stats();

    println!("=== TribeChain Statistics ===");
    println!("Blocks: {}", stats.block_count);
    println!("Transactions: {}", stats.transaction_count);
    println!("Pending Transactions: {}", stats.pending_transactions);
    println!("Difficulty: {}", stats.difficulty);
    println!("Mining Reward: {} TRIBE", stats.mining_reward as f64 / 1_000_000.0);
    println!("Total Supply: {} TRIBE", stats.total_supply as f64 / 1_000_000.0);
    println!("Active Addresses: {}", stats.active_addresses);
    println!("Average Block Time: {} seconds", stats.avg_block_time);
    println!("Active Miners: {}", stats.active_miners);
    println!("Tensor Tasks: {}", stats.tensor_tasks);

    Ok(())
}

async fn handle_token_commands(matches: &clap::ArgMatches) -> TribeResult<()> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let symbol = sub_matches.get_one::<String>("symbol").unwrap();
            let supply: u64 = sub_matches.get_one::<String>("supply")
                .unwrap()
                .parse()
                .map_err(|_| TribeError::Generic("Invalid supply".to_string()))?;
            let creator = sub_matches.get_one::<String>("creator").unwrap();

            let mut blockchain = TribeChain::new("./data")?;
            
            let token_info = TokenInfo {
                token_type: if symbol == "STOMP" { TokenType::STOMP } else { TokenType::AUM },
                name: name.clone(),
                symbol: symbol.clone(),
                decimals: 6,
                total_supply: supply * 1_000_000, // Convert to smallest unit
                mintable: true,
                burnable: true,
                stakeable: true,
            };

            let transaction = Transaction::new(
                creator.clone(),
                TransactionType::TokenCreate { token_info }
            );

            blockchain.add_transaction(transaction)?;
            println!("Token creation transaction added to pending pool");
            println!("Name: {}", name);
            println!("Symbol: {}", symbol);
            println!("Supply: {}", supply);
            println!("Creator: {}", creator);
        }
        _ => {
            println!("Available token commands: create");
        }
    }

    Ok(())
}

async fn handle_ai3_commands(matches: &clap::ArgMatches) -> TribeResult<()> {
    match matches.subcommand() {
        Some(("compute", sub_matches)) => {
            let operation = sub_matches.get_one::<String>("operation").unwrap();
            let data_str = sub_matches.get_one::<String>("data").unwrap();
            let requester = sub_matches.get_one::<String>("requester").unwrap();

            // Parse input data
            let input_data: Vec<f32> = data_str
                .split(',')
                .map(|s| s.trim().parse::<f32>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| TribeError::Generic("Invalid input data format".to_string()))?;

            let mut blockchain = TribeChain::new("./data")?;
            
            let task = TensorTask {
                id: format!("task_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()),
                operation: operation.clone(),
                input_data,
                requester: requester.clone(),
                reward: 100_000, // 0.1 TRIBE token
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                completed: false,
                result: None,
            };

            // Add tensor computation transaction
            let transaction = Transaction::new(
                requester.clone(),
                TransactionType::TensorCompute {
                    task_id: task.id.clone(),
                    operation: operation.clone(),
                    input_data: task.input_data.clone(),
                }
            );

            blockchain.add_tensor_task(task.clone())?;
            blockchain.add_transaction(transaction)?;

            println!("Tensor computation task submitted");
            println!("Task ID: {}", task.id);
            println!("Operation: {}", operation);
            println!("Requester: {}", requester);
            println!("Reward: {} TRIBE", task.reward as f64 / 1_000_000.0);
        }
        _ => {
            println!("Available AI3 commands: compute");
        }
    }

    Ok(())
}

async fn start_esp32_mining(config: ESP32Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting ESP32 mining with configuration:");
    println!("  Device ID: {}", config.device_id);
    println!("  WiFi SSID: {}", config.wifi_ssid);
    println!("  Node URL: {}", config.node_url);
    println!("  Mining Threads: {}", config.mining_threads);
    println!("  AI3 Enabled: {}", config.ai3_enabled);
    println!("  Power Limit: {} watts", config.power_limit);
    
    let mut esp32_miner = ESP32Miner::new(config);
    
    // Initialize the ESP32 miner
    esp32_miner.initialize()?;
    
    // Start the mining loop
    esp32_miner.run_mining_loop()?;
    
    Ok(())
} 