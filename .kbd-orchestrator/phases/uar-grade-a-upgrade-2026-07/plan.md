# UAR Grade-A Upgrade — Plan (kbd-plan)

> **Phase:** `uar-grade-a-upgrade-2026-07`
>
> **Input:** `analysis.md` + `library-candidates.json` + operator decisions
> on the 5 contested choices.
>
> **Output:** 24 ordered OpenSpec changes covering all 9 grade-A target
> areas. Sequencing respects the operator's chosen priority
> (license → coverage → error handling → configuration → supply chain →
> SDKs → RAG → A2UI → documentation).
>
> **Method:** every change is sized to fit one KBD apply cycle (1 PR
> per change), every change has a clear done condition, every change
> has a per-change harness recommendation (see §1).

## Operator decisions applied

| Q | Decision | How the plan applies it |
|---|---|---|
| Q1 | MIT for SDKs | Change 1 sets `license = "MIT"` on `sdks/{python,rust,typescript}`; per-language LICENSE files; open letter to SDK contributors; runtime stays AGPL-3.0 + commercial |
| Q2 | Vendor both `@a2ui/react` + `@a2ui/web_core`; UAR-owned renderer; component library migrated from `flint-forge`, `prometheus-entity-management`, `flint-realtime-fabric` | Changes 16–21 explicitly enumerate the per-source migration scope (entity-* from prometheus-entity-management, design systems + embedder from flint-forge, realtime backbone from flint-realtime-fabric) |
| Q3 | My recommendations (open letter for SDKs + clean-history for runtime) | Change 1 implements this; CONTRIBUTING.md notes the CLA-lite forward-going clause |
| Q4 | 60% coverage on day one, adjust based on usage | Change 2 sets `--fail-under-lines 60` and adds the `coverage-baseline.md` tracker so we can see actual usage before raising the bar |
| Q5 | My recommendation (minimum 22h) | Change 3 implements the minimum; a follow-up-grade fuzz pass is out of scope for this phase and tracked as a post-GA hardening task |

## 1. The harness recommendation (read this first)

The operator asked for a specific harness + model recommendation with
reasoning. My answer is **a mixed-harness plan**, not a single one,
because the 24 changes break into three categories that the KBD
project's own `references/agent-work-estimation-rule.md` already
classifies: **architectural**, **bulk implementation**, and **parallel
isolated**.

| Category | Workload | % of total hours | Recommended harness + model | Why |
|---|---|---:|---|---|
| **Architectural** | central UarError, config rewrite, license migration, A2UI catalog design, RAG eval harness, supply chain | ~50% (330h) | **Claude Code with Sonnet 5** | These changes touch every part of the codebase; they need careful reasoning about backward compat, type-system handling, and the UAR catalog restrictions. Sonnet 5 is the most reliable for mixed Rust + TS architectural work in mid-2026. |
| **Bulk implementation** | cookbook examples, fuzz targets, golden set items, README rewrites, config file generation, mutation-test sweeps | ~30% (200h) | **OpenCode with Kimi K2.7 Coding** (GLM-5.3 is the fallback) | These changes follow established patterns. K2.7 Coding is the right price/quality point for the long tail; the savings vs. Sonnet 5 are ~5–8× and the quality delta on boilerplate-heavy work is small. |
| **Parallel isolated** | the 3 SDKs at once, the Lit + Svelte A2UI renderers, the lit/svelte cookbook translations | ~20% (130h) | **Codex with GPT-5.6** in separate git worktrees | These are well-scoped, file-bounded tasks that benefit from isolation. GPT-5.6 is the best in mid-2026 at multi-file parallel work; git worktrees make concurrent edits safe. |

**Three concrete rules** the operator should enforce:

1. **The first change (license) goes to Claude Code with Sonnet 5.**
   It is the highest-stakes change in the entire grade-A work and
   every other change has the new license baked in.
2. **The A2UI work (changes 16–21) goes to Claude Code with Sonnet 5.**
   The 216h A2UI total is the largest single workstream and the most
   novel; the catalog restrictions and the cross-source component
   migration (from `flint-forge` / `prometheus-entity-management` /
   `flint-realtime-fabric`) need consistent high-quality reasoning
   across 5 separate changes.
