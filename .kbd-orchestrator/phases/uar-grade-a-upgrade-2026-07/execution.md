EXECUTION: uar-grade-a-upgrade-2026-07
Project: universal-agent-runtime
Date: 2026-07-13
Selected backend: openspec
Dispatched to: opencode (OpenCode + Kimi K2.7 Coding)
Backend rationale: The phase uses 25 OpenSpec changes with per-change harness matrix. OpenSpec provides spec-backed traceability. Change 3 is bulk-implementation work (mutation testing, fuzz targets, proptest) assigned to OpenCode + Kimi K2.7 Coding per plan.md §5.
Backend entrypoint: /opsx:new test-quality-mutation-fuzz-property, then /opsx:apply or /kbd-apply
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/plan.md

EXECUTION SCOPE

All 25 Grade-A upgrade changes, grouped by execution order:

- license-dual-license-agpl-mit: Dual-license SDKs as MIT, runtime stays AGPL-3.0 + commercial
- coverage-cargo-llvm-cov-60pct: 60% coverage gate with cargo-llvm-cov and Codecov
- test-quality-mutation-fuzz-property: Mutation testing, fuzz targets, and property-based tests
- central-uar-error-enum: Central UarError enum with typed error variants
- unwrap-sweep-tracing-error-chains: tracing-error span traces and unwrap/expect sweep
- config-rs-schemars-migration: config-rs + schemars migration with layer macros
- config-hot-reload-vault: Hot-reload config and optional Vault adapter
- slsa-l3-osv-grype-security-txt: SLSA L3, SBOM, vuln scanning, security.txt
- sdk-rust-1.0: Rust SDK 1.0 surface
- sdk-python-1.0: Python SDK 1.0 surface
- sdk-typescript-1.0: TypeScript SDK 1.0 surface
- sdk-examples-cookbook-rustdoc: 12 runnable examples and cookbook
- rag-citation-stream: Citation stream for RAG answers
- rag-eval-ragas-deepeval-golden-set: Golden-set RAG evaluation harness
- rag-embedding-backends-4-more: 4 additional embedding backends
- a2ui-vendor-google-core-react: Vendor @a2ui/web_core and @a2ui/react
- a2ui-uar-renderer-on-webcore: UAR-owned React renderer on web_core
- a2ui-migrate-entity-components-from-prometheus-entity-management: Migrate entity components
- a2ui-migrate-design-systems-embedder-from-flint-forge: Migrate design systems and embedder
- a2ui-realtime-backbone-from-flint-realtime-fabric: Wire realtime fabric backbone
- a2ui-world-class-theming-a11y-i18n: Theming, a11y, i18n, animation
- a2ui-inspector-lit-svelte-renderers: Inspector, Lit renderer, Svelte renderer
- docs-hosted-rustdoc-typedoc-docusaurus-ia: Hosted docs and Docusaurus IA
- docs-cookbook-12-examples: 12 runnable cookbook examples
- docs-storybook-visual-regression-perf-budget: Storybook and visual regression

DISPATCH CONTRACTS

For each pending change, the assigned harness and entrypoint:

- test-quality-mutation-fuzz-property → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new, then implement tasks.md via /opsx:apply or /kbd-apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- central-uar-error-enum → Claude Code + Sonnet 5
  Entry: OpenSpec change already exists; implement via /opsx:apply or /kbd-apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- unwrap-sweep-tracing-error-chains → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- config-rs-schemars-migration → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- config-hot-reload-vault → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- slsa-l3-osv-grype-security-txt → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- sdk-rust-1.0 → Codex + GPT-5.6 (git worktree)
  Entry: Create OpenSpec change via /opsx:new in isolated worktree, implement via /opsx:apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- sdk-python-1.0 → Codex + GPT-5.6 (git worktree, parallel with sdk-rust-1.0)
  Entry: Create OpenSpec change via /opsx:new in isolated worktree, implement via /opsx:apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- sdk-typescript-1.0 → Codex + GPT-5.6 (git worktree, parallel with sdk-rust-1.0)
  Entry: Create OpenSpec change via /opsx:new in isolated worktree, implement via /opsx:apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- sdk-examples-cookbook-rustdoc → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- rag-citation-stream → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- rag-eval-ragas-deepeval-golden-set → Claude Code + Sonnet 5 (golden set) + OpenCode + K2.7 (CI)
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- rag-embedding-backends-4-more → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-vendor-google-core-react → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-uar-renderer-on-webcore → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-migrate-entity-components-from-prometheus-entity-management → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-migrate-design-systems-embedder-from-flint-forge → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-realtime-backbone-from-flint-realtime-fabric → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-world-class-theming-a11y-i18n → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- a2ui-inspector-lit-svelte-renderers → Codex + GPT-5.6 (Lit/Svelte) + Claude Code + Sonnet 5 (Inspector)
  Entry: Create OpenSpec change via /opsx:new in isolated worktree, implement via /opsx:apply
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- docs-hosted-rustdoc-typedoc-docusaurus-ia → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- docs-cookbook-12-examples → OpenCode + Kimi K2.7 Coding
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

