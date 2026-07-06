# fmt-drift-cleanup

## Why

`uar-carryover-audit`'s assessment found the carried debt note
"Unformatted routes.rs + ingestion_worker.rs on main" was stale in its
specific file names (neither appears in a fresh `cargo fmt --check`),
but the underlying problem — unformatted code drifting onto `main` — is
real and current: 21 diffs across 12 different files, likely
accumulated incrementally across several recent phases/`spawn_task`
sessions each adding a few unformatted lines rather than one large
event.

## What changed

Ran `cargo fmt` across the whole workspace. Files reformatted:
`src/server.rs`, `src/uar/compiler/conformance.rs` (7 sites),
`src/uar/eval/integration_tests.rs`, `src/uar/guardrails.rs`,
`src/uar/mcp_server.rs`, `src/uar/memory/mcp_server.rs`,
`src/uar/runtime/skills/wasm_runtime.rs` (2 sites),
`tests/agent_templates_test.rs`, `tests/bdd.rs` (3 sites),
`tests/integration/live/load_test.rs`. Formatting-only — zero behavior
change.

## Verification

- `cargo fmt --check`: clean (was 21 diffs across 12 files).
- `cargo check --lib`: clean (same 2 pre-existing, unrelated
  `tool_router` dead_code warnings as every recent phase).
