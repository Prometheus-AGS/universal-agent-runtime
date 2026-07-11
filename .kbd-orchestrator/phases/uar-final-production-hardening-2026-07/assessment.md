# ASSESSMENT: uar-final-production-hardening-2026-07

Project: Universal Agent Runtime
Date: 2026-07-10
Codebase baseline: Preview-quality runtime with recently repaired CI, RAG, and dependency security; broad 1.0 remains blocked by release, version, reproducibility, feature-boundary, and product-claim inconsistencies.
Cross-tool progress: 4 of 9 phase changes complete; 5 pending

## VERDICT AND DISTANCE

**Not ready for broad 1.0/general availability.** A bounded developer preview remains supportable. The repository is materially closer than the supplied 61/100 review because RAG embeddings, main CI, vulnerability remediation, and security/community policy work have landed.

**Current evidence-based readiness: 68/100 (+7).** This is an assessment, not release certification.

| Area | Score | Assessment |
|---|---:|---|
| Architecture/product vision | 9/10 | Strong; kernel and platform distribution remain coupled. |
| Core maturity | 8/10 | Real RAG and broader wiring landed; experimental breadth remains. |
| Testing/CI | 8/10 | Ordinary CI recorded green; release CI still contradicts it. |
| Security | 9/10 | Zero open alerts recorded; private reporting enabled. |
| Reproducibility | 4/10 | Git dependencies and build-time network behavior remain. |
| Release engineering | 4/10 | Stale workflow; no current candidate-tag run. |
| Docs/onboarding | 7/10 | Extensive; public site/config/operations work remains. |
| API/version stability | 4/10 | Cargo 0.1.0 versus 1.0-only security support. |
| Packaging/distribution | 6/10 | Paths exist; matrix, signing, SBOM, provenance unproven. |
| Governance/community | 9/10 | Policy/support/community files now credible. |

## IMPLEMENTATION STATUS

- RAG/local embeddings: **DONE** — FastEmbed replaced zero vectors; progress records 389/389 library, 59/59 integration, ranked retrieval, and BDD 6/6.
- Main CI/dependency security: **DONE** — progress records four push workflows green and zero open Dependabot alerts.
- Release feature matrix: **MISSING** — ordinary CI excludes `model-build` and `memory-palace`; release runs `--all-features -- -D warnings`.
- Frontend release alignment: **MISSING** — CI uses Node 22/pnpm/frontend lockfile; release uses Node 18/Bun/`bun.lockb` assumptions.
- Version/support alignment: **MISSING** — `Cargo.toml` is 0.1.0; SECURITY.md supports 1.0.x and rejects pre-1.0.
- Candidate evidence: **MISSING** — no successful current candidate-tag release evidence set.
- Offline build: **MISSING** — revision-pinned Git dependencies remain; `build.rs` fetches models.dev unless skipped and `model-build` downloads from Hugging Face.
- Runtime/platform modularity: **PARTIAL** — Cedar, sycophancy, Burn, FastEmbed, SurrealDB, document processing, and telemetry remain largely unconditional in one package.
- Capability features/bundles: **PARTIAL** — major conceptual capabilities are not optional Cargo features; no minimal/server-full/desktop-full bundles.
- Persistence optionality: **STUB** — `surreal-backend = []` while SurrealDB is unconditional; no persistence-free/in-memory profile.
- Local-first contract: **PARTIAL** — PGlite thread/message persistence and SSE reconciliation are now documented/implemented; full SurrealDB/Postgres/PGlite/mobile replication and conflict matrix is absent.
- HTML versus React positioning: **PARTIAL** — a separate React frontend exists, but the optional console versus embeddable shell packaging boundary needs explicit support language.
- Tauri/mobile: **STUB** — `tauri = []` is not proof of tested installers or iOS/Android certification.
- WASM: **PARTIAL** — native Wasmtime/WASI sandbox exists; `wasm-runtime` can imply browser-WASM portability, which is not present.
- Tool exposure: **PARTIAL** — per-agent policies exist, but docs still say tools are always on; no stable `disabled|auto|required|allowlist|policy_driven` request contract was found.
- Cedar hard deny: **FAIL** — `GovernanceEngine` produces a denial, but `runtime/manager.rs` explicitly converts governance denial into HITL approval. `Allow`, `RequireApproval`, and non-overridable `Deny` are not distinct end-to-end.
- Security defaults: **PARTIAL** — code says injection/PII are detect-only and Cedar defaults permit-all; SECURITY.md’s “guardrails active” lacks an enforcement/detection table.
- Routing: **PARTIAL** — catalog ranking, configured providers, health cooldown, and cost/context inputs exist; residency/policy/live-rate-limit routing and a formal router taxonomy do not.
- Provider taxonomy: **MISSING** — “142+ providers” remains without tested/partial/untested capabilities, support tier, or last-verified date.
- Embedding abstraction: **PARTIAL** — working local/provider implementations exist, but FastEmbed/tokenizers are unconditional and consumers lack one detachable common profile.
- `memory-palace`: **STUB** — public feature remains, ordinary CI excludes it, and no mature UAR integration contract is evident.
- `model-build`: **FAIL** — remains a Cargo product feature with build-script downloading/regeneration rather than maintainer-only `xtask` tooling.
- Runtime Console: **PARTIAL** — current OpenSpec change is wiring Provider Health/AG-UI and removing unsupported panels; incomplete.
- Config/docs/tests/release: **PARTIAL** — four corresponding KBD changes remain pending.

