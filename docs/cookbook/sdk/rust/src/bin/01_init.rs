//! Cookbook SDK (Rust): initialize a UAR client.

use universal_agent_runtime_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into());
    let client = Client::new(&base_url)?;
    println!("UAR client initialized for {}", client.base_url());
    Ok(())
}
