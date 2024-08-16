use clap::{App, Arg};
use log::{error, info};
use omnitensor_core::{
    config::Config,
    consensus::ConsensusEngine,
    network::NetworkManager,
    node::Node,
    storage::Storage,
};
use std::process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    env_logger::init();

    // Parse command line arguments
    let matches = App::new("OmniTensor Core")
        .version("0.1.0")
        .author("OmniTensor Team")
        .about("Decentralized AI Infrastructure")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    // Load configuration
    let config_path = matches.value_of("config").unwrap_or("config/default.toml");
    let config = match Config::from_file(config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    info!("Starting OmniTensor Core node...");

    // Initialize components
    let storage = Storage::new(&config.storage_path)?;
    let network_manager = NetworkManager::new(&config.network)?;
    let consensus_engine = ConsensusEngine::new(&config.consensus, &storage)?;

    // Create and start the node
    let mut node = Node::new(storage, network_manager, consensus_engine);
    
    // Start the main event loop
    if let Err(e) = node.run().await {
        error!("Node failed: {}", e);
        process::exit(1);
    }

    info!("OmniTensor Core node shutting down.");
    Ok(())
}