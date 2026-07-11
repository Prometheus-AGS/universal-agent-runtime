# PLAN: uar-final-production-hardening-2026-07

Project: Universal Agent Runtime
Date: 2026-07-11
OpenSpec available: YES
Completed changes preserved: 6
Changes remaining: 18
Total phase changes after replan: 24

## Release success criterion

UAR may be tagged `v1.0.0` only when every stable claim has an executable acceptance test, the React frontend obeys the repository's required layering, AG-UI/A2UI profiles are declared and conformant, public documentation matches the implementation, the supported feature/platform matrix is green, offline locked builds work, and immutable release artifacts include checksums, SBOM, provenance, and signatures.

The current `docs-site-github-pages` change remains in progress. The former generic `test-hardening` and `release-1-0-0` changes are superseded by the smaller changes below.

## CHANGE LIST (ordered)

### 7. docs-site-github-pages — Complete the in-flight operational docs site

- Scope: docs | CI
- Depends on: NONE (already 3/12 tasks)
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: HIGH
- Details: Finish installation, configuration, backup/restore, upgrade, troubleshooting, and API content and verify GitHub Pages deployment. Limit this change to site scaffolding and operational documentation; final product-claim reconciliation occurs later after behavior certification.

### 8. establish-react-product-contract — Make React and layer ownership canonical

- Scope: architecture | docs | frontend CI
- Depends on: NONE
- Recommended agent: Roo Code (Architect mode) + Codex
- Est. complexity: M
- Customer value: HIGH
- Details: Add the React-first ADR, frontend architecture contract, route/action/API/spec/test inventory, and product-support matrix skeleton. Add an import/fetch boundary checker with an explicit, shrinking legacy allowlist so new violations cannot enter while existing pages are migrated.

### 9. certify-provider-model-settings-flow — Repair and certify configuration-to-routing

- Scope: React UI | hooks | stores | services | Rust API | tests
- Depends on: establish-react-product-contract
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: CRITICAL
- Details: Migrate Providers, Models, and Settings to Component → Hook → Store → Service → API. Certify provider configuration/default/removal, model catalog/config/default routing, settings schema/save/reload/error/secret behavior, and a real routed request; extend the boundary gate for these domains.

### 10. certify-knowledge-rag-flow — Certify the full knowledge customer journey

- Scope: React UI | knowledge store | services | Rust RAG API | tests
- Depends on: establish-react-product-contract
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: CRITICAL
- Details: Move knowledge service calls out of hooks, then certify create KB → upload → index → ranked search → chat grounding → delete/retry. Cover empty, invalid upload, indexing failure, authorization, and persisted/realtime reconciliation paths.

### 11. certify-agui-chat-flow — Establish one conformant AG-UI ingestion path

- Scope: protocol | Rust events | React adapter/store | chat | tests | docs
- Depends on: establish-react-product-contract
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: CRITICAL
- Details: Declare the supported AG-UI profile, map UAR normalized events, remove deprecated event families, and make Chat and Runtime Console consume one adapter. Certify lifecycle, text, tool calls/results, snapshots/deltas, cancel, reconnect/resume, replay, ordering, deduplication, and visible errors.

### 12. certify-a2ui-react-flow — Build and certify the validated React A2UI path

- Scope: protocol | Rust API | React renderer | A2UI Testing | chat artifacts | tests
- Depends on: establish-react-product-contract, certify-agui-chat-flow
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: HIGH
- Details: Select A2UI v0.9.1 as the GA profile and label v1.0 candidate experimental. Share one validated, allowlisted React renderer between chat and A2UI Testing; migrate the page to hook/store/service layering and certify surface/data updates, progressive rendering, action responses, invalid inputs, and unknown-component rejection.

### 13. certify-runtime-console-governance — Certify Cockpit, Protocols, Runs, and Approvals

- Scope: React UI | runtime store | services | governance | Rust API | tests
- Depends on: establish-react-product-contract, certify-agui-chat-flow
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: HIGH
- Details: Remove direct graph/fetch mutations, use the shared AG-UI/runtime ingestion path, and certify live Provider Health, runs, steps, tools, memory, routing, A2UI surfaces, artifacts, inspection, and error states. Introduce end-to-end `Allow | RequireApproval | Deny`; Cedar `Deny` must never expose an approve action.