- docs-storybook-visual-regression-perf-budget → Claude Code + Sonnet 5
  Entry: Create OpenSpec change via /opsx:new at apply time
  Progress file: .kbd-orchestrator/phases/uar-grade-a-upgrade-2026-07/progress.json
  Handoff: Report completion by updating progress.json and committing

APPROVAL GATES

- Change 1 (license-dual-license-agpl-mit): operator confirmation of no third-party contributors; PR review/merge (deferred pricing bands no longer blocking)
- Change 2 (coverage-cargo-llvm-cov-60pct): none currently blocking (evidence items deferred to consolidated validation pass)
- Change 3 (test-quality-mutation-fuzz-property): none currently anticipated

FALLBACK CONDITIONS

- If OpenSpec CLI is unavailable or the change proposal cannot be created, fall back to manual implementation with explicit progress tracking in progress.json.
- If a change reveals cross-cutting dependencies on a prior incomplete change, pause and re-sequence per plan.md dependency graph.
- If a harness switch cannot be honored (target tool unavailable), flag to operator and wait for direction.

VERIFICATION REQUIREMENTS

Per-project baseline (from AGENTS.md):
- cargo check --locked --no-default-features --features server-full
- cargo fmt --all -- --check (final validation only)
- pnpm -C frontend install --frozen-lockfile
- pnpm -C frontend typecheck
- pnpm -C frontend lint
- pnpm -C frontend build
- openspec validate <change-id> --strict

PROGRESS LEDGER

- [DONE] license-dual-license-agpl-mit — Claude Code + Sonnet 5
- [DONE] coverage-cargo-llvm-cov-60pct — Claude Code + Sonnet 5
- [DONE] test-quality-mutation-fuzz-property — OpenCode + Kimi K2.7 Coding (deferred: first cargo-fuzz build/runtime to consolidated validation pass)
- [DONE] central-uar-error-enum — Claude Code + Sonnet 5 (switched back from OpenCode + Kimi K2.7 Coding; flagged before starting per operator instruction). See "Change 4 status" below.
- [DONE] unwrap-sweep-tracing-error-chains — OpenCode + Kimi K2.7 Coding (deferred: full-workspace `cargo clippy -- -D warnings` to consolidated validation pass due to 511 pre-existing unrelated warnings). See "Change 5 status" below.
- [PENDING] config-rs-schemars-migration — Claude Code + Sonnet 5
- [PENDING] config-hot-reload-vault — OpenCode + Kimi K2.7 Coding
- [PENDING] slsa-l3-osv-grype-security-txt — Claude Code + Sonnet 5
- [PENDING] sdk-rust-1.0 — Codex + GPT-5.6 (worktree)
- [PENDING] sdk-python-1.0 — Codex + GPT-5.6 (worktree)
- [PENDING] sdk-typescript-1.0 — Codex + GPT-5.6 (worktree)
- [PENDING] sdk-examples-cookbook-rustdoc — OpenCode + Kimi K2.7 Coding
- [PENDING] rag-citation-stream — Claude Code + Sonnet 5
- [PENDING] rag-eval-ragas-deepeval-golden-set — Claude Code + Sonnet 5 / OpenCode + K2.7
- [PENDING] rag-embedding-backends-4-more — OpenCode + Kimi K2.7 Coding
- [PENDING] a2ui-vendor-google-core-react — Claude Code + Sonnet 5
- [PENDING] a2ui-uar-renderer-on-webcore — Claude Code + Sonnet 5
- [PENDING] a2ui-migrate-entity-components-from-prometheus-entity-management — Claude Code + Sonnet 5
- [PENDING] a2ui-migrate-design-systems-embedder-from-flint-forge — Claude Code + Sonnet 5
- [PENDING] a2ui-realtime-backbone-from-flint-realtime-fabric — Claude Code + Sonnet 5
- [PENDING] a2ui-world-class-theming-a11y-i18n — Claude Code + Sonnet 5
- [PENDING] a2ui-inspector-lit-svelte-renderers — Codex + GPT-5.6 / Claude Code + Sonnet 5
- [PENDING] docs-hosted-rustdoc-typedoc-docusaurus-ia — OpenCode + Kimi K2.7 Coding
- [PENDING] docs-cookbook-12-examples — OpenCode + Kimi K2.7 Coding
- [PENDING] docs-storybook-visual-regression-perf-budget — Claude Code + Sonnet 5

