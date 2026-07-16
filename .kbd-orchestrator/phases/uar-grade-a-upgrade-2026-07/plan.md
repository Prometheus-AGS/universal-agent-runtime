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

---

## Supplemental Plan — Admin/Agents UI defects (operator-directed, 2026-07-15)

> **Phase:** `uar-grade-a-upgrade-2026-07` (supplemental — outside the original
> 24-change scope above, which remains unchanged and fully merged)
>
> **Input:** `assessment.md`'s operator-directed investigation into 4 reported
> Admin console defects + service-worker console errors (written
> 2026-07-15T20:37:54Z).
>
> **library-candidates.json applicability:** the existing file covers the
> original 24 changes' build-vs-adopt decisions and does not cover these
> findings. All 6 changes below are first-party bugfixes/UX gaps in code
> already in this repo, not net-new capability — no library evaluation
> applies.
>
> **OpenSpec available:** YES (`openspec/` exists at project root). Changes
> below are OpenSpec change-ids; scaffolding (`openspec new change`) happens
> in the Spec stage, not here.

### Sycophancy self-check

- **S-02**: The plan does not assume the "provider then model" and "Edit
  Agent panel" complaints are fully valid as stated — Finding 3 in the
  assessment already established the Default Model field works correctly,
  and Change 3 below scopes to the real gap (missing two-step UX,
  catalog/registered-model mismatch) rather than a blanket "rewrite the
  panel" that the literal complaint would imply.
- **S-07**: Scope is held to the 6 findings in the assessment; no adjacent
  "while we're in there" work was added (e.g., not bundling a Providers-page
  redesign, not adding new governance policy authoring UI beyond what's
  needed to reconcile the observed Deny behavior).
- **S-03**: Explicit trade-off surfaced below — Change 5 (governance) and
  Change 6 (freeze) are marked **investigation-first**, not "fix," because
  the assessment could not confirm root cause for either. Committing to an
  implementation estimate for unconfirmed root causes would be the kind of
  caveat collapse this check exists to catch.

### CHANGE LIST (ordered)

1. `admin-sw-scheme-safe-caching`: skip `cache.put()` for any non-http(s)
   request scheme in the service worker's fetch handler
   - Scope: frontend (sw.js only)
   - Depends on: NONE
   - Recommended agent: OpenCode + Kimi K2.7 Coding (matches this phase's
     established pattern for small, well-scoped, single-file fixes — see
     Changes 3/5/7/12/15/23/24 in the original plan)
   - Est. complexity: S (< 1 hour)
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM (removes console noise; not user-blocking, but
     is the most visible/alarming symptom in the original bug report)
   - Details: Add a scheme check (`new URL(event.request.url).protocol`)
     alongside the existing method/path filters at sw.js:40-47; only attempt
     `cache.put()` for `http:`/`https:` requests. No behavior change for the
     app's own assets. Root cause and fix location fully confirmed in
     assessment Finding 1 — this is close to a mechanical fix.

