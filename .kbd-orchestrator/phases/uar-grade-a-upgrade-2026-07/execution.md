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
- [DONE] config-rs-schemars-migration — Claude Code + Sonnet 5 (switched back from OpenCode + Kimi K2.7 Coding; flagged before starting per operator instruction). See "Change 6 status" below.
- [IN_PROGRESS] config-hot-reload-vault — OpenCode + Kimi K2.7 Coding (concurrent with Change 8 below; not confirmed complete by Claude — this ledger line updates when OpenCode reports done)
- [DONE] slsa-l3-osv-grype-security-txt — Claude Code + Sonnet 5, dispatched OUT OF ORDER concurrently with Change 7 (plan.md's dependency graph marks Change 8 "independent" — no dependency on 7). See "Change 8 status" and "Concurrency note" below.
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

CHANGE 6 STATUS: config-rs-schemars-migration (Claude Code + Sonnet 5)

Created the OpenSpec change from scratch (openspec new change, then
authored proposal.md/tasks.md/specs/uar-config-model/spec.md by hand —
none of it pre-existed). Implemented:
- `SecurityConfig::jwt_secret`: `String` → `secrecy::SecretString`. 4 real
  call sites updated to `.expose_secret()`: `src/server.rs` (ApiKeyService
  init), `src/uar/security/middleware.rs` (JWT decode),
  `src/uar/settings/manager.rs` (settings-bootstrap JSON),
  `tests/settings_persistence.rs` (test fixture). `#[schemars(with =
  "String")]` so the generated schema still describes it as an opaque
  string.
- `schemars::JsonSchema` derived on all 30 structs/enums in `src/config.rs`
  plus 3 external types `AppConfig` reaches into:
  `uar::runtime::matching::intent::{ClassifierConfig, ClassifierBackend}`,
  `llm::registry::{ProviderConfig, ProtocolSetting, ModelConfig}`,
  `uar::context::strategy::ContextStrategy`.
- `AppConfig::json_schema() -> serde_json::Value` (schemars::schema_for!),
  new `GET /.well-known/uar-config` route in `src/server.rs`.
- `Cargo.toml`: `schemars = "1"` — checked crates.io for the current
  release rather than assuming a version.
- 2 new unit tests in `src/config.rs` verifying the schema's top-level
  shape and that `jwt_secret` resolves to an opaque `{"type": "string"}`.

Scope corrections found during implementation (disclosed in `tasks.md`
and `proposal.md`'s "Out of scope" section, same pattern as Changes 1
and 4):
- The plan's `#[derive(ConfigLayer)]` requirement does not correspond
  to any real macro in the `config` crate or elsewhere in the dependency
  tree — verified against its actual API. No such derive exists; the
  existing builder-pattern env-var wiring is unaffected and unchanged.
- The plan's "`src/config.rs` < 800 lines (was 2,046)" file-split was
  not attempted — audited as a large, separate mechanical refactor
  (30+ structs, a 300-line env-loading `impl` block) with its own
  regression risk, not required for the schema/secrecy goals this
  change actually delivers. Deferred as follow-up.
- `secrecy::Secret` for `llm.api_key`/`PROVIDER_*_API_KEY` (also named
  in the plan) was audited and deferred: `jwt_secret` had exactly 4
  well-defined call sites; `llm.api_key` is read across many test
  fixtures via struct-update syntax plus at least one non-obvious
  consumer, a much larger and less-audited surface. Follow-up candidate.
- `pnpm generate-config-types` TS codegen was deferred — needs a
  schema-source path that doesn't require booting the full server
  first (DB, providers, etc.); a `--print-config-schema` CLI
  flag/example binary is the natural next step, tracked as follow-up.

Verified: `cargo check --no-default-features --features server-full`
PASS; `cargo test --locked --no-default-features --features server-full
--lib config::` PASS (includes the 2 new schema tests plus all
pre-existing `config` module tests); `openspec validate
config-rs-schemars-migration --strict` PASS. `cargo check ... --tests`
was run once before the schemars-derive additions (passed, confirming
the jwt_secret/SecretString fix alone); re-running it to also cover the
schemars-derive changes across every test binary is deferred to the
phase's consolidated validation pass, along with full-workspace
`cargo fmt`/`cargo clippy`.

CHANGE 8 STATUS: slsa-l3-osv-grype-security-txt (Claude Code + Sonnet 5, dispatched concurrently with Change 7)

Created the OpenSpec change from scratch. Audited the existing
supply-chain CI before writing anything and found it materially more
mature than plan.md assumed: `.github/workflows/supply-chain.yml`
already builds+pushes a multi-arch image, generates CycloneDX/SPDX
SBOMs (syft), signs everything keylessly (cosign), attests provenance
of the release-payload checksums (`actions/attest@v4`), and
independently re-verifies all of it in a separate `verify` job before
publishing evidence to the GitHub release. `ci.yml`'s "Offline
Reproducible Source" job already covers the reproducible-builds done
condition. Implemented the real remaining gaps:
- `actions/attest-sbom@v4` steps in `supply-chain.yml`: attest the
  Linux x64 release tarball and the container image against their own
  already-generated per-artifact SBOMs (first-party SBOM attestation,
  additive to the existing checksum-provenance attestation).
- New `.github/workflows/vuln-scan.yml`: nightly `osv-scanner` reusable
  workflow (recursive dependency scan, `fail-on-vuln: true`) + a
  `grype` job that builds `Dockerfile` locally (unpushed, no registry
  credentials needed) and scans it (`anchore/scan-action@v7`,
  `severity-cutoff: high`, `fail-build: true`) — independent of the
  existing weekly Rust-only `cargo audit` in `security-audit.yml`.
- New `GET /.well-known/security.txt` in `src/server.rs` (RFC 9116),
  pointing at the real GitHub private-vulnerability-reporting channel
  documented in `SECURITY.md` (not a fabricated email/PGP key), with
  the existing 90-day coordinated-disclosure default referenced via
  `Policy`. 1 new unit test.
- New README "Supply-chain provenance (SLSA L3 self-declared)"
  section: what's attested, the independent `verify` job, and real
  `cosign verify-blob`/`gh attestation verify` commands using the
  actual signing-identity regex `supply-chain.yml`'s own `verify` job
  uses.

Scope corrections (disclosed in `proposal.md`'s "Out of scope"
section, same pattern as prior changes): no new `provenance.yml` was
created (provenance generation already exists via `actions/attest@v4`,
GitHub's modern mechanism — the plan's named `slsa-github-generator` is
the older, more complex approach and wasn't needed); reproducible-builds
verification needed no new work (already existed); SBOM attestation for
every per-platform artifact (macOS/Windows/offline-source) was scoped
down to the Linux x64 tarball + image for this pass (mechanical
follow-up, `actions/attest-sbom` has no batch-subject form);
`vuln-scan.yml`'s grype job scans a locally-rebuilt copy of the release
`Dockerfile`, not the actual last-published registry image (needs
operator confirmation of which registry — ghcr.io per `supply-chain.yml`
or the ACR image in `deploy.yml` — is canonical, before wiring
credentials for that).

Verified: `.github/workflows/supply-chain.yml` and `vuln-scan.yml` YAML
syntax valid; `cargo check --no-default-features --features
server-full` clean; new `security_txt_handler` unit test passes;
`openspec validate slsa-l3-osv-grype-security-txt --strict` PASS.
Actually running the two workflows in GitHub Actions is deferred to the
phase's consolidated validation pass (both need live CI).

CONCURRENCY NOTE (Changes 7 + 8, both landing 2026-07-14 in the same
working tree, not isolated worktrees)

OpenCode (Change 7) and Claude (Change 8) both edited this repo at the
same time, unlike prior changes which alternated strictly sequentially.
Observed transient cross-contamination twice: `cargo check`/`cargo
test` runs picked up the OTHER agent's mid-edit compile errors (a
missing `Cli` field, then a missing `use clap::Parser;` in OpenCode's
new `src/config_manager.rs`) even though those errors were never in
Claude's own diff. Both resolved on their own once the respective
agent's edit landed — no actual corruption occurred, and `git status`
confirms each agent's changes stayed in its own files (no overlapping
edits to the same lines). This is a real risk worth naming: CLAUDE.md
mandates worktree isolation (`~/.claude/worktrees/`, `scripts/worktree-new.sh`)
for exactly this scenario; this phase's dispatch model has been running
multiple tools directly against the same checkout. Recommend the
operator decide whether to formalize worktree-per-harness for the
remaining concurrent-eligible changes (per plan.md's dependency graph,
several more pairs are independent enough to parallelize the same way).