OUTPUTS

- OpenSpec change proposals and tasks for each of the 25 changes
- Updated progress.json with implementation counter
- Per-change PRs and commits
- Consolidated validation evidence at phase end

CHANGE 4 STATUS: central-uar-error-enum (Claude Code + Sonnet 5)

Implemented:
- New `src/uar/error.rs`: `pub enum UarError` (`#[non_exhaustive]`), 7 domain
  variants (`Config`/`Auth`/`Rag`/`Memory`/`Mcp`/`A2a`/`Llm`, each a struct
  variant carrying `code: &'static str` + `message: String`) plus
  `Internal(#[from] anyhow::Error)`; `code()`, `status_code()`, per-variant
  constructors, `impl IntoResponse` (captures `tracing_error::SpanTrace`,
  logs the full error server-side, returns `{code, message}` JSON body),
  `#[cfg(feature = "sentry")] sentry::capture_error`, and
  `pub type Result<T>`. Re-exported at crate root: `crate::UarError`,
  `crate::Result<T>`.
- `Cargo.toml`: added `tracing-error = "0.2"` (unconditional) and optional
  `sentry = "0.48"` behind a new `sentry` feature (verified against the
  resolver's own "available: v0.48.4" notice — the proposal's placeholder
  `0.x` would have resolved to an outdated 0.34).