2. `admin-agent-model-warning-clarity`: distinguish "defers to system
   default" from "broken" in the agent list/detail warning UI
   - Scope: frontend (Admin Agents list + detail panel)
   - Depends on: NONE
   - Recommended agent: Claude Code + Sonnet 5 (small but requires reading
     the actual runtime-resolution semantics correctly to avoid a
     misleading fix — see note below)
   - Est. complexity: S (< 1 hour)
   - Complexity score: Low
   - Model class: small
   - Customer value: HIGH (this is the exact confusion that triggered
     today's bug report; a copy/logic fix here directly prevents recurrence)
   - Details: An agent with empty `policy.provider.default` is not
     necessarily broken — it correctly falls through to the system-wide
     registry default (confirmed in assessment Finding 4). Change the
     warning condition and/or copy so it only fires when the agent has no
     usable resolution path at all (i.e., also no registry default
     configured), not merely "no explicit per-agent override." Where an
     override is genuinely absent, prefer neutral wording ("Using system
     default") over a yellow warning triangle.

3. `admin-agent-provider-first-model-picker`: two-step provider-then-model
   selection in the Edit Agent Identity tab, scoped to registered models
   - Scope: frontend (Edit Agent Identity tab) + api (verify `/api/agents`
     response already carries enough data; likely no backend change needed)
   - Depends on: NONE (independent of Change 2, though they address the
     same screen — sequencing them together in one PR is reasonable if the
     assignee prefers, but each is independently shippable)
   - Recommended agent: Claude Code + Sonnet 5 (UI pattern reuse across
     the Providers page and the agent editor; needs to reason about shared
     component extraction, not just a local fix)
   - Est. complexity: M (1–4 hours)
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (this is the operator's primary, explicitly
     stated ask: "Agents need to be configurable for their provider and
     model — in that order")
   - Details: Reuse the Providers page's existing "select a provider, then
     see its models" pattern (assessment Finding 2 confirms this pattern
     already exists in the codebase) inside the Edit Agent Identity tab,
     replacing the current single flat "Default Model" combobox. Scope the
     model list to models actually registered for the selected provider
     (per `GET /api/uar/providers`), not the full static catalog — this
     closes the secondary gap where a catalog-only model (e.g. the original
     "gpt-5.2") can be selected and silently fail at chat time.

4. `admin-agent-edit-panel-verification`: complete save-path verification
   for Prompt / Capabilities / Memory tabs
   - Scope: frontend + api (whichever save endpoints are found broken)
   - Depends on: `admin-agent-provider-first-model-picker` (run after,
     since Change 3 touches the same dialog and re-verifying beforehand
     would need re-verifying again after)
   - Recommended agent: Claude Code + Sonnet 5
   - Est. complexity: M (1–4 hours)
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM (closes out the "nothing works" claim with
     certainty; assessment Finding 3 already disproved it for the model
     field specifically, but Prompt/Capabilities/Memory save-paths were
     not exhaustively tested)
   - Details: For each tab, make an edit, save, reload, and confirm the
     edit persisted and (where applicable — e.g. system prompt) took
     effect in a real chat turn. Fix whichever specific save paths are
     found broken; do not assume the whole panel needs a rewrite going in.

5. `governance-tool-approval-reconciliation`: reconcile the Governance
   tab's "Tool Approval: auto" control with the observed native tool-denial
   behavior — **investigation-first**
   - Scope: api (governance engine, `src/uar/governance/engine.rs`) +
     frontend (Governance tab) — exact scope depends on investigation outcome
   - Depends on: NONE
   - Recommended agent: Claude Code + Sonnet 5 (security-adjacent; per this
     project's own priority rules, security/data-integrity work outranks
     convenience features and should not go to a smaller model)
   - Est. complexity: L (4–8 hours) — includes the investigation, not just
     the fix
   - Complexity score: High
   - Model class: frontier
   - Customer value: MEDIUM-HIGH (tool-call approval is a trust/safety
     boundary; an operator who can't predict which tools will silently fail
     mid-conversation can't reason about what the agent will actually do)
   - Details: First determine which of the two explanations in assessment
     Finding 5 is correct — (a) an intentional fail-closed default for
     specific built-in tools when zero Cedar policies are loaded, or (b) the
     "auto" UI control is genuinely disconnected from the real enforcement
     path. Only after that's confirmed should the fix be scoped: either
     surface the fail-closed default explicitly in the UI (if (a)), or wire
     the "auto"/"manual" control to the real enforcement mechanism (if (b)).
     This is flagged High/frontier specifically because implementing a fix
     against the wrong explanation would create a new, harder-to-diagnose
     security-UX mismatch.

6. `admin-ui-freeze-diagnostics`: reproduce and instrument the reported UI
   freeze — **investigation-first, not a fix**
   - Scope: frontend (Admin console) + possibly build config (Web Worker
     boundary for PGLite)
   - Depends on: NONE
   - Recommended agent: Claude Code + Sonnet 5, working session with the
     operator present (freeze was not reproduced without the original
     reporter's exact interaction sequence)
   - Est. complexity: M (1–4 hours) for diagnostics; the fix itself (if
     PGLite/main-thread WASM is confirmed as cause) is unscoped pending
     that result
   - Complexity score: Medium (diagnostics) / unscored (fix, pending root
     cause)
   - Model class: medium
   - Customer value: HIGH (a frozen admin console blocks all other
     configuration work, including Changes 2-5 above)
   - Details: Assessment Finding 6 could not reproduce the freeze and
     identified PGLite (WASM Postgres) loading on the Admin/Agents route as
     a plausible but unconfirmed factor. Next step is a live session with
     the operator reproducing the exact sequence, with a Long Task
     PerformanceObserver and main-thread profiling active, plus confirming
     whether PGLite initialization is confined to a Web Worker. Do not
     attempt a blind fix (e.g. "move PGLite to a worker") without
     confirming it's actually implicated — that risks masking the real
     cause.

### EXECUTION ROUND ORDER

Round 1 (parallel): `admin-sw-scheme-safe-caching`,
`admin-agent-model-warning-clarity`, `admin-agent-provider-first-model-picker`,
`governance-tool-approval-reconciliation`, `admin-ui-freeze-diagnostics`
Round 2 (after Round 1's Change 3 lands): `admin-agent-edit-panel-verification`

### COMMANDS TO RUN

```
/opsx:new admin-sw-scheme-safe-caching
/opsx:new admin-agent-model-warning-clarity
/opsx:new admin-agent-provider-first-model-picker
/opsx:new admin-agent-edit-panel-verification
/opsx:new governance-tool-approval-reconciliation
/opsx:new admin-ui-freeze-diagnostics
```

### Trade-offs and scope cuts (explicit, per S-03)

- Changes 5 and 6 are deliberately NOT scoped as fixes yet — the
  assessment's evidence supports two competing explanations for the
  governance gap and zero reproductions for the freeze. Sizing either as a
  confident implementation task would be a caveat collapse.
- The 24-change original plan above is untouched by this supplemental plan;
  none of these 6 changes were part of that scope, and none block or are
  blocked by it.
- No change here proposes a broader Admin console redesign, even though
  Finding 2/3 together suggest the underlying pattern (provider-scoped
  pickers) could be extracted into a shared component used by both the
  Providers page and the Agent editor. That extraction is a reasonable
  follow-up but is out of scope for closing the 4 reported complaints —
  flagging it here rather than silently expanding Change 3's scope.

*Supplemental plan complete. Recommended first change:
`admin-sw-scheme-safe-caching` — smallest, safest, fully-confirmed root
cause, unblocks nothing else but ships a clean win immediately.*
