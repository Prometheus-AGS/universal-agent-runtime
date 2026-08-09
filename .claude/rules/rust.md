---
paths: ['**/*.rs', '**/Cargo.toml']
---

# Rust — universal-agent-runtime

Loaded when a Rust file is read. Not resident.

| Tier | Command |
|---|---|
| T0 every edit | `cargo check --locked --no-default-features --features server-full` |
| T1 unit complete | `cargo test <test_name>` — the just-written unit only |
| T2 phase complete | `cargo fmt --all -- --check`; `cargo test --locked --no-default-features --features server-full` |
| T3 milestone only | `cargo build --release`; supported-profile tests; release certification |

`server-full` is the BossFang sidecar profile and is the checkpoint feature set.
Scope T0 to a crate where the workspace allows it; never workspace-wide on every
edit.

## Hard rules

- **Zero warnings.** Fix immediately, or annotate `#[expect(lint, reason="...")]`.
  A warning left in place is a defect with a timer on it.
- **Never `cargo clean`.** Active caches are preserved deliberately. Use reviewed,
  reversible cleanup only.
- Never `--release` during implementation. It invalidates incremental artifacts
  and pays full optimization for code that is about to change.
- One build profile per session. Switching thrashes the incremental cache.
- Batch related fixes. During implementation use static inspection and a cohesive
  `cargo check`; validate the finished product in one consolidated sequence.
- CI and tests are asynchronous evidence, not the work queue. Do not watch
  workflows while actionable implementation remains.

## Style

4-space indent. `snake_case` for functions and modules, `CamelCase` for types.
`anyhow` for application errors, `tracing` for structured logs.

No glob re-exports (`pub use foo::*`); use `#[doc(inline)]` for public
re-exports. Public items carry `///` docs with `# Examples`, `# Errors`, and
`# Panics` sections.

## Build concurrency

Single-writer within one `CARGO_TARGET_DIR`. Across worktrees with separate
target dirs and a **shared** `CARGO_HOME`, run check, build, test, and clippy in
parallel; serialize only dependency-mutating commands (`fetch`, `update`, `add`).
A separate `CARGO_HOME` per agent breaks registry sharing and forces full
recompiles — the fingerprint includes that path.

## Platform support

Linux and macOS are Stable. Windows is Experimental and non-blocking.

## LLM access

All LLM access goes through liter-llm — 142+ providers via unified
`provider/model` addressing. Set `UAR_LLM__MODEL` and `UAR_LLM__API_KEY`, or a
provider shortcut. The `llm:` section of `example.config.yaml` is the reference.

Model routing goes through `POST /api/uar/route` with capability requirements
(`needs_tools`, `needs_vision`, `min_context`). The catalog is built at compile
time from models.dev plus liter-llm schemas.