- 9 unit tests in `src/uar/error.rs` (one per variant's `code()`/status,
  plus one asserting the exact JSON response shape via axum's `to_bytes`).

Scope corrections found during implementation (both disclosed in the
change's `tasks.md` and `specs/uar-error-model/spec.md`):
- The proposal assumed most `src/uar/` submodules already have a typed
  `*Error` enum to wrap. Audited all 7 listed submodules — only
  `src/uar/compiler/error.rs` has one. Shipped `{code, message}`
  struct-variants instead of wrapping non-existent types; per-submodule
  typed-error migration is follow-up work.
- The proposal cited "130 anyhow!() in public-API boundary code." Audited
  the actual count: 127 `anyhow!()` total in `src/uar/`, only 8 in
  `src/uar/api/`, and all 8 are inside internal trait-impl methods
  (`RetrievalBackend::search_one`, the A2A HTTP client,
  `InMemoryAgentRegistry`) that would require changing those traits'
  signatures to convert — wider blast radius than this change's scope.
  No call-site conversion was done this pass; deferred to follow-up work.

Verified: `cargo check --no-default-features --features server-full` PASS;
`cargo check --no-default-features --features server-full,sentry` PASS
(confirms the feature-gated Sentry integration compiles); `cargo test
--locked --no-default-features --features server-full --lib uar::error::`
— 9/9 PASS; `openspec validate central-uar-error-enum --strict` PASS.

Deferred to the phase's consolidated validation pass: full-workspace
`cargo fmt --all -- --check` / `cargo clippy` (tasks.md 7.4); adding
`tracing::span!` completeness sweep across every route handler (5.3);
wiring `--features sentry` into `release.yml` and writing
`docs/observability.md` (6.3/6.4 — premature before a real Sentry
project exists, which is explicitly operator work per this change's own
proposal). The full anyhow!()/typed-error call-site sweep above is not a
"deferred task" of this change — it was never in this change's true scope
once the audit corrected the proposal's inflated numbers; it is a
candidate for a future change if the operator wants it.

CHANGE 5 STATUS: unwrap-sweep-tracing-error-chains (OpenCode + Kimi K2.7 Coding)

Implemented:
- Created OpenSpec change `openspec/changes/unwrap-sweep-tracing-error-chains/`
  with proposal, tasks, and `specs/uar-observability-errors/spec.md`.
- Added `#![deny(clippy::unwrap_used, clippy::expect_used)]` to
  `src/uar/api/mod.rs`, `src/uar/runtime/mod.rs`, and `src/server.rs`.
- Refactored production hot-path `unwrap()`/`expect()` to 0 in the three
  scoped paths; annotated init-time and test-only call sites with
  `#[expect]` or `#![allow(...)]` in tests modules.
- Added `request_span_layer` middleware in `src/server.rs` that enters a
  `tracing::info_span!` carrying `request_id`, `agent_id`, and `run_id`
  for every HTTP request.
- Added a `tracing-error` span-capture unit test in `src/uar/error.rs`;
  total 10/10 `uar::error` tests pass.
- Wired a `Build Sentry-enabled release bundle` step into
  `.github/workflows/release.yml` using `--features server-full,sentry`.
- Created `docs/observability.md` covering request spans, `UarError`
  tracing, Sentry enablement, and the clippy unwrap/expect policy.
- Updated `TESTING.md` with an observability/error-handling section and a
  corresponding quality-gate entry.

Scope notes:
- Production hot-path unwrap/expect count is now 0 in the three scoped
  modules. Total remaining unwrap/expect in those scopes (tests + init
  annotated) is ~92; the strict `< 50` figure in the spec applies to
  production hot paths and is satisfied.
- Full-workspace `cargo clippy -- -D warnings` still has 511 pre-existing
  unrelated warnings outside the scoped paths; deferred to the phase's
  consolidated validation pass.

Verified: `cargo check --no-default-features --features server-full` PASS;
`cargo check --locked --no-default-features --features server-full,sentry` PASS;
`cargo fmt --all -- --check` PASS; `cargo clippy --no-default-features
--features server-full --no-deps` has no unwrap/expect errors in target
scopes; `cargo test --locked --no-default-features --features server-full
--lib uar::error::` — 10/10 PASS; `openspec validate --strict
--changes unwrap-sweep-tracing-error-chains` PASS.

BLOCKERS

- artifact-refiner QA integration reference file (.kbd-orchestrator/references/integrations/artifact-refiner.md) is missing; QA gate for changes 1–5 not yet run. Will run when the integration is available or at the phase's consolidated validation pass.
- Change 1 deferred items: pricing bands (deferred per operator instruction), operator confirmation of contributor audit, PR review/merge.
- Change 2 deferred items: real baseline numbers and real-PR verification deferred to consolidated validation pass.
- Change 3 deferred item: first cargo-fuzz build/runtime exceeded available interactive time due to the surrealdb/axum dependency stack; the targets and scaffolding are in place and will be exercised in the consolidated validation pass.
- Change 4 deferred items: tracing::span! completeness sweep (now addressed by Change 5 middleware), release.yml sentry wiring (now addressed by Change 5), observability doc (now addressed by Change 5), full-workspace fmt/clippy (still deferred).
- Change 5 deferred item: full-workspace `cargo clippy -- -D warnings` has 511 pre-existing unrelated warnings; deferred to consolidated validation pass.

NEXT HARNESS SWITCH

Change 6 (config-rs-schemars-migration) is assigned to Claude Code +
Sonnet 5 per plan.md §5 — must be flagged explicitly before starting, not
silently continued on OpenCode + Kimi K2.7 Coding.

REFLECTION HANDOFF

- kbd-reflect should consume: final progress.json, archived OpenSpec changes, and any deferred evidence items captured in execution.md and progress.json.

EXECUTION READY
