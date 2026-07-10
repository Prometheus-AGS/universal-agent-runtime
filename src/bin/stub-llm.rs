//! Standalone boot target for the `tests/integration/live/stub_llm.rs`
//! fixture server, so Playwright's `webServer` (browser-driven BDD suite,
//! `tests/bdd/`) can boot a deterministic, keyless LLM backend the same way
//! `tests/bdd.rs`'s in-process Cucumber suite already does.
//!
//! Reads a fixture file (JSON array of `{model, last_user_message, has_tools,
//! has_tool_result, response}`) from argv[1] or `STUB_LLM_FIXTURES`, and a
//! port from `STUB_LLM_PORT` (defaults to 4600, documented in
//! `tests/bdd/playwright.config.ts`).

use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tokio::net::TcpListener;

#[path = "../../tests/integration/live/stub_llm.rs"]
mod stub_llm;

use stub_llm::{FixtureResponse, FixtureSet, RequestFingerprint};

#[derive(Deserialize)]
struct FixtureFileEntry {
    model: String,
    last_user_message: String,
    #[serde(default)]
    has_tools: bool,
    #[serde(default)]
    has_tool_result: bool,
    response: FixtureFileResponse,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FixtureFileResponse {
    Content { text: String },
    ToolCall { name: String, arguments: String },
}

fn load_fixtures(path: &PathBuf) -> FixtureSet {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture file {}: {e}", path.display()));
    let entries: Vec<FixtureFileEntry> = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse fixture file {}: {e}", path.display()));

    entries.into_iter().fold(FixtureSet::new(), |set, entry| {
        let fingerprint = RequestFingerprint {
            model: entry.model,
            last_user_message: entry.last_user_message,
            has_tools: entry.has_tools,
            has_tool_result: entry.has_tool_result,
        };
        let response = match entry.response {
            FixtureFileResponse::Content { text } => FixtureResponse::Content(text),
            FixtureFileResponse::ToolCall { name, arguments } => {
                FixtureResponse::ToolCall { name, arguments }
            }
        };
        set.with(fingerprint, response)
    })
}

#[tokio::main]
async fn main() {
    let fixture_path = env::args()
        .nth(1)
        .or_else(|| env::var("STUB_LLM_FIXTURES").ok())
        .map(PathBuf::from)
        .expect("usage: stub-llm <fixture-file.json> (or set STUB_LLM_FIXTURES)");

    let port: u16 = env::var("STUB_LLM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4600);

    let fixtures = load_fixtures(&fixture_path);
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|e| panic!("failed to bind stub-llm on 127.0.0.1:{port}: {e}"));

    println!(
        "stub-llm listening on http://127.0.0.1:{port}/v1 (fixtures: {})",
        fixture_path.display()
    );
    stub_llm::serve(listener, fixtures).await;
}
