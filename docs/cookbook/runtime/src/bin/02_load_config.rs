//! Cookbook: load a UAR configuration file and inspect a value.

use clap::Parser as _;
use std::path::PathBuf;
use universal_agent_runtime::config::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::var("UAR_CONFIG_FILE").unwrap_or_else(|_| {
        if PathBuf::from("config.yaml").exists() {
            "config.yaml".into()
        } else {
            "example.config.yaml".into()
        }
    }));
    let cli = universal_agent_runtime::config::Cli::parse_from([
        "cookbook",
        "--config",
        &path.to_string_lossy(),
    ]);
    let config = AppConfig::load_with_cli(cli)?;

    println!("Loaded config from: {}", path.display());
    println!("Server host: {}", config.server.host);
    println!("Server port: {}", config.server.port);
    println!("LLM model: {}", config.llm.model);
    Ok(())
}
