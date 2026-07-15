## Why

The UAR Grade-A upgrade plan (Change 24) requires 12 runnable cookbook examples
spanning the runtime, the SDKs, and the A2UI surface. The repository already has
SDK examples in `sdks/{rust,python,typescript}/examples/`, but there is no
unified cookbook location that covers the runtime and ties the examples together.

This change introduces a single `docs/cookbook/` directory, implements the 8
non-A2UI examples now, and leaves A2UI placeholders for the work tracked in
Changes 16–22.

## What Changes

- Create `docs/cookbook/` with subdirectories for `runtime/`, `sdk/` (Rust,
  Python, TypeScript), and `a2ui/`.
- Add 4 runtime examples (Rust):
  - `01_start_server.rs` — dry-run the server startup config path.
  - `02_load_config.rs` — load a UAR config file and inspect it.
  - `03_mcp_tool_call.rs` — register and call an in-process native MCP-style tool.
  - `04_streaming_sse.rs` — minimal Axum SSE streaming endpoint.
- Add 4 SDK examples across the 3 language SDKs:
  - `sdk/rust/src/bin/01_init.rs` — initialize a UAR client.
  - `sdk/python/examples/02_send_message.py` — send a chat message.
  - `sdk/typescript/examples/03_handle_response.ts` — handle a chat response.
  - `sdk/rust/src/bin/04_subscribe.rs` — subscribe to a streaming completion.
- Add `tools/validate-cookbook.sh` that compiles and runs the self-contained
  examples, typechecks the TypeScript, and skips the A2UI placeholders with a
  clear message.
- Add `.github/workflows/cookbook.yml` to run the validation on every PR and
  push to `main`.
- Leave A2UI examples as placeholders under `docs/cookbook/a2ui/` with a
  `README.md` explaining the dependency on Changes 21–22.

## Capabilities

### New Capabilities

- `cookbook-2026`: a unified, CI-validated cookbook of runnable UAR examples.

## Impact

- **No production code changes.** All examples are additive documentation and
  tooling.
- **CI gate added.** Future cookbook drift will fail `tools/validate-cookbook.sh`.
- **SDK API drift is caught.** Every Rust and Python example is compiled against
  the current SDK; TypeScript examples are typechecked.
- **A2UI examples are deferred.** They are documented and skipped rather than
  invented ahead of the renderer work.

## Out of scope

- The 4 A2UI cookbook examples. They require the `@a2ui` integration and UAR
  renderer work from Changes 16–22.
- Live end-to-end execution of SDK examples against a running UAR server. The
  validation script supports `VALIDATE_COOKBOOK_LIVE=1`, but the default CI mode
  is compile/typecheck only.
- Updating the main documentation site (Docusaurus) to embed the cookbook. That
  is covered by Change 23.
