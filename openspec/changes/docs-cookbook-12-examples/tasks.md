## 1. Create cookbook directory structure
- [x] 1.1 Created `docs/cookbook/` with `runtime/`, `sdk/`, and `a2ui/` subdirectories.
- [x] 1.2 Added `docs/cookbook/README.md` explaining structure, usage, and status.
- [x] 1.3 Added `docs/cookbook/a2ui/README.md` placeholder documenting the dependency on Changes 21–22.

## 2. Runtime examples
- [x] 2.1 Created `docs/cookbook/runtime/Cargo.toml` as a standalone crate depending on `universal-agent-runtime` (`in-memory-backend`).
- [x] 2.2 `01_start_server.rs` — dry-run the server startup config path.
- [x] 2.3 `02_load_config.rs` — load a UAR config file and inspect it.
- [x] 2.4 `03_mcp_tool_call.rs` — register and call an in-process native MCP-style tool.
- [x] 2.5 `04_streaming_sse.rs` — minimal Axum SSE streaming endpoint.
- [x] 2.6 Verified all four runtime examples build and run with `cargo run`.

## 3. SDK examples
- [x] 3.1 Created `docs/cookbook/sdk/rust/Cargo.toml` as a standalone crate depending on `universal-agent-runtime-sdk`.
- [x] 3.2 `sdk/rust/src/bin/01_init.rs` — initialize a UAR client.
- [x] 3.3 `sdk/python/examples/02_send_message.py` — send a chat message.
- [x] 3.4 `sdk/typescript/examples/03_handle_response.ts` — handle a chat response.
- [x] 3.5 `sdk/rust/src/bin/04_subscribe.rs` — subscribe to a streaming completion.
- [x] 3.6 Created `docs/cookbook/sdk/typescript/tsconfig.json` extending the SDK config for typechecking.

## 4. Validation tooling
- [x] 4.1 Created `tools/validate-cookbook.sh` that builds and runs self-contained examples, typechecks TypeScript, and skips A2UI placeholders.
- [x] 4.2 Made `tools/validate-cookbook.sh` executable.
- [x] 4.3 Added `VALIDATE_COOKBOOK_LIVE=1` mode for optional live execution against a running UAR server.

## 5. CI workflow
- [x] 5.1 Created `.github/workflows/cookbook.yml` to run `tools/validate-cookbook.sh` on PRs and pushes to `main`.

## 6. Final validation
- [x] 6.1 `cargo check` for Rust runtime and SDK cookbook crates passes.
- [x] 6.2 `tools/validate-cookbook.sh` passes in compile/typecheck mode.
- [x] 6.3 Python examples compile with `py_compile`.
- [x] 6.4 TypeScript examples typecheck with `tsc -p docs/cookbook/sdk/typescript/tsconfig.json`.
- [x] 6.5 `openspec validate --change docs-cookbook-12-examples` passes.
- [ ] 6.6 Mark Change 24 implementation complete in `progress.json` and update `current-waypoint.json` (after merge).

## Blockers

- A2UI cookbook examples are blocked on Changes 16–22 (`@a2ui` integration and
  UAR renderer migration).
- `openspec validate` may require a new capability spec delta under
  `openspec/specs/` for `cookbook-2026`; add it if validation fails.
