//! Demonstrate the SDK's `miette`-backed diagnostic error model.
//!
//! Self-contained: deliberately targets an unreachable port so the
//! request fails locally, then prints the resulting [`miette::Diagnostic`]
//! (code + help text) instead of a bare error message. Exits `0` on the
//! expected failure, so it can run as a CI smoke test without a live
//! UAR server.
use miette::Diagnostic;
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Port 1 is reserved and never accepts connections, so this always fails
    // fast with a transport error - no network access required.
    let client = Client::new("http://127.0.0.1:1")?;
    let result = client
        .chat()
        .complete(ChatCompletionRequest {
            messages: vec![ChatMessage::text("user", "Hello")],
            ..Default::default()
        })
        .await;

    match result {
        Ok(response) => println!("unexpectedly succeeded: {response:#?}"),
        Err(error) => {
            println!("error: {error}");
            if let Some(code) = error.code() {
                println!("code: {code}");
            }
            if let Some(help) = error.help() {
                println!("help: {help}");
            }
        }
    }
    Ok(())
}
