# rmcp-pin-bump

## Why

`rmcp` was pinned via git `rev` to `085470025f690050e8776ffa939e7ba71d3abc01`
(predating `rmcp`'s `v1.4.0` tag). `GHSA-89vp-x53w-74fx`: prior to
`1.4.0`, `rmcp`'s Streamable HTTP server transport did not validate the
incoming `Host` header, allowing a DNS-rebinding attack from a malicious
public page to send authenticated requests to an MCP server running on
the victim's loopback/private-network interface — full tool
enumeration/invocation, resource/prompt reads, and side effects (file
writes, shell execution, etc.) limited only by what tools the server
exposes. This project's `Cargo.toml` explicitly enables
`transport-streamable-http-server` (and `-session`), so this was a real,
directly-exposed vulnerability, not a theoretical one.

## What changed

Bumped the `rev` to the `rmcp-v1.8.0` tag
(`26b65b6b88c5552447905923f683b6e4720a5600`, well past the `v1.4.0` fix
commit `8e22aa2`/PR #764). `cargo update -p rmcp` also surfaced a
separate transitive `rmcp` instance (`v1.7.0`, pulled in by `kreuzberg`
via its own git tag) — already ≥1.4.0, so already unaffected; no
action needed there.

The bump broke the build in one systematic way: several rmcp types
(`Tool`, `CallToolRequestParams`, `Implementation`,
`InitializeResult`/`ServerInfo`, `StreamableHttpServerConfig`) became
`#[non_exhaustive]`. Contrary to the usual pattern (as fixed in
`wasmtime-disposition` for a different trait issue), `#[non_exhaustive]`
rejects **all** struct-literal syntax cross-crate — `..Default::default()`
does **not** work around it (confirmed via `rustc --explain E0639`,
which explicitly recommends looking for a `new` function instead). Fixed
by switching to each type's provided constructor + builder methods:

- `src/mcp/registry.rs` (3 sites): `Tool { .. }` → `Tool::new(name,
  description, input_schema)`; `CallToolRequestParams { .. }` →
  `CallToolRequestParams::new(name).with_arguments(args)`.
- `src/uar/mcp_server.rs` + `src/uar/memory/mcp_server.rs` (3 sites
  each, identical shape): `ServerInfo { .. }` →
  `ServerInfo::new(capabilities)` + field assignment;
  `Implementation { .. }` → `Implementation::new(name, version)`;
  `StreamableHttpServerConfig { .. }` → `StreamableHttpServerConfig::default()`
  + `config.stateful_mode = true` (all fields are `pub`, so direct
  mutation after `Default::default()` is the correct escape hatch for
  `#[non_exhaustive]` types, not struct-literal + `..`).

## Verification

- `cargo check` (default features): clean (was 9 `E0639` errors).
- `cargo check --features wasm-runtime`: clean (combined with
  `wasmtime-disposition`'s bump from earlier in this phase).
- `cargo test --lib`: 363/363 green.
- `cargo check --tests`: clean.
- `cargo test --test test_mcp_optional`: 4/4 green (MCP-registry-specific
  tests).
- `cargo test --test integration`: 56/56 green, 2 pre-existing
  `#[ignore]`d — including `tool_loop_round_trip`, which exercises a
  real MCP tool-call round trip through the actual server + stub LLM,
  proving the MCP client/server path still works end-to-end after the
  bump, not just that it compiles.
- `cargo clippy`: zero new warnings at any touched line. 2 pre-existing
  `field 'tool_router' is never read` warnings surfaced in both
  `mcp_server.rs` files (a `#[tool_router]`/`#[tool_handler]`
  macro-generated field, flagged by `dead_code` analysis but used
  internally by the macro expansion) — present in plain `cargo check`
  too, not clippy-specific, and not a correctness issue; disclosed, not
  investigated further as out of this change's scope.