## CROSS-TOOL PROGRESS

- `fix-embeddings-fastembed`: **DONE** — repaired embedding inference and KB retrieval.
- `green-main-ci`: **DONE** — ordinary CI/live integration/BDD/deploy recorded green.
- `re-remediate-stale-rustsec`: **DONE** — zero open Dependabot alerts recorded; suppressions reduced.
- `security-policy-and-community-files`: **DONE** — security/support/templates/licensing and private reporting verified.
- `runtime-console-wire-or-remove`: **IN PROGRESS** — OpenSpec reports 10/15 tasks, though phase progress still says pending.
- `fix-config-passthroughs`, `docs-site-github-pages`, `test-hardening`, `release-1-0-0`: **PENDING**.
- Blockers reported by executing tools: none.

## SPEC GAP SUMMARY

- No authoritative release-support, feature-maturity, persistence, or provider-tier contract spans the 54 canonical specs.
- Ordinary CI and release validation encode incompatible feature/toolchain/warning policies.
- Cargo features do not correspond to conceptual product capabilities; major platform dependencies are unconditional.
- Provider, mobile, local-first, tools, and security claims exceed explicit tested support contracts.
- Governance conflates non-overridable authorization denial with approval.
- Maintainer tooling (`model-build`) and fragile integration (`memory-palace`) remain public product features.
- Packaging lacks proven offline rebuild, signed checksums, SBOM/provenance, and per-platform install/startup verification.

## BUILD HEALTH

- build check: **UNKNOWN** — attempted `SKIP_FRONTEND_BUILD=1 SKIP_MODEL_BUILD=1 SKIP_CATALOG_BUILD=1 cargo check --locked --features postgres-backend,tauri,wasm-runtime`; sandbox DNS could not resolve `static.crates.io` for uncached crates. Interrupted as environment failure, not source failure.
- ordinary CI: **PASS (recorded)** — KBD progress records all four push workflows green on `e563d68`; dependency checks green after `db71bf1`.
- release check: **FAIL by static contradiction/not executed** — current workflow uses incompatible Node/Bun/all-features policy and stale artifact assumptions.
- known violations: release divergence, networked build, Git dependencies, version-policy contradiction, Cedar override, unsupported maturity claims.
- test coverage: **PARTIAL** — strong backend/integration/BDD structure; platform release, soak/recovery, provider certification, and load-bearing frontend coverage remain incomplete.

## CONSTRAINT CHECK

- AGENTS.md violations: release `--all-features -- -D warnings` does not match the supported feature set or achieved warning baseline. No source/UI code was changed by this assessment; unrelated working-tree files were preserved.
- constraints.md violations: **N/A** — file absent.
- architectural concern: unconditional platform dependencies and deny-to-approval behavior conflict with modularity and least privilege.

## GOAL PROGRESS

- Goal 1, RAG/KB retrieval: **MET**.
- Goal 2, zero open security alerts: **MET based on recorded live checks**; retain the final audit run as release evidence.
- Goal 3, Runtime Console: **PARTIAL** — 10/15 tasks reported.
- Goal 4, production test credibility: **PARTIAL** — ordinary CI/BDD improved; release/platform hardening remains.
- Goal 5, config surface: **NOT MET** — `PORT`/`JWT_REQUIRED` correction pending.
- Goal 6, public docs: **NOT MET** — docs-site change pending.
- Goal 7, all customer blockers: **NOT MET** — release/version/reproducibility/modularity/provider maturity/hard-deny remain.

## PRIORITIZED DISTANCE

### P0 — before any 1.0 candidate

1. Derive `release.yml` from green CI: Node 22, pnpm, explicit supported feature matrix, current artifacts, Windows-native validation.
2. Align Cargo/CLI/docs/images/tag/SECURITY version atomically.
3. Run a candidate tag and retain SHA, reports, SBOM, provenance, signatures, and checksums.
4. Prove offline source builds: snapshot catalog inputs, remove mandatory network, publish/vendor Git dependencies, run `cargo build --locked --offline`.
5. Implement non-overridable Cedar `Deny` distinct from `RequireApproval`.

### P1 — truthful product/support contract

6. Publish stable/optional/experimental/internal features; move `model-build` to maintainer tooling and remove/isolate `memory-palace`.
7. Optionalize heavyweight dependencies/backends and add minimal/server/desktop bundles.
8. Publish provider support tiers and per-capability verification.
9. Clarify React versus HTML shell, Tauri-ready versus certified platforms, native sandbox versus browser WASM, and catalog versus adaptive routing.
10. Publish persistence/sync/conflict and exact security-default matrices.

### P2 — hardening

11. Add platform install/startup/health, cancellation/retry/recovery/soak, outage/rate-limit, non-root, and restore tests.
12. Finish Runtime Console, config, docs, and frontend test changes.
13. Establish enforceable `uar-core`, `uar-protocols`, and `uar-platform` boundaries.

## SYCOPHANCY REVIEW

This assessment rejects broad 1.0 while correcting stale supplied findings: RAG, ordinary CI, dependency alerts, PGlite sync documentation, and routing maturity improved; release/version/reproducibility and feature boundaries remain directly evidenced. The optional correction MCP was unavailable, so no score was stored.

## ASSESSMENT COMPLETE
