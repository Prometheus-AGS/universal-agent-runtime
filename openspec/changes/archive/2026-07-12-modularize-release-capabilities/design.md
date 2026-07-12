## Baseline

Measured 2026-07-11 on the production-release worktree:

- `cargo tree --no-default-features --edges normal`: 1,175 unique rendered nodes.
- `cargo tree --all-features --edges normal`: 1,310 unique rendered nodes.
- `target/debug/universal-agent-runtime`: 452 MiB.

The small delta proves current features mostly toggle code while large native
and protocol dependencies remain unconditional. This is also direct evidence
against linking the present crate into BossFang.

## Boundary plan

Capability features must own both code and dependency edges. Release bundles
are additive aliases over those capability features:

- `minimal`: core HTTP/OpenAI-compatible chat, embedded persistence and MCP.
- `server-full`: minimal plus local embeddings, governance, quality,
  documents, telemetry, A2A and native WASM tools.
- `desktop-full`: server-full plus the Tauri desktop integration.

The unintegrated memory-palace SQLite extension is removed because UAR has no
palace API call sites. Model generation is a maintainer operation, never a
runtime capability. Compile-time assertions reject builds without a backend.

After capability isolation, the dependency-only in-memory build contains 763
unique rendered nodes, down from the 1,175-node baseline (35% fewer). The full
server bundle contains 1,136 nodes; its additional cost is now explicitly
selected rather than imposed on every consumer.

## Verification

Each product bundle is checked independently. Dependency-tree assertions prove
disabled dependency-bearing capabilities are absent, while API/config tests
prove unavailable capabilities do not advertise themselves.

Local verification on 2026-07-12 checked all three bundles. The minimal unit
suite compiled and passed 371 tests; six catalog-routing tests require the
models.dev catalog that the managed sandbox could not resolve, and one local
socket probe was excluded because sandbox binds are denied. The cross-platform
CI bundle matrix runs the unfiltered stable suite on Linux, macOS, and Windows.
