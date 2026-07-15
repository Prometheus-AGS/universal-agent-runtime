//! Cookbook: dry-run the UAR server startup path.
//!
//! This example loads the configuration the same way the server binary does and
//! prints the address it would listen on. It intentionally does not start the
//! full HTTP server so it can run in a bare CI environment.

use clap::Parser as _;
use universal_agent_runtime::config::Cli;
use universal_agent_runtime::config_manager::ConfigManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config_manager = ConfigManager::load_without_watcher(cli).await?;
    let config = config_manager.current().server.clone();

    println!("UAR server would start on {}:{}", config.host, config.port);
    println!("Log format: {:?}", config.log_format);
    println!("Config loaded successfully (dry-run).");
    Ok(())
}