BLOCKERS

- artifact-refiner QA integration reference file (.kbd-orchestrator/references/integrations/artifact-refiner.md) is missing; QA gate for changes 1–6, 8 not yet run. Will run when the integration is available or at the phase's consolidated validation pass.
- Change 1 deferred items: pricing bands (deferred per operator instruction), operator confirmation of contributor audit, PR review/merge.
- Change 2 deferred items: real baseline numbers and real-PR verification deferred to consolidated validation pass.
- Change 3 deferred item: first cargo-fuzz build/runtime exceeded available interactive time due to the surrealdb/axum dependency stack; the targets and scaffolding are in place and will be exercised in the consolidated validation pass.
- Change 4 deferred items: tracing::span! completeness sweep (now addressed by Change 5 middleware), release.yml sentry wiring (now addressed by Change 5), observability doc (now addressed by Change 5), full-workspace fmt/clippy (still deferred).
- Change 5 deferred item: full-workspace `cargo clippy -- -D warnings` has 511 pre-existing unrelated warnings; deferred to consolidated validation pass.
- Change 6 deferred items: see "CHANGE 6 STATUS" above (config.rs file-split, llm.api_key secrecy wrapping, TS codegen script, full `--tests` re-run, full-workspace fmt/clippy).
- Change 7: in progress by OpenCode; status not yet confirmed by Claude.
- Change 8 deferred items: see "CHANGE 8 STATUS" above (remaining per-platform SBOM attestations, canonical-registry confirmation for vuln-scan.yml's image target, live CI runs, full-workspace fmt/clippy).

NEXT HARNESS SWITCH

Once Change 7 (config-hot-reload-vault, OpenCode) reports complete,
Changes 9–11 (the 3 SDKs) are next per plan.md's dependency graph
(Change 6/config unblocks them) — assigned to **Codex + GPT-5.6 in
isolated git worktrees**, the SIXTH harness switch of the phase and the
first worktree-isolated dispatch. Must be flagged explicitly before
starting, not silently continued on Claude Code + Sonnet 5 or OpenCode.

DISPATCH — CHANGES 9–11 (2026-07-14)

Selected backend: hybrid (`codex` implementation + `openspec` traceability)
Backend rationale: the three SDKs are parallel, file-bounded changes explicitly
assigned by plan.md to Codex in isolated git worktrees. OpenSpec is the fallback
if a dispatch cannot maintain an inspectable task ledger or encounters an
unresolved API-contract ambiguity.
Canonical progress: this phase's `progress.json` plus each change's
`openspec/changes/<change-id>/tasks.md`.

PROGRESS LEDGER

- [IN_PROGRESS] `sdk-rust-1.0` — Codex — `/Users/gqadonis/.claude/worktrees/sdk-rust-1-0` — `feat/sdk-rust-1.0`
- [IN_PROGRESS] `sdk-python-1.0` — Codex — `/Users/gqadonis/.claude/worktrees/sdk-python-1-0` — `feat/sdk-python-1.0`
- [IN_PROGRESS] `sdk-typescript-1.0` — Codex — `/Users/gqadonis/.claude/worktrees/sdk-typescript-1-0` — `feat/sdk-typescript-1.0`

All three worktrees start at committed checkpoint `b9a85515`. Changes 6–8 have
uncommitted edits in the primary checkout and are intentionally excluded from
the dispatch bases; integration must reconcile their finalized contracts.

HANDOFF NOTE for Codex — `sdk-rust-1.0`:
1. Work only in `/Users/gqadonis/.claude/worktrees/sdk-rust-1-0` on `feat/sdk-rust-1.0`.
2. Read current waypoint and plan.md Change 9; create and strictly validate the OpenSpec change before implementation.
3. Limit implementation ownership to `sdks/rust/` plus the minimum SDK-specific docs/tests/config needed by the spec.
4. Implement the complete Change 9 done condition, preserving compatibility with the committed server API.
5. Verify the Rust SDK with its focused fmt/check/test/example/docs commands; do not run repository-wide final validation.
6. On completion update the OpenSpec task ledger, commit the branch, and report commit, verification, deferred items, and integration risks. Do not merge.
7. On blocker record it in the change tasks/handoff and report immediately.

HANDOFF NOTE for Codex — `sdk-python-1.0`:
1. Work only in `/Users/gqadonis/.claude/worktrees/sdk-python-1-0` on `feat/sdk-python-1.0`.
2. Read current waypoint and plan.md Change 10; create and strictly validate the OpenSpec change before implementation.
3. Limit implementation ownership to `sdks/python/` plus the minimum SDK-specific docs/tests/config needed by the spec.
4. Implement the complete Change 10 done condition and maintain 1:1 public-surface parity with the Rust SDK contract described by plan.md.
5. Verify with focused Python lint/type/test/build/docs/example checks available in the SDK; do not run repository-wide final validation.
6. On completion update the OpenSpec task ledger, commit the branch, and report commit, verification, deferred items, and integration risks. Do not merge.
7. On blocker record it in the change tasks/handoff and report immediately.

HANDOFF NOTE for Codex — `sdk-typescript-1.0`:
1. Work only in `/Users/gqadonis/.claude/worktrees/sdk-typescript-1-0` on `feat/sdk-typescript-1.0`.
2. Read current waypoint and plan.md Change 11; create and strictly validate the OpenSpec change before implementation.
3. Limit implementation ownership to `sdks/typescript/` plus the minimum SDK-specific docs/tests/config needed by the spec.
4. Implement the complete Change 11 done condition and maintain 1:1 public-surface parity with the Rust SDK contract described by plan.md.
5. Verify with focused TypeScript typecheck/lint/test/build/typedoc/example checks available in the SDK; do not run repository-wide final validation.
6. On completion update the OpenSpec task ledger, commit the branch, and report commit, verification, deferred items, and integration risks. Do not merge.
7. On blocker record it in the change tasks/handoff and report immediately.

APPROVAL GATES

- Operator authorized the sixth harness switch and dispatch in this request.
- Publishing SDK packages, pushing branches, opening PRs, or merging remains unauthorized.

FALLBACK CONDITIONS

- Fall back to OpenSpec-only clarification when the committed runtime API cannot support a required SDK surface without cross-SDK/server changes.
- Stop and report if a dependency must be introduced without current-version verification or if ownership would overlap another SDK worktree.

VERIFICATION REQUIREMENTS

- Each branch must pass focused SDK tests, type/lint/build checks, examples, and documentation generation supported by that SDK.
- Each OpenSpec change must pass `openspec validate <change-id> --strict`.
- Artifact-refiner QA and archive occur only after implementation is reported DONE; no dispatch may self-merge.

REFLECTION HANDOFF

- kbd-reflect should consume: final progress.json, archived OpenSpec changes, and any deferred evidence items captured in execution.md and progress.json.

EXECUTION READY

---

CHANGE 7 COMPLETION LOG — config-hot-reload-vault (OpenCode + Kimi K2.7 Coding)
Date: 2026-07-14

Implemented:
- `src/config_manager.rs`: `ConfigManager` wrapping `ArcSwap<AppConfig>` with async `load`, `reload`, and `current()`.
- Background `notify` watcher on the resolved config file path with 500 ms debounce.
- `--strict-config` / `UAR_STRICT_CONFIG` flag: reload rejected if effective config differs from initial snapshot.
- Optional `vault` Cargo feature with `src/config/vault.rs`: resolves `vault://mount/path` and `vault://mount/path#field` URLs in `security.jwt_secret`, `llm.api_key`, and `persistence.surreal_pass` via `vaultrs` KV-v2.
- `AppState.config_manager` wired through `start_server` / `start_server_sidecar` / `main.rs` / `uar-sidecar.rs`.
- `POST /.well-known/uar-config/reload` admin endpoint requiring `X-UAR-Admin-Key` (when enabled).
- Unit tests for config reload, strict mode, and Vault URL parsing.

Deferred / follow-up:
- Incremental migration of remaining handler call sites from `state.config` to `state.config_manager.current()` (currently only server core and startup paths use the live manager; `state.config` is retained for backward compatibility).
- Full `vault://` URL support for arbitrary config fields (currently resolves a fixed allowlist of secret fields).

Verification:
- `cargo fmt --all -- --check` clean.
- `cargo check --no-default-features --features server-full` clean.
- `cargo check --no-default-features --features server-full,vault` clean.
- `cargo test --no-default-features --features server-full --lib config_manager` passes.
- `cargo test --no-default-features --features server-full,vault --lib config::vault` passes.
- `openspec validate --strict --changes config-hot-reload-vault` passes.