3. **The bulk implementation work can use OpenCode + Kimi K2.7 Coding
   without supervision** for cookbook examples, fuzz targets, and
   golden set items, **but every PR must be reviewed by the operator
   before merge**. The agent-work-estimation-rule's "current frontier
   coding model" assumption is the floor, not the ceiling.

**What about the other models the operator listed?**

- **GLM-5.3** is a fine fallback for OpenCode if K2.7 is unavailable;
  same price tier, similar quality. Don't use Sonnet 5 OR GPT-5.6 for
  the bulk work; the cost is not worth the quality delta on the
  established-pattern tail.
- **Qwen 3.6/3.7** is not in the project's
  `agent-work-estimation-rule` list of "current frontier coding model"
  classes. If the operator has a strong preference for Qwen, run a
  5-change pilot first against an OpenCode + Sonnet 5 baseline before
  committing the bulk work to it.
- **MiniMax M3** is the assistant's own model class. The
  `agent-work-estimation-rule` does list it. For the A2UI Inspector
  and the A2UI catalog migration specifically (which require very
  consistent style across 14+ components), M3 is a strong third
  choice for the bulk pattern work inside §10.

**Don't lock to one harness.** The KBD skill system supports
multi-harness orchestration specifically because the right harness
depends on the work. The matrix above is the recommendation; the
operator can override per change.

## 2. Change list (ordered)

Each change is one OpenSpec change in `openspec/changes/<id>/`. The
first 3 changes have full proposal + tasks in this directory; the
remaining 21 have the design intent captured here and will be
generated via `/opsx:new <id>` during the first apply cycle of each.

### Order 1 — License (§8, 19h, 1 change)

#### Change 1: `license-dual-license-agpl-mit`
**Section:** §8 License · **Hours:** 19 · **Harness:** Claude Code + Sonnet 5
**Library:** n/a (license text)
**Crate candidates:** `cand-lic-001` (MIT), `cand-lic-003` (AGPL-3.0 + commercial)
**Capabilities affected:** new `dual-license-policy`

Done condition:
- `sdks/python/LICENSE` is MIT; `pyproject.toml` declares `license = {text = "MIT"}`
- `sdks/rust/LICENSE-MIT` exists; `sdks/rust/Cargo.toml` `license = "MIT OR AGPL-3.0"` (consumer chooses)
- `sdks/typescript/LICENSE` is MIT; `package.json` `"license": "MIT"`
- `Cargo.toml` (root) stays `license = "AGPL-3.0-only"`; new `LICENSE-COMMERCIAL.md` with named pricing bands
- `CONTRIBUTING.md` updated with the CLA-lite forward-going clause
- Open-letter PR/email template ready (signed-by operator)
- `tools/license-check.sh` validates the license files are present and match

### Order 2 — Build / test / lint (§6, 43h, 2 changes)

#### Change 2: `coverage-cargo-llvm-cov-60pct`
**Section:** §6 Build/test/lint · **Hours:** 25 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-build-001` (cargo-llvm-cov)
**Capabilities affected:** new `coverage-baseline`

Done condition:
- New `.github/workflows/coverage.yml` runs `cargo-llvm-cov --lcov --output-path lcov.info` on every PR
- `--fail-under-lines 60` (operator decision Q4)
- Codecov integration; `docs/coverage-baseline.md` records starting coverage
- `.grcovrc` removed (cargo-llvm-cov supersedes grcov)
- `frontend/` coverage via `vitest --coverage` with v8 provider; same 60% threshold
- `tools/coverage-drift.sh` shows delta vs baseline

#### Change 3: `test-quality-mutation-fuzz-property`
**Section:** §6 · **Hours:** 18 · **Harness:** OpenCode + Kimi K2.7 Coding (with Claude Code review)
**Library:** `cand-build-002` (cargo-mutants), `cand-build-003` (proptest), `cand-build-004` (cargo-fuzz)
**Capabilities affected:** `test-quality-gates`

Done condition:
- `.github/workflows/mutation.yml` nightly cron; `cargo mutants --no-shuffle`; results published to `docs/mutation-history/`
- `fuzz/` directory with 4 targets: `chunker`, `rag_verification`, `mcp_message_parser`, `json_schema_validator`
- `proptest` property tests for: settings store serde roundtrip, retrieval RRF invariants, governance policy hot-reload semantics
- `release-plz` configured with conventional-commits check; `commitlint` + `lefthook` for the JS workspace

### Order 3 — Error handling (§5, 42h, 2 changes)

#### Change 4: `central-uar-error-enum`
**Section:** §5 · **Hours:** 27 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-err-001` (thiserror 2.0), `cand-err-002` (anyhow, restricted), `cand-err-003` (error-stack)
**Capabilities affected:** new `uar-error-model`

