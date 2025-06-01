use clap::{Arg, Command};
use tribechain::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("ai3-miner")
        .version(VERSION)
        .about("TribeChain AI3 Tensor Mining Client")
        .arg(
            Arg::new("node-url")
                .long("node-url")
                .value_name("URL")
                .help("TribeChain node URL")
                .default_value("http://localhost:8334")
        )
        .arg(
            Arg::new("miner-id")
                .long("miner-id")
                .value_name("ID")
                .help("Unique miner identifier")
                .required(true)
        )
        .arg(
            Arg::new("difficulty")
                .long("difficulty")
                .value_name("DIFFICULTY")
                .help("Mining difficulty")
                .default_value("1000")
        )
        .arg(
            Arg::new("threads")
                .long("threads")
                .value_name("THREADS")
                .help("Number of mining threads")
                .default_value("1")
        )
        .get_matches();

    let node_url = matches.get_one::<String>("node-url").unwrap();
    let miner_id = matches.get_one::<String>("miner-id").unwrap();
    let difficulty: u32 = matches.get_one::<String>("difficulty").unwrap().parse()?;
    let threads: usize = matches.get_one::<String>("threads").unwrap().parse()?;

    println!("🚀 Starting AI3 Miner");
    println!("   Miner ID: {}", miner_id);
    println!("   Node URL: {}", node_url);
    println!("   Difficulty: {}", difficulty);
    println!("   Threads: {}", threads);

    // Create AI3 miner
    let miner = AI3Miner::new(miner_id.clone(), difficulty);
    
    // Start mining tasks
    let mut handles = Vec::new();
    
    for i in 0..threads {
        let miner_clone = miner.clone();
        let miner_id_clone = format!("{}-{}", miner_id, i);
        
        let handle = tokio::spawn(async move {
            loop {
                // Generate a random tensor task for demonstration
                match TensorTask::generate_random(difficulty, 1000) {
                    Ok(task) => {
                        println!("🔍 Thread {}: Mining task {}", i, task.id);
                        
                        match miner_clone.mine_proof(&task) {
                            Ok(Some(proof)) => {
                                println!("✅ Thread {}: Found proof! Optimization factor: {:.4}", 
                                        i, proof.optimization_factor);
                                // In a real implementation, submit proof to network
                            }
                            Ok(None) => {
                                println!("❌ Thread {}: No proof found", i);
                            }
                            Err(e) => {
                                eprintln!("💥 Thread {}: Mining error: {}", i, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("💥 Thread {}: Task generation error: {}", i, e);
                    }
                }
                
                // Small delay between mining attempts
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        
        handles.push(handle);
    }

    // Wait for all mining threads
    for handle in handles {
        handle.await?;
    }

    Ok(())
} 