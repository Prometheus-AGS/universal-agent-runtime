# UAR Cookbook

This directory contains runnable examples for the Universal Agent Runtime.

## Structure

- `runtime/` — 4 Rust examples demonstrating the UAR runtime itself.
- `sdk/` — 4 SDK examples across Rust, Python, and TypeScript.
- `a2ui/` — 4 placeholder examples for the A2UI surface (out of scope until Changes 21–22 land).

## Running examples

Each example is self-contained. The easiest way to validate the collection is to
run the CI helper:

```bash
tools/validate-cookbook.sh
```

The helper compiles every example and runs the ones that do not require a live
UAR server or external LLM backend.

## Runtime examples

| # | Example | What it shows | Runnable in CI? |
|---|---|---|---|
| 1 | `runtime/src/bin/01_start_server.rs` | Dry-run the server startup config path | Yes |
| 2 | `runtime/src/bin/02_load_config.rs` | Load a UAR config file and inspect it | Yes |
| 3 | `runtime/src/bin/03_mcp_tool_call.rs` | Register and call an in-process native tool | Yes |
| 4 | `runtime/src/bin/04_streaming_sse.rs` | Minimal Axum SSE streaming endpoint | Yes |

## SDK examples

| # | Example | What it shows | Language |
|---|---|---|---|
| 1 | `sdk/rust/src/bin/01_init.rs` | Initialize a UAR client | Rust |
| 2 | `sdk/python/examples/02_send_message.py` | Send a chat message | Python |
| 3 | `sdk/typescript/examples/03_handle_response.ts` | Handle a chat response | TypeScript |
| 4 | `sdk/rust/src/bin/04_subscribe.rs` | Subscribe to a streaming completion | Rust |

## A2UI examples (out of scope)

See `a2ui/README.md` for the planned examples and the blocker.
