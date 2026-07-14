//! Inspect checkpoints for a run and resume it from the latest one.
//!
//! Requires a running UAR server (`UAR_BASE_URL`, default
//! `http://localhost:1906`) with an existing run id in `UAR_RUN_ID`.
use serde_json::json;
use universal_agent_runtime_sdk::{Client, ResumeRunRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(
        std::env::var("UAR_BASE_URL").unwrap_or_else(|_| "http://localhost:1906".into()),
    )?;
    let run_id = std::env::var("UAR_RUN_ID").unwrap_or_else(|_| "replace-me".into());

    let checkpoints = client.runs().checkpoints(&run_id).await?;
    println!("{checkpoints:#?}");

    let resumed = client
        .runs()
        .resume(
            &run_id,
            ResumeRunRequest {
                artifact: json!({"name": "example", "version": "1"}),
                input: None,
                session_id: None,
            },
        )
        .await?;
    println!("{resumed:#?}");
    Ok(())
}
