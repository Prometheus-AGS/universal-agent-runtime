# Universal Agent Runtime Rust SDK 1.0

Typed async access to UAR chat, tools, structured outputs, embeddings, agent
runs, checkpoints, knowledge bases, documents, search, and ingestion.

```toml
[dependencies]
universal-agent-runtime-sdk = "1"
```

```rust,no_run
use universal_agent_runtime_sdk::{ChatCompletionRequest, ChatMessage, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("http://localhost:1906")?;
    let response = client.chat().complete(ChatCompletionRequest {
        messages: vec![ChatMessage::text("user", "Hello")],
        ..Default::default()
    }).await?;
    println!("{}", response.id);
    Ok(())
}
```

Set a runtime API key with `Client::with_api_key`. Run any sample with
`UAR_BASE_URL=http://localhost:1906 cargo run --example chat`.

The `embedded` feature links the AGPL runtime; HTTP-client-only consumers use
the MIT-licensed SDK. See [BREAKING.md](BREAKING.md) for migration notes.
