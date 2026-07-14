# Assessment: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

- **Date**: 2026-07-13
- **Assessed by**: claude (six parallel audit streams: docs/SDKs/site, frontend screens, skills system, backend capabilities, RAG pipeline, Mastra research)
- **Scope**: operator-expanded customer-release readiness (goals.md §5–10) plus the original release-evidence track (§1–4)
- **Baseline**: `main` @ `ca655a6` — 24/24 implementation changes from the prior phase complete; consolidated local validation passed

## Executive verdict

**UAR is NOT ready for customer release.** The prior phase's 24/24 counter measured
implementation of the release *pipeline* (certification, supply chain, promotion), and that
holds. This assessment, against the operator's customer-readiness bar, found **3 CRITICAL
security holes (user isolation), 1 CRITICAL legal blocker (SDK licensing), and 2 CRITICAL
product-claim gaps (skills install path, orchestrator delegation)** that must close before
the candidate tag is worth cutting — otherwise `v1.0.0-rc.3` would certify a multi-user
product whose multi-user isolation is broken.

What IS solid: any-agent conversation selection, local-first architecture (PGlite +
service worker + SSE entity sync), intent classification and skill activation, context
management, memory scope model, AG-UI core vocabulary, JWT middleware, and a
video-proof-capable BDD harness — the machinery the operator's validation plan needs
already exists.

## CRITICAL findings (release blockers)

| # | Finding | Evidence |
|---|---------|----------|
| C1 | **Sessions/threads not user-scoped** — any authenticated user can read any user's conversations; thread lookup is by id only, `_user_id` is a dead field | `src/session/thread.rs:22-46`, `src/uar/persistence/providers/postgres.rs:38-73`, `encrypted.rs:86` |
| C2 | **Legacy memory REST IDOR** — `/api/memory` takes `user_id` from request body/query, ignoring the JWT-derived `UserContext`; any caller reads/writes any user's memories | `src/uar/api/memory.rs:23-86` |
| C3 | **Knowledge bases are global** — any user lists/searches/deletes any KB; chat retrieval silently falls back to searching ALL KBs when configured names don't resolve | `postgres.rs:446-448`, `src/uar/api/knowledge.rs:174-179`, `src/uar/runtime/manager.rs:748-753` |
| C4 | **SDK license conflict** — sdks declare MIT / MIT-or-Apache-2.0 against an AGPL-3.0-only runtime; plus placeholder authorship, zero tests, no CI builds, ~15% API coverage, no streaming in TS/Python, unpublishable path deps | `sdks/rust/Cargo.toml`, `sdks/typescript/`, `sdks/python/` |
| C5 | **No customer install path for the prometheus skill system** — network fetch explicitly unimplemented; no installer/toolchain bootstrap; submodule URL is SSH-only | `src/uar/runtime/skills/pack_detection.rs:18-21`, `.gitmodules` |
| C6 | **Orchestrator delegation dormant** — `orchestrator-agent` is a cosmetic clone of default; AgentNode/RouterNode graph reachable only via `with_agent_graph` which has zero callers | `src/uar/defaults.rs:76-82`, `src/uar/runtime/graph/nodes/`, `manager.rs:492` |

## HIGH findings

| # | Finding | Evidence |
|---|---------|----------|
| H1 | RAG retrieval emits NO SSE events — KB hits pasted into system prompt invisibly; `Citation` fires only from provider-emitted citations. Operator requirement "KB hit chunks shown in UI/UX" fails today | `manager.rs:764-768`, `manager.rs:1449`, `src/uar/api/sse.rs:497` |
| H2 | Skill scope semantics vs requirement: per-agent scope is an in-memory (restart-lossy) inverted allowlist; builtin global-disable overwritten on restart; backend has no conversation scope (frontend session-config toggles exist — see O1) | `service.rs:105,582-585`, `server.rs:412-424`, `capability-toggles.tsx:159-271` |
| H3 | Skill delete guard too narrow: only `origin === "builtin"` protected; non-builtin pack skills deletable in UI and API. API omits `origin` from `SkillResponse`, so the UI cannot even gate builtins correctly | `api/skills.rs:52-64`, `skills-page.tsx:248-306` |
| H4 | JWT hardening: default fallback secret `"fallback_secret_change_in_production"`, HS256 validates only signature+exp (no iss/aud/nbf), everything collapses to `anonymous` when disabled. Minting exists (`/api/uar/auth/exchange`, API-key→1h JWT) — operator's "JWTs minted using a JWT secret" is supported | `config.rs:997-1000`, `middleware.rs:45`, `api/auth.rs:123-148` |
| H5 | README/docs not customer-grade: zero mermaid, one unexplained fabric paragraph, no SDK/skills/Docker/K8s sections, no docs-site link; site has 7 pages, no SDK/skills/deployment guides, `onBrokenLinks:'warn'`; OpenAPI hardcoded `0.1.0` and covers a fraction of routes | `README.md`, `website/`, `src/uar/api/openapi.rs` |
| H6 | Ingestion status bug: documents stay `"pending"` after successful indexing (SurrealDB 3.0.5 rejects `type::thing`) | documented `tests/integration/live/baseline_cases.rs:371-380` |
| H7 | Memory storage round-trip untested (sole test `#[ignore]`d); isolation tests exist only for credentials | `baseline_cases.rs:282-288` |

## MEDIUM findings

