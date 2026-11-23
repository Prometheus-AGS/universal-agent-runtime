---
paths: ['**/*.rs', '**/Cargo.toml']
---

# Rust — universal-agent-runtime

Loaded when a Rust file is read. Not resident.

## Verification in UAR

Follow the project-owned **UAR verification: prove the user outcome** rule in
`AGENTS.md`. Generic tier labels do not prescribe commands or per-edit execution.
Inspect the intended Rust behavior and evidence mapping early; run the smallest
admitted compile and changed-behavior BDD path. `server-full` is the BossFang
sidecar profile; use it when the changed behavior or existing release gate
requires it, not as a blanket whole-workspace suite.

## Hard rules

- **Zero warnings.** Fix immediately, or annotate `#[expect(lint, reason="...")]`.
  A warning left in place is a defect with a timer on it.
- **Never `cargo clean`.** Active caches are preserved deliberately. Use reviewed,
  reversible cleanup only.
- Never `--release` during implementation. It invalidates incremental artifacts
  and pays full optimization for code that is about to change.
- One build profile per session. Switching thrashes the incremental cache.
- Batch related fixes. During implementation use source/IDE feedback and only
  the executable development checks admitted by the selected method. Do not
  rerun an unchanged build for a harness-only correction.
- CI and tests are asynchronous evidence, not the work queue. Do not watch
  workflows while actionable implementation remains.

## Style

4-space indent. `snake_case` for functions and modules, `CamelCase` for types.
`anyhow` for application errors, `tracing` for structured logs.

No glob re-exports (`pub use foo::*`); use `#[doc(inline)]` for public
re-exports. Public items carry `///` docs with `# Examples`, `# Errors`, and
`# Panics` sections.

## Build concurrency

Single-writer within one `CARGO_TARGET_DIR`. Across authorized, resource-isolated worktrees, use distinct target dirs and a
**shared** `CARGO_HOME`; do not schedule concurrent builds that compete for the
same host resources. Serialize dependency-mutating commands (`fetch`, `update`, `add`).
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
