# wire-mcp-server-provisioning

## Why

`mcp/registry.rs`'s `McpServerEntry::Stdio` spawn path calls
`Command::new(&command_path)` directly with zero provisioning logic —
if the configured command (e.g. `kreuzberg`) isn't on `PATH`, the tool
call just fails with no recovery path. `provisioning-strategy-core`
(this round's first change) built a pluggable resolver for exactly this
situation; this change wires it in.

Also folds in a scope correction: the round's original second change,
`wire-toolchain-provisioning`, assumed there's a code path in UAR that
compiles a skill from source (needing Rust/Node/Python/Go/wasmtime).
There isn't one — the Dockerfile's own comment confirms those
toolchains are kept resident "for user builds" (a human manually
building their own skill inside the container), not something UAR's
runtime invokes. Rather than invent a fake call site, the 5 toolchain
`ToolSpec`s were added to `provisioning.rs` as ready-to-use, tested
recipes (`skill_toolchain_specs()`) with no forced wiring — confirmed
with the user as the right scope.

## What changed

- `src/mcp/registry.rs`: before spawning an `McpServerEntry::Stdio`
  entry, check `provisioning::is_on_path(&command_path)`. If it's not
  already resolvable, call `known_tool_spec(command)` +
  `ToolProvisioner::resolve()`. On success, spawn the resolved path
  instead. On failure, log a warning and fall through to spawning the
  originally-configured command exactly as before — provisioning
  failure surfaces through the pre-existing `set_mcp_server_status` +
  error path, not a new error type, so behavior for an unprovisionable
  tool is unchanged from before this change.
- `src/uar/orchestrator/provisioning.rs`: added `known_tool_spec(name)`
  (a curated spec for `kreuzberg`, matching the install methods already
  documented in `Cargo.toml`'s comment above the `kreuzberg` dependency
  — `brew tap kreuzberg-dev/tap && brew install kreuzberg-cli`, or
  `cargo install --path . --bin kreuzberg` after a clone; falls back to
  an Adopt-only spec for any uncurated command name, so this never
  invents installation strategies for arbitrary MCP commands) and
  `is_on_path(name_or_path)` (a public, allocation-light "is this
  already resolvable" check, exposed so call sites can preserve their
  fast path without needing the heavier `ToolProvisioner::resolve`
  machinery for the common case).
- `src/uar/orchestrator/provisioning.rs`: added `skill_toolchain_specs()`
  — the 5 toolchain `ToolSpec`s (rustc/node/python3/go/wasmtime),
  mirroring the Dockerfile's own install choices where practical (Go
  and wasmtime use the same prebuilt-release pattern the Dockerfile
  does). Documented explicitly as not wired to any current caller.

## Verification

- 6 new unit tests (13 total in the module now): curated `kreuzberg`
  spec shape, Adopt-only fallback for uncurated names, `is_on_path`
  finds/misses correctly, all 5 toolchain specs present in the right
  order, and a real (non-mocked) Adopt-path resolution of the `rustc`
  spec — this CI/dev environment has `rustc` since the crate itself
  needs it to build, so this exercises a genuine, non-synthetic case.
- `cargo test --lib`: 385/385 green (379 prior + 6 new).
- `cargo clippy --lib`: zero new warnings (502, unchanged).
- Existing `tests/test_mcp_optional.rs` (4/4) re-run and still green —
  confirms the fast path (command already resolvable) is unaffected.
- Not covered by an automated test: an actually-missing MCP command
  that successfully provisions via a real install (would modify this
  host). The fallback-to-original-behavior path (provisioning fails,
  falls through to the pre-existing spawn+error path) IS covered.