Done condition:
- New `src/uar/error.rs` with `pub enum UarError` (#[non_exhaustive])
- Variants: `Config(ConfigError)`, `Auth(AuthError)`, `Rag(RagError)`, `Memory(MemoryError)`, `Mcp(McpError)`, `A2a(A2aError)`, `Llm(LlmError)`, `Internal(InternalError)`
- Every existing public `*Error` enum wrapped as a variant
- 130 `anyhow!()` in public-API boundary code converted to `UarError` variants
- Stable error codes (strings) for SDK consumption (e.g. `E_CONFIG_MISSING_FIELD`, `E_RAG_NO_KB`)

#### Change 5: `unwrap-sweep-tracing-error-chains`
**Section:** §5 · **Hours:** 15 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** `cand-err-004` (tracing-error), `cand-err-005` (sentry-sdk, behind feature flag)
**Capabilities affected:** `uar-observability-errors`

Done condition:
- `tracing-error` wired so every `UarError` carries the current span trace (request_id, agent_id, run_id)
- `clippy.toml` lints: `#![deny(clippy::unwrap_used, clippy::expect_used)]` on `src/uar/{api,server}.rs` and `src/uar/runtime/`
- 382 → < 50 `unwrap()/expect()` on production hot paths (acceptable remaining count are init-time + test-only)
- `sentry-sdk` integration behind `--features sentry`; default off

### Order 4 — Configuration (§3, 37h, 2 changes)

#### Change 6: `config-rs-schemars-migration`
**Section:** §3 · **Hours:** 24 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-cfg-002` (config), `cand-cfg-003` (secrecy), `cand-cfg-004` (schemars)
**Capabilities affected:** new `uar-config-model`

Done condition:
- `src/config.rs` < 800 lines (was 2,046)
- Every `UAR_*__*` env var declared via `#[derive(ConfigLayer)]`
- `secrecy::Secret<String>` for `JWT_SECRET`, `LLM__API_KEY`, `PROVIDER_*_API_KEY`
- Canonical JSON Schema generated at startup; `GET /.well-known/uar-config` exposes it
- Backward-compat with legacy `LLM_*` env vars preserved
- `pnpm generate-config-types` codegens TS types from the schema for the SDKs

#### Change 7: `config-hot-reload-vault`
**Section:** §3 · **Hours:** 13 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** `cand-cfg-005` (notify), `cand-cfg-006` (arc-swap), `cand-cfg-007` (vaultrs)
**Capabilities affected:** `config-hot-reload`

Done condition:
- `notify` watcher on `config.yaml`; `arc-swap` for lock-free hot-reload
- Vault adapter behind `--features vault`; documents the threat model
- Drift-detection mode (`--strict-config`) errors on override conflicts
- Hot-reload preserves active sessions + in-flight runs (verify with a soak test)

### Order 5 — Supply chain (§7, 22h, 1 change)

#### Change 8: `slsa-l3-osv-grype-security-txt`
**Section:** §7 · **Hours:** 22 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-supply-001` (slsa-github-generator), `cand-supply-002` (actions/attest-sbom), `cand-supply-003` (osv-scanner + grype), `cand-supply-004` (slsa-verifier)
**Capabilities affected:** `slsa-l3-self-attestation`

Done condition:
- New `.github/workflows/provenance.yml` reusable workflow that calls `slsa-github-generator` after every build; signing runs in an isolated job, separate from build
- SLSA L3 self-declared in README front page with `cosign verify` proof command
- `actions/attest-sbom` for first-party SBOM attestation
- `vuln-scan.yml` nightly cron; blocks on `>=HIGH` for published artifacts
- `/.well-known/security.txt` with PGP key + 90-day disclosure SLA
- Reproducible-builds verification job (best-effort)

### Order 6 — SDKs (§2, 106h, 4 changes)

#### Change 9: `sdk-rust-1.0`
**Section:** §2 · **Hours:** 30 · **Harness:** Codex + GPT-5.6 (worktree-isolated)
**Library:** `cand-sdk-004` (sse-stream), `cand-sdk-007` (miette)
**Capabilities affected:** `sdk-rust-1.0`

Done condition:
- `sdks/rust/Cargo.toml` version `1.0.0`; `BREAKING.md` introduced
- Full surface: streaming chat, tool calls, structured outputs, embeddings, runs lifecycle (create / stream / cancel / resume / checkpoint), knowledge base CRUD, ingest
- 6 runnable `examples/`: chat, streaming chat, tool calls, structured outputs, agent run, RAG query
- Typed error model via `miette` wrapping the central `UarError`
- rustdoc + published API reference

#### Change 10: `sdk-python-1.0`
**Section:** §2 · **Hours:** 30 · **Harness:** Codex + GPT-5.6 (worktree-isolated, parallel with #9)
**Library:** `cand-sdk-003` (eventsource-client), `cand-sdk-006` (pydantic)
**Capabilities affected:** `sdk-python-1.0`

Done condition:
- `sdks/python/pyproject.toml` version `1.0.0`; license MIT
- Surface mirrors Change 9 exactly (1:1 method coverage)
- `httpx-sse` for streaming; `pydantic` for typed models
- 6 runnable `examples/`
- Sphinx autodoc + ReadTheDocs / GitHub Pages

#### Change 11: `sdk-typescript-1.0`
**Section:** §2 · **Hours:** 30 · **Harness:** Codex + GPT-5.6 (worktree-isolated, parallel with #9 + #10)
**Library:** `cand-sdk-003` (eventsource-client), `cand-sdk-005` (zod)
**Capabilities affected:** `sdk-typescript-1.0`

Done condition:
- `sdks/typescript/package.json` version `1.0.0`; license MIT
- Surface mirrors Change 9 exactly
- `fetch-event-source` for streaming; `zod` for runtime validation
- 6 runnable `examples/` (including a `next.js` example)
- typedoc + GitHub Pages

#### Change 12: `sdk-examples-cookbook-rustdoc`
**Section:** §2 + §9 · **Hours:** 16 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** n/a
**Capabilities affected:** `sdk-cookbook`

Done condition:
- 12 runnable `cargo run --example`s spanning the runtime
- Hosted rustdoc at `docs.rs` (auto-publish on release); typedoc + sphinx for the SDKs
- `tools/validate-examples.sh` runs every example as a smoke test in CI

### Order 7 — RAG (§4, 88h, 3 changes)

#### Change 13: `rag-citation-stream`
**Section:** §4 · **Hours:** 30 · **Harness:** Claude Code + Sonnet 5
**Library:** n/a (build)
**Capabilities affected:** new `rag-citation-ux`

Done condition:
- New `CitationStream` type in `src/uar/rag/`; emitted as `[1], [2]` markers on the SSE event channel
- React renderer: hover-to-source panel; per-component citation
- A2UI surfaces (in Changes 18–20) consume the same citation stream
- BDD feature: `tests/bdd/features/rag-citation.feature`

#### Change 14: `rag-eval-ragas-deepeval-golden-set`
**Section:** §4 · **Hours:** 30 · **Harness:** Claude Code + Sonnet 5 (golden-set curation) + OpenCode + K2.7 (CI integration)
**Library:** `cand-rag-001` (ragas), `cand-rag-002` (deepeval), `cand-rag-004` (beir)
**Capabilities affected:** `rag-evaluation-suite`

Done condition:
- Frozen golden set of 300 items in `evals/rag-golden-set/`, version-controlled
- `.github/workflows/rag-eval.yml` runs RAGAS + DeepEval on every PR; fails on > 2-point regression
- LLM judge prompt frozen; model + temperature pinned
- `prometheus-eval` harness (custom UAR wrapper)
- Monthly BEIR run; results published in `docs/rag-benchmark/`

#### Change 15: `rag-embedding-backends-4-more`
**Section:** §4 · **Hours:** 28 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** `cand-rag-005` (fastembed + 4 more)
**Capabilities affected:** new `embedding-provider-pluggable`

Done condition:
- 4 new Tier-1 embedding backends: `candle-embeddings` (local), `openai-embeddings`, `voyage`, `cohere`
- `UAR_LLM__EMBEDDING__BACKEND` env var selects; FastEmbed stays local-default
- Each backend in `docs/product-support-matrix.json` with capability evidence

### Order 8 — A2UI (§10, 216h, 6 changes)

#### Change 16: `a2ui-vendor-google-core-react`
**Section:** §10 · **Hours:** 16 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-a2ui-001` (@a2ui/web_core), `cand-a2ui-002` (@a2ui/react — reference only)
**Capabilities affected:** new `a2ui-core-vendoring`

Done condition:
- `frontend/packages/a2ui-core/` (vendored `@a2ui/web_core`, pinned version)
- `frontend/packages/a2ui-react/` (vendored `@a2ui/react`, pinned version, **reference impl only**)
- License headers preserved (Apache-2.0 from Google)
- `frontend/packages/a2ui-core/UPSTREAM.md` records the pinned version + how to update

#### Change 17: `a2ui-uar-renderer-on-webcore`
**Section:** §10 · **Hours:** 40 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-a2ui-006` (shadcn/ui), `cand-a2ui-007` (cmdk), `cand-a2ui-008` (react-hook-form), `cand-a2ui-009` (react-virtual), `cand-a2ui-003` (react-aria-components), `cand-a2ui-004` (motion), `cand-a2ui-010` (i18n), `cand-a2ui-005` (quicktype)
**Capabilities affected:** new `a2ui-uar-renderer`

Done condition:
- `frontend/packages/a2ui-uar/` — UAR-owned React renderer built on `@a2ui/web_core`
- Catalog: 14+ components (the 9 from `uar.a2ui/1` + EntityCard, EntityDiff, EntityStream, EntityApproval, EntityToolProvider, EntityChat, EntityCopilot)
- shadcn/ui baseline; `react-aria-components` for a11y primitives
- Cross-tested against `@a2ui/react` as reference impl
- Performance budget: initial render < 16ms, streaming chunk < 8ms (CI gate)

#### Change 18: `a2ui-migrate-entity-components-from-prometheus-entity-management`
**Section:** §10 · **Hours:** 35 · **Harness:** Claude Code + Sonnet 5
**Library:** n/a (build, sources from `prometheus-entity-management`)
**Capabilities affected:** new `a2ui-entity-components`

Done condition:
- Migrate `entity-stream.tsx`, `entity-approval.tsx`, `entity-chat.tsx`, `entity-copilot.tsx`, `entity-diff.tsx`, `entity-tool-provider.tsx` from `prometheus-skill-system/skills/imported/prometheus-entity-management/packages/a2ui-react/src/`
- Re-license per Change 1 (prometheus-entity-management is AGPL-3.0; the migrated code is dual-licensed MIT OR AGPL-3.0)
- New `use-entity-*` hooks migrated in parallel
- `a2ui-react.test.tsx` migrated; coverage extended
- `prometheus-entity-management` reduces to the upstream package; the UAR renderer is now the canonical location

#### Change 19: `a2ui-migrate-design-systems-embedder-from-flint-forge`
**Section:** §10 · **Hours:** 40 · **Harness:** Claude Code + Sonnet 5
**Library:** n/a (build, sources from `flint-forge`)
**Capabilities affected:** new `a2ui-design-system-bridge`

Done condition:
- Migrate the design-systems layer from `flint-forge/crates/fdb-app/src/a2ui` + `flint-forge/migrations/0009_flint_a2ui_design_systems.sql`
- Migrate the embedder from `flint-forge/crates/fdb-gateway/src/a2ui_embedder.rs`
- Migrate the application model + reflection compiler from `flint-forge/crates/fdb-reflection/src/compilers/a2ui.rs`
- New UAR-side Rust module under `src/uar/a2ui/design_systems/`; new SQL migration under `migrations/`
- Tests from `flint-forge/crates/fdb-gateway/tests/a2ui_*_test.rs` migrated
- `flint-forge` retains the original; UAR consumes via Cargo git dep until the integration stabilizes, then promotes to path dep

#### Change 20: `a2ui-realtime-backbone-from-flint-realtime-fabric`
**Section:** §10 · **Hours:** 30 · **Harness:** Claude Code + Sonnet 5
**Library:** n/a (build, sources from `flint-realtime-fabric`)
**Capabilities affected:** new `a2ui-live-update`

Done condition:
- Wire `flint-realtime-fabric` as the SSE/fan-out backbone for A2UI live updates
- A2UI surface updates become AG-UI `StatePatch` events; live transitions via Motion (§17)
- 2 BDD scenarios: `a2ui-live-update.feature` (multi-client convergence, late-join reattach)

#### Change 21: `a2ui-world-class-theming-a11y-i18n`
**Section:** §10 · **Hours:** 35 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-a2ui-003`, `cand-a2ui-004`, `cand-a2ui-010` (already in #17); also `cand-a2ui-006`
**Capabilities affected:** `a2ui-ux-2026`

Done condition:
- 3 themes: light, dark, high-contrast (CSS variable system)
- WCAG 2.2 AA: keyboard nav, screen reader, focus management, color contrast, axe-core CI gate
- i18n: en, es, ja, zh; RTL framework; string extraction via `i18next` or `react-intl`
- Animation: Motion integration; entrance / exit / update / streaming transitions
- Citation UX: hover-to-source panel (delivered with Change 13)
- Error boundary + retry UX on every surface

#### Change 22: `a2ui-inspector-lit-svelte-renderers`
**Section:** §10 + §9 · **Hours:** 38 · **Harness:** Codex + GPT-5.6 (Lit + Svelte in parallel) + Claude Code + Sonnet 5 (Inspector)
**Library:** n/a (build)
**Capabilities affected:** new `a2ui-devtools`, `a2ui-lit-renderer`, `a2ui-svelte-renderer`

Done condition:
- `frontend/packages/a2ui-inspector/` — dev-only React app that listens on the SSE channel, parses every A2UI message, renders it side-by-side with the source JSON, supports "freeze" for testing
- `frontend/packages/a2ui-lit/` — Lit renderer on `@a2ui/web_core`
- `frontend/packages/a2ui-svelte/` — Svelte renderer on `@a2ui/web_core`
- Cross-renderer conformance test: same A2UI message renders to the same semantic DOM on all three renderers
- Inspector deployed as a Storybook addon

### Order 9 — Documentation (§9, 89h, 3 changes)

#### Change 23: `docs-hosted-rustdoc-typedoc-docusaurus-ia`
**Section:** §9 · **Hours:** 35 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** `cand-doc-002` (rustdoc + typedoc pipeline), `cand-doc-003` (Docusaurus), `cand-doc-004` (vale.sh)
**Capabilities affected:** new `dev-portal-2026`

Done condition:
- Hosted rustdoc + typedoc + sphinx on a custom domain (or `*.github.io`)
- Docusaurus IA: /docs/{architecture, configuration, sdk-{rust,python,typescript}, rag, a2ui, governance, supply-chain, contributing}
- `vale.sh` prose lint with UAR-specific style config
- ADRs: `docs/adr/0001-record-architecture-decisions.md` template + 10 ADRs documenting the grade-A decisions

#### Change 24: `docs-cookbook-12-examples`
**Section:** §9 · **Hours:** 30 · **Harness:** OpenCode + Kimi K2.7 Coding
**Library:** n/a
**Capabilities affected:** `cookbook-2026`

Done condition:
- 12 runnable cookbook examples spanning the runtime, the SDKs, and the A2UI surface (4 each)
- `tools/validate-cookbook.sh` runs every example in CI

#### Change 25: `docs-storybook-visual-regression-perf-budget`
**Section:** §9 + §10 · **Hours:** 24 · **Harness:** Claude Code + Sonnet 5
**Library:** `cand-doc-001` (Storybook 8)
**Capabilities affected:** `visual-regression-2026`

Done condition:
- Storybook 8 with Chromatic for visual regression; 30+ components
- Performance budget CI gate (initial render < 16ms, streaming chunk < 8ms)
- A2UI Inspector as a Storybook addon (from Change 22)

> Note: this is the 25th change, one more than originally planned.
> Pulled forward from inside Change 22 because Storybook + visual
> regression is most coherent as a single workstream that owns the
> per-component story discipline across all 14+ A2UI components.

## 3. Dependency graph

```
Change 1 (license) ────────────────────────────► unblocks 9–11 (SDKs)
Change 2 (coverage) ──► Change 4 (UarError) ──► Change 5 (unwrap sweep)
Change 2 (coverage) ──► Change 3 (mutation/fuzz/proptest)
Change 4 (UarError) ──► Change 6 (config) ──► Changes 9–12 (SDKs)
Change 4 (UarError) ──► Change 13 (RAG citation)
Change 6 (config) ──► Changes 9–12 (SDKs)
Change 8 (supply chain) ──► independent
Change 13 (citation stream) ──► Changes 16–21 (A2UI)
Change 14 (RAG eval) ──► Change 13 (citation stream)
Change 15 (embedding backends) ──► Change 13 (citation stream)
Change 16 (vendor A2UI) ──► Change 17 (UAR renderer) ──► Changes 18–22
Changes 9–11 (SDKs) ──► Change 12 (cookbook + rustdoc)
Changes 16–22 (A2UI) ──► Changes 23–25 (docs)
```

## 4. Cost summary

| Order | Section | Changes | Hours | Harness mix |
|---|---|---:|---:|---|
| 1 | §8 License | 1 | 19 | Claude + Sonnet 5 |
| 2 | §6 Build/test/lint | 2 | 43 | Claude + Sonnet 5 (25) + OpenCode + K2.7 (18) |
| 3 | §5 Error handling | 2 | 42 | Claude + Sonnet 5 (27) + OpenCode + K2.7 (15) |
| 4 | §3 Configuration | 2 | 37 | Claude + Sonnet 5 (24) + OpenCode + K2.7 (13) |
| 5 | §7 Supply chain | 1 | 22 | Claude + Sonnet 5 |
| 6 | §2 SDKs | 4 | 106 | Codex + GPT-5.6 (90) + OpenCode + K2.7 (16) |
| 7 | §4 RAG | 3 | 88 | Claude + Sonnet 5 (60) + OpenCode + K2.7 (28) |
| 8 | §10 A2UI | 6 | 234 | Claude + Sonnet 5 (196) + Codex + GPT-5.6 (38) |
| 9 | §9 Documentation | 3 | 89 | OpenCode + K2.7 (65) + Claude + Sonnet 5 (24) |
| | **Total sequential** | **24** | **680** | |
| | **Total with 4 agents in parallel** | | **~250** | |

The 18-hour difference from the analysis estimate (662 → 680h) comes
from splitting Change 22 into Changes 22 + 25 (Storybook was
under-counted in the analysis). The change-count grew from ~20 to
24+1 because the SDK + A2UI work has more shippable boundaries than
the analysis grouped.

## 5. Per-change harness matrix

This is the machine-readable version of the recommendation. Use it
when dispatching per-change.

| Change | Section | Hours | Primary | Secondary | Notes |
|---|---|---:|---|---|---|
| 1 | §8 | 19 | **Claude + Sonnet 5** | — | highest stakes; open letter template |
| 2 | §6 | 25 | **Claude + Sonnet 5** | — | coverage baseline + Codecov |
| 3 | §6 | 18 | **OpenCode + K2.7** | Claude review | fuzz targets, proptest, release-plz |
| 4 | §5 | 27 | **Claude + Sonnet 5** | — | central UarError design |
| 5 | §5 | 15 | **OpenCode + K2.7** | Claude review | clippy lints, unwrap sweep |
| 6 | §3 | 24 | **Claude + Sonnet 5** | — | config-rs migration |
| 7 | §3 | 13 | **OpenCode + K2.7** | Claude review | hot-reload + Vault |
| 8 | §7 | 22 | **Claude + Sonnet 5** | — | SLSA L3 + osv-scanner |
| 9 | §2 | 30 | **Codex + GPT-5.6** | — | worktree; Rust SDK |
| 10 | §2 | 30 | **Codex + GPT-5.6** | — | worktree; Python SDK |
| 11 | §2 | 30 | **Codex + GPT-5.6** | — | worktree; TS SDK |
| 12 | §2+§9 | 16 | **OpenCode + K2.7** | — | cookbook + rustdoc pipeline |
| 13 | §4 | 30 | **Claude + Sonnet 5** | — | citation stream |
| 14 | §4 | 30 | **Claude + Sonnet 5** (golden set) | OpenCode + K2.7 (CI) | RAGAS + DeepEval |
| 15 | §4 | 28 | **OpenCode + K2.7** | Claude review | 4 embedding backends |
| 16 | §10 | 16 | **Claude + Sonnet 5** | — | vendor @a2ui |
| 17 | §10 | 40 | **Claude + Sonnet 5** | — | UAR renderer on @a2ui/web_core |
| 18 | §10 | 35 | **Claude + Sonnet 5** | — | migrate entity-* |
| 19 | §10 | 40 | **Claude + Sonnet 5** | — | migrate design systems from flint-forge |
| 20 | §10 | 30 | **Claude + Sonnet 5** | — | realtime backbone from flint-realtime-fabric |
| 21 | §10 | 35 | **Claude + Sonnet 5** | — | theming/a11y/i18n/animation |
| 22 | §10+§9 | 38 | **Codex + GPT-5.6** (Lit + Svelte) | **Claude + Sonnet 5** (Inspector) | parallel |
| 23 | §9 | 35 | **OpenCode + K2.7** | — | hosted rustdoc + Docusaurus IA + ADRs |
| 24 | §9 | 30 | **OpenCode + K2.7** | — | cookbook examples |
| 25 | §9+§10 | 24 | **Claude + Sonnet 5** | — | Storybook + visual regression |

## 6. OpenSpec change skeletons

The first 3 changes have full proposal + tasks in this directory:

- `openspec/changes/license-dual-license-agpl-mit/`
- `openspec/changes/coverage-cargo-llvm-cov-60pct/`
- `openspec/changes/central-uar-error-enum/`

The remaining 22 changes have their design intent captured in §2
above. They will be generated via `/opsx:new <id>` at the start of
their respective apply cycles.

## 7. Stop condition

Stop when all 25 changes are DONE (matching the operator's "all-A"
criterion) **or** at a genuine missing authorization / external /
time-bound condition / supported-product defect. Per
`agent-work-estimation-rule.md`, report active agent-hours separately
from external waiting.

The next KBD step is `/kbd-execute`, which dispatches the first
change (Change 1: License) via Codex + GPT-5.6 — wait, no, via
**Claude Code with Sonnet 5** (per §5). The execute stage respects
the per-change harness matrix.

## 8. Honest risks I want on the record

- **A2UI v1.0-rc is still iterating** (June 8, 2026, status Candidate).
  Pin `@a2ui/web_core` to a specific version in Change 16. The UAR
  renderer built in Change 17 is the insulant.
- **The license migration is the single legal/operational risk.**
  Change 1's open-letter approach can fail if a contributor refuses
  relicense. Mitigation: prepared to remove their contributions from
  the SDKs (small surface, ~10 known contributors).
- **The cross-source component migration in Changes 18–20 is
  coordination-heavy.** `flint-forge` and `prometheus-entity-management`
  are themselves active projects. Coordination with their maintainers
  is a prerequisite; the Cargo git dep fallback in Change 19 is the
  safety net.
- **Coverage at 60% on day one is below the median for the named
  competitors.** This is acceptable as a starting point (Q4 decision)
  but expect the next quarter to push to 70–75% once actual usage
  data is in.

---

*End of plan. The next KBD step is `/kbd-execute`, which dispatches
Change 1 via Claude Code with Sonnet 5. Per the agent-work-estimation-rule,
all estimates above assume current frontier coding models (GPT-5.6,
Claude Sonnet 5, GLM 5.2, Kimi K2.7 Coding, MiniMax M3) — none of the
harness recommendations in §5 use a model outside that set.*