### 14. certify-remaining-admin-surfaces — Certify all other advertised React routes

- Scope: React UI | hooks | stores | services | APIs | tests
- Depends on: establish-react-product-contract
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: HIGH
- Details: Vertically migrate and certify Agents, Skills, Tools, MCP Health, Compiler, Memory, Auth, Credentials, Costs, and remaining chat/session components. Each stable action needs success, failure, empty, auth, and realtime behavior; remove or mark experimental any surface that cannot meet the contract.

### 15. close-react-boundary-gate — Eliminate the legacy layering allowlist

- Scope: frontend architecture | CI
- Depends on: certify-provider-model-settings-flow, certify-knowledge-rag-flow, certify-agui-chat-flow, certify-a2ui-react-flow, certify-runtime-console-governance, certify-remaining-admin-surfaces
- Recommended agent: Codex
- Est. complexity: M
- Customer value: MEDIUM
- Details: Remove every remaining direct `fetch`, component→service, component→store-mutation, and hook→service violation from live frontend code. Make the boundary checker blocking with zero allowlisted production violations and document the narrow exceptions for asset loading/transport infrastructure.

### 16. publish-capability-support-matrix — Make provider, feature, persistence, and platform claims truthful

- Scope: product contract | docs | metadata | tests
- Depends on: establish-react-product-contract
- Recommended agent: Roo Code (Architect mode)
- Est. complexity: M
- Customer value: HIGH
- Details: Define stable/optional/experimental/internal Cargo features, provider Tier 1/2/3 capability evidence, PGlite/SurrealDB/Postgres authority and sync rules, catalog versus adaptive routing, tool exposure modes, security default enforcement, Tauri-ready versus certified desktop/mobile, and native WASM sandbox versus browser WASM.

### 17. modularize-release-capabilities — Make the compiled feature model match the product model

- Scope: Cargo | Rust modules | dependencies | CI | docs
- Depends on: publish-capability-support-matrix
- Recommended agent: Codex
- Est. complexity: L
- Customer value: HIGH
- Details: Optionalize heavyweight platform dependencies and persistence backends, add supported bundles (`minimal`, `server-full`, `desktop-full`), move `model-build` to maintainer tooling, and remove/isolate `memory-palace` until supported. Test every stable bundle and enforce valid backend combinations.

### 18. make-build-offline-reproducible — Prove locked offline source builds

- Scope: build.rs | dependency supply chain | catalog/model artifacts | CI | docs
- Depends on: modularize-release-capabilities
- Recommended agent: Codex
- Est. complexity: L
- Customer value: CRITICAL
- Details: Vendor or publish Git dependencies, commit versioned catalog/model snapshots with source date/digest, eliminate mandatory build-time downloads, and add clean-source `cargo build --locked --offline` verification. Catalog/model refresh becomes an explicit maintainer task.

### 19. reconcile-product-documentation — Rewrite current docs from certified behavior

- Scope: README | package metadata | architecture/state/testing/deployment docs | docs site
- Depends on: docs-site-github-pages, close-react-boundary-gate, publish-capability-support-matrix, modularize-release-capabilities
- Recommended agent: Claude Code
- Est. complexity: L
- Customer value: HIGH
- Details: Make React primary everywhere, distinguish AG-UI from A2UI, correct provider/platform/security/persistence claims, align versions/licenses, and archive or banner historical HTMX/no-React material. Add CI truth/link gates so prohibited present-tense claims can only occur in marked historical documents.

### 20. align-release-workflow-platforms — Replace stale release automation with the supported matrix

- Scope: GitHub Actions | packaging | frontend build | Rust build | platforms
- Depends on: modularize-release-capabilities, make-build-offline-reproducible
- Recommended agent: Codex
- Est. complexity: L
- Customer value: CRITICAL
- Details: Derive release validation from ordinary CI using Node 22, pnpm, the authoritative Cargo bundle matrix, and current asset/config paths. Add Linux x86_64/arm64, macOS arm64/x64, and Windows x64 compile/install/startup/health/archive tests for only the platforms declared supported.

### 21. certify-operational-resilience — Prove production lifecycle and failure behavior

