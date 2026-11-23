//! Standalone boot target for the `tests/integration/live/stub_llm.rs`
//! fixture server, so Playwright's `webServer` (browser-driven BDD suite,
//! `tests/bdd/`) can boot a deterministic, keyless LLM backend the same way
//! `tests/bdd.rs`'s in-process Cucumber suite already does.
//!
//! Reads fixture files (each a JSON array of `{model, last_user_message,
//! has_tools, has_tool_result, response}`) from the positional arguments or
//! `STUB_LLM_FIXTURES`, and a port from `STUB_LLM_PORT` (defaults to 4600,
//! documented in `tests/bdd/playwright.config.ts`).
//!
//! `STUB_LLM_EXTRA_FIXTURES` (optional, comma-separated file paths) loads more
//! fixture files after the positional ones; a later entry with the same
//! fingerprint replaces an earlier one. The `hold_content` and `served_model` kinds are
//! meant to live in such an extra file. Introspection routes: `GET/POST
//! /_stub/requests`, `GET /_stub/held`, `POST /_stub/release`, and `GET
//! /_stub/capabilities`.

use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tokio::net::TcpListener;

#[path = "../../tests/integration/live/stub_llm.rs"]
#[expect(
    dead_code,
    reason = "This exact-fingerprint CLI omits the shared fixture's DelayedContent and with_prompt_prefix cases used by live integration tests"
)]
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
    Content {
        text: String,
    },
    GroundedContent {
        required_context: String,
        text: String,
    },
    ToolCall {
        name: String,
        arguments: String,
    },
    HoldContent {
        text: String,
    },
    ServedModel {
        served_model: String,
        text: String,
    },
}

/// Paths named by `STUB_LLM_EXTRA_FIXTURES`, in order; empty when unset.
fn extra_fixture_paths() -> Vec<PathBuf> {
    env::var("STUB_LLM_EXTRA_FIXTURES")
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn load_fixtures(base: FixtureSet, path: &PathBuf) -> FixtureSet {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture file {}: {e}", path.display()));
    let entries: Vec<FixtureFileEntry> = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse fixture file {}: {e}", path.display()));

    entries.into_iter().fold(base, |set, entry| {
        let fingerprint = RequestFingerprint {
            model: entry.model,
            last_user_message: entry.last_user_message,
            has_tools: entry.has_tools,
            has_tool_result: entry.has_tool_result,
        };
        let response = match entry.response {
            FixtureFileResponse::Content { text } => FixtureResponse::Content(text),
            FixtureFileResponse::GroundedContent {
                required_context,
                text,
            } => FixtureResponse::GroundedContent {
                required_context,
                text,
            },
            FixtureFileResponse::ToolCall { name, arguments } => {
                FixtureResponse::ToolCall { name, arguments }
            }
            FixtureFileResponse::HoldContent { text } => FixtureResponse::HoldContent { text },
            FixtureFileResponse::ServedModel { served_model, text } => {
                FixtureResponse::ServedModel { served_model, text }
            }
        };
        set.with(fingerprint, response)
    })
}

#[tokio::main]
async fn main() {
    // Every positional argument is a fixture file; with none, STUB_LLM_FIXTURES
    // names the one file. A stub built before multi-file support reads only the
    // first positional argument.
    let mut fixture_paths: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    if fixture_paths.is_empty() {
        fixture_paths.extend(env::var("STUB_LLM_FIXTURES").ok().map(PathBuf::from));
    }
    assert!(
        !fixture_paths.is_empty(),
        "usage: stub-llm <fixture-file.json>... (or set STUB_LLM_FIXTURES)"
    );

    let port: u16 = env::var("STUB_LLM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(4600);

    let fixtures = fixture_paths
        .iter()
        .chain(extra_fixture_paths().iter())
        .fold(FixtureSet::new(), load_fixtures);
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|e| panic!("failed to bind stub-llm on 127.0.0.1:{port}: {e}"));

    let loaded = fixture_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!("stub-llm listening on http://127.0.0.1:{port}/v1 (fixtures: {loaded})");
    stub_llm::serve(listener, fixtures).await;
}