- M1: AG-UI missing official events STATE_SNAPSHOT, MESSAGES_SNAPSHOT, RAW; `ToolStart`→TOOL_CALL_END remap quirk (`sse.rs:533-546`).
- M2: KB→agent assignment by name (no FK); chat path uses basic top-3/0.7 retrieval, not the hardened `RagRetrievalPipeline` used by `/search`.
- M3: Local threads never sync to server (PGlite-only transcripts); agent selection not restored per-thread on reload (`agent-selector.tsx:69`).
- M4: `SkillService` LLM matching algorithm is a keyword-fallback stub (`service.rs:437-441`); no Rust integration test for classify→overlay-injection (BDD feature covers it browser-level).
- M5: Context manager hardcodes 128k token limit at call site; skill overlays not separately budgeted (`manager.rs:990`).
- M6: No BDD coverage of any admin screen (6 features are chat-only); `/about`, A2UI Testing untested.
- M7: Provider-count inconsistency (site "142+" vs README "269"); repo-root clutter (`output*.txt`, `u00261`, `TEST_EXECUTION_REPORT.md`).
- M8: Skills/agents/settings tables global — acceptable if declared shared-admin resources, but must be an explicit decision (`postgres.rs:136,335`, `surreal.rs:646-698`).

## Open questions for /kbd-plan

- **O1**: Do the frontend per-conversation capability toggles actually gate backend skill activation (session agent-config consumed in the run loop), or are they cosmetic? Backend audit found no conversation scope in `SkillService`; frontend audit found persisted toggles. Needs a live trace before H2 remediation is scoped.
- **O2**: `chat-kb-retrieval.feature` exists and presumably passes — against the stub LLM, what does it actually assert, given H1 says retrieval emits no events? Verify it isn't asserting a vacuous path.
- **O3**: Is multi-tenancy in scope for 1.0 (fix C1–C3) or is 1.0 declared single-tenant/BossFang-sidecar-only (document the boundary, defer isolation)? This decision reshapes the critical list — as a single-user sidecar behind flint-gate, C1–C3 may be accepted-and-documented rather than fixed. **Operator decision required.**
- **O4**: Are skills/agents/settings intended as shared admin-level resources (M8)?

## What already supports the operator's validation plan

- **Screen inventory ready**: 20 screens catalogued (chat, about, 18 admin surfaces: runtime cockpit, runs, approvals, protocols, providers, credentials, models, knowledge, memory, agents, skills, tools, api-keys, cost, settings, mcp-health, compiler, a2ui-testing) with routes/files/stores — see frontend audit for the full table.
- **Any-agent conversations**: implemented; AgentSelector lists all runtime+federated agents, persists per-session (`services/agents-api.ts`, `agent-selector.tsx:19,43`).
- **Video proof**: `tests/bdd/` playwright-bdd with `video:'on'`, cucumber JSON wired to the bdd-video-proof certification bundler, runs against the real binary (`tests/bdd/playwright.config.ts:47-54`).
- **Intent/skills/context**: 6 pluggable classifiers, run-loop activation with `SkillActivated` events, token-budget + summarization context management — implemented with unit + BDD coverage.
- **Memory**: 5 scopes (Global/Agent/User/Session/Task) with recall waterfall and MemoryRecall/MemoryMutation SSE events (needs H7 test coverage).
- **AG-UI**: core official vocabulary emitted + golden schema tests (needs M1 for full parity).

## Mastra comparison (summary)

Mastra (Apache-2.0 + paid EE, ~26k stars, TS) ships Studio: agent chat playground with
per-chat model/param switching, workflow graph visualization, trace-first debugging,
evals (scorers/datasets/experiments), tool testing, memory inspection. **UAR's structural
advantages**: JWT multi-user without a paid tier, first-party end-user UI (Mastra has
none), KB/RAG admin UI, local-first, AG-UI native. **Match-worthy gaps**: inline per-turn
trace view in chat (Mastra's most-praised feature), in-UI model/param switching mid-chat,
minimal evals panel. **Consciously skip**: workflow graph viz (no comparable UAR engine).
Positioning: UAR's app is simultaneously dev playground, admin console, and end-user
product — Mastra gates that behind EE licensing and DIY frontend work.

## Certification-track status (goals §1–4, unchanged)

Pipeline implementation complete (prior phase). Blocked only by the decision above:
cutting `v1.0.0-rc.3` before resolving C1–C6 (or explicitly descoping via O3) would
certify a candidate that fails its own external-adopter scenario. Recommendation:
resolve O3 first; fix the resulting critical list; then run soak → candidate → installs →
GA exactly as goals §1–4 specify.

## Recommended change list seed for /kbd-plan

1. `decide-tenancy-boundary-1-0` (O3 — operator decision, reshapes everything below)
2. `fix-user-isolation-sessions-memory-kb` (C1, C2, C3) — or `document-single-tenant-boundary`
3. `fix-skills-scope-semantics` (H2, H3, O1, + admin UI origin bug)
4. `ship-skill-pack-install-path` (C5)
5. `resolve-sdk-licensing-and-scope` (C4 — fix or withdraw SDKs from 1.0)
6. `emit-rag-retrieval-events` (H1, + H6 status bug, M2)
7. `wire-orchestrator-delegation-or-descope` (C6 — wire the graph or remove the stub agent)
8. `harden-jwt-defaults` (H4)
9. `rewrite-readme-and-docs-site` (H5, M7 — mermaid architecture/flow/scenario + fabric diagrams)
10. `screen-by-screen-validation-with-video` (M6 + operator's live validation plan; needs 2–8 landed first)
11. Certification track (goals §1–4) last, unchanged.