- Scope: Rust runtime | persistence | providers | MCP | containers | integration/load tests | runbooks
- Depends on: certify-agui-chat-flow, certify-runtime-console-governance, make-build-offline-reproducible
- Recommended agent: Codex
- Est. complexity: L
- Customer value: CRITICAL
- Details: Add startup/shutdown, cancellation, timeout, retry, provider outage/rate limit, MCP crash/restart, parallel tool-call, reconnect/replay, multi-hour streaming soak, non-root container, backup/restore, and corruption/recovery tests. Define thresholds and retain reports as release artifacts.

### 22. produce-supply-chain-artifacts — Generate signed release evidence

- Scope: release CI | SBOM | checksums | provenance | container images | docs
- Depends on: align-release-workflow-platforms, certify-operational-resilience, reconcile-product-documentation
- Recommended agent: Codex
- Est. complexity: M
- Customer value: HIGH
- Details: Generate CycloneDX/SPDX SBOMs, SHA-256 checksums, keyless signatures, SLSA provenance, signed multi-arch GHCR images, and a machine-readable release manifest tying every artifact to source SHA, tests, and vulnerability reports.

### 23. certify-release-candidate — Run an immutable pre-1.0 release candidate

- Scope: release operations | external-install validation | evidence
- Depends on: produce-supply-chain-artifacts, close-react-boundary-gate
- Recommended agent: Manual + Codex
- Est. complexity: L
- Customer value: CRITICAL
- Details: Cut `v1.0.0-rc.1`, run the real workflow, install artifacts on clean advertised platforms, execute the stable capability matrix and docs instructions, and resolve every failure through new focused OpenSpec changes. Require an immutable release evidence bundle and at least three external installations without checkout-specific knowledge.

### 24. release-1-0-0 — Publish GA only from the certified candidate

- Scope: versioning | release | support policy | announcements
- Depends on: certify-release-candidate
- Recommended agent: Manual + Codex
- Est. complexity: M
- Customer value: CRITICAL
- Details: Align Cargo, npm packages, CLI, image labels, docs, changelog, compatibility policy, SECURITY.md, and tag at 1.0.0. Publish from the unchanged certified commit or rerun the complete certification if the commit differs; verify downloadable artifacts and post-release health.

## EXECUTION ROUND ORDER

- Round 0 (continue): `docs-site-github-pages`
- Round 1: `establish-react-product-contract`
- Round 2 (parallel by code ownership): `certify-provider-model-settings-flow`, `certify-knowledge-rag-flow`, `certify-agui-chat-flow`, `certify-remaining-admin-surfaces`, `publish-capability-support-matrix`
- Round 3: `certify-a2ui-react-flow`, `certify-runtime-console-governance`, `modularize-release-capabilities`
- Round 4: `close-react-boundary-gate`, `make-build-offline-reproducible`
- Round 5: `reconcile-product-documentation`, `align-release-workflow-platforms`, `certify-operational-resilience`
- Round 6: `produce-supply-chain-artifacts`
- Round 7: `certify-release-candidate`
- Round 8: `release-1-0-0`

Parallel execution is allowed only in separate worktrees under `~/.claude/worktrees/` and only when file ownership does not overlap. The five Round-2 changes must coordinate shared frontend infrastructure and `progress.json`; sequence them if worktree merges reveal overlap.

## EXPLICIT TRADE-OFFS AND SCOPE CUTS

- GA targets A2UI v0.9.1; v1.0 candidate remains experimental.
- Mobile is not GA unless platform-specific packaging/tests pass; otherwise document it as experimental.
- Provider catalog presence is not support certification; only evidence-backed tiers are advertised.
- Historical design documents are preserved as records but cannot remain unlabeled product truth.
- No new admin/UI framework. MSW and `@ag-ui/core` require focused adoption spikes and dependency verification before introduction.
- `v1.0.0` is not date-driven. Failure of the RC gate delays GA.

## COMMANDS TO RUN

The existing active command remains:

`/kbd-apply docs-site-github-pages`

Then execute each prepared OpenSpec change with `/kbd-apply <change-id>` in the round order above. Do not use bare `/opsx:apply`; KBD progress reconciliation is required.

## PLAN COMPLETE
