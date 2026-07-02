# Assessment — uar-next-harness

Phase goal: implement **every** recommendation in `docs/uar-next.md` to make UAR
the world's best agent harness. This assessment verifies each §17 (Phases 1–5)
and §18 recommendation against the **committed codebase** (per the lesson from
`gate-activation-and-security-cleanup`: verify against committed state, not the
dirty working tree).

## Headline finding: the source doc is materially stale

`docs/uar-next.md` is a forward-looking analysis dated **2026-07-01** (authored by
an external model, "Mavis / MiniMax M3"). Several of its "Critical" §14 gaps were
**already closed** by prior KBD phases (`uar-harness-parity`, `uar-production-
readiness-gaps`, the eval phases) and are contradicted by the current code:

| Doc claim (§14/§15/§19) | Actual state | Evidence |
|---|---|---|
| "No dynamic model router" (#1 gap, P0) | **Router exists** (capability + cost); only health-driven selection missing | `src/llm/router.rs`, `src/llm/catalog.rs:203-244` |
| "Cedar unwired / keyword heuristic" (P1) | **Cedar wired into the tool loop** (OR'd with heuristic) | `src/server.rs:445`, `src/uar/runtime/manager.rs:939-945`, `governance/engine.rs:141-154` |
| H3 "emit-runtime-step-events never built" (carry-over) | **Built + wired end-to-end** (backend emit → SSE → frontend console) | `src/normalized.rs:196`, `orchestrator.rs:393`, `manager.rs:1207`, `sse.rs:529-539`, `runtime-console-page.tsx` |
| H8 sandbox + MCP-status recorders "dead" (carry-over) | **All recorders are called** | `metrics.rs:164-214` ← `orchestrator.rs:733-789`, `mcp/registry.rs:91-128` |
| Skill schema transformation (P1, Phase 2) | **Already exists** (OpenAI JSON + capability-driven XML injection) | `native_skill.rs:135`, `tool_normalizer.rs`, `xml_tool_injector.rs:17` |
| "No Docker Compose full stack" (P0) | **Present** (app + pgvector + surreal + redis) | `docker-compose.dev.yaml`, `docker-compose.prod.postgres.yaml` |
| Web UI "below competitors / not S-tier" | Provider config + 35-category settings + Runtime Console are **production-grade** | `providers-page.tsx`, `settings-page.tsx` (3336 LOC), `runtime-console-page.tsx` |

**Implication for planning:** do NOT treat §17 as a fresh 20-week greenfield plan.
The real remaining surface is a focused set of genuine gaps plus completion work
on partially-done items. Planning must scope to what's actually missing.

---

## Verdicts by goal group

Legend: **DONE** / **PARTIAL** (exists, needs completion) / **MISSING** / **DECISION** (needs product call) / **OPERATOR** (human-only).

### G1 — Foundation (§17 Phase 1)

| # | Item | Verdict | Note / evidence |
|---|------|---------|-----------------|
| G1.1 | A2A gRPC transport | **MISSING** | Full tonic impl exists but `pub mod grpc` commented out (`a2a/mod.rs:21-23`); proto codegen disabled (`build.rs:16-24`); `include_proto!` would not compile. tonic/prost/tonic-build all pinned 0.14. |
| G1.2 | Postgres credential store | **MISSING** | AES-256-GCM + `SurrealCredentialStore` done (`credentials/encryption.rs`, `store.rs`); Postgres branch explicitly falls back to in-memory (`server.rs:188-190`). |
| G1.3 | Dynamic model router | **PARTIAL** | Router + capability registry + cost calc DONE (`router.rs`, `catalog.rs`); **health-monitor-driven selection MISSING** (failover lives in orchestrator, not routing — see G2.7). |
| G1.4 | Prompt dialect engine | **MISSING** | Zero `dialect`/`PromptDialectEngine` in `src`. No XML/JSON/Markdown/GLM per-model prompt transformation. (Note: `xml_tool_injector` handles *tools*, not prompt dialect.) |
| G1.5 | MemPalace fix | **PARTIAL/DECISION** | `memory-palace` feature off by default; no active rusqlite conflict in current `Cargo.lock` (mempalace-core now uses fastembed). Enabling is a decision, not a blocked fix. |
| G1.6 | Unpin dependencies | **DECISION (won't-do as written)** | `surrealdb "=3.0.5"`, `pgvector "=0.4.1"` pins are **intentional** (comments: client-version alignment; avoid sqlx 0.9 dup). Doc's "unpin" recommendation would reintroduce the exact breakage the pins prevent. Recommend: keep pins, document rationale; reject the doc item. |
| G1.7 | Docker Compose full stack | **DONE** | `docker-compose.dev.yaml` (app+postgres/pgvector+surreal+redis) and prod variants. |

### G2 — Intelligence (§17 Phase 2)

| # | Item | Verdict | Note / evidence |
|---|------|---------|-----------------|
| G2.1 | Cedar in tool loop | **DONE** | `server.rs:445` `.with_governance_engine`; tool gate OR's Cedar `is_tool_allowed` with heuristic (`manager.rs:939-945`). Stale "will be replaced" comment at `manager.rs:261` can be cleaned. |
| G2.2 | Per-model context strategy | **PARTIAL** | Token-window-aware trimming (`runtime/context/manager.rs:39-42`); strategies are global, count/position-based (`context/strategy.rs`). **Model-aware chunking/placement/compression MISSING**; `Summarize`/`Hierarchical` fall back to sliding-window. |
| G2.3 | Cost tracking | **PARTIAL** | Per-model cost calc + `uar_llm_cost_usd` metric + per-run `cost_usd_estimate` DONE (`catalog.rs:244`, `metrics.rs:62-67`). **Per-task spend + budget alerts MISSING**; cache-read cost not accumulated (`manager.rs:1424`). |
| G2.4 | Skill activation metrics | **MISSING** | No precision/recall metric per skill/model. Only prose ("high precision, low recall", `intent/rules.rs:3`). |
| G2.5 | Skill schema transformation | **DONE** | `openai_tools_json()` + `ToolNormalizerDriver` per `ModelCapabilityProfile` + XML injection for non-native models. |
| G2.6 | Extract RAG to Knowledge Service | **PARTIAL** | RAG in-process (`src/uar/rag/`: ingest, chunking, retrieval, leiden). Self-labeled MVP/naive. **No query decomposition / verification / audit; not a separate MCP service.** Decision: extract-to-service vs harden-in-process (§5.3 vs §5.4). |
| G2.7 | Provider health monitoring + failover | **PARTIAL** | `with_failover` API exists but **has no callers**; `health_check_secs` config field is **never consumed** (no monitor loop). Failover is error-driven, not health-driven. |
| G2.8 | Model capability registry | **PARTIAL** | `ModelCapabilityRegistry` (tool-tier + feature flags) + pricing/context catalog DONE. **No benchmark/leaderboard data** and no import mechanism. |

### G3 — UX & Integration (§17 Phase 3 + §18)

| # | Item | Verdict | Note / evidence |
|---|------|---------|-----------------|
| G3.1 | Web UI provider/settings config | **DONE** | `providers-page.tsx` (Cherry-Studio-style), `settings-page.tsx` (35 nav categories), `credentials-page.tsx`. |
| G3.2 | Model comparison dashboard | **PARTIAL** | `models-page.tsx` is a filterable catalog with cost+capabilities; **no side-by-side selection, no benchmark scores.** |
| G3.3 | Real-time agent state visualization | **DONE** | Runtime Console: timeline, tool calls, memory, AG-UI stream, model-routing decisions; backed by real step events. |
| G3.4 | Cost tracking dashboard | **MISSING** | Cost shown per-model in catalog + config knobs only; no spend-aggregation/budget-alert dashboard. (Depends on G2.3 backend.) |
| G3.5 | LibreFang integration (A2A/AG-UI/shared router) | **MISSING** | No LibreFang integration code. A2A (`src/uar/api/a2a/`) and AG-UI (internal chat streaming) exist **parallel-but-separate**; A2A gRPC disabled (see G1.1). External-team dependency. |
| G3.6 | Prometheus Skill Pack bundling (§18) | **PARTIAL** | PMPO compiler + Ed25519-signed descriptors + `crates/prometheus-skill-system` submodule DONE; **no bundled/version-pinned pack, no lazy-load, no `uar skill update`.** |

### G4 — Specification & Distribution (§17 Phase 4)

| # | Item | Verdict | Note / evidence |
|---|------|---------|-----------------|
| G4.1 | Agent spec v2.0 fields | **MISSING** | Spec is v1.1 (`docs/agents/AGENTS_SPEC_RFC.md`, IR `compiler/ir.rs`). None of `model_requirements`, `prompt_dialect`, `rag_configuration`, `api_harness` exist; `context_strategy` exists only as unrelated runtime config. |
| G4.2 | Update PMPO compiler for v2.0 | **MISSING** (compiler itself DONE) | 8-stage pipeline exists (`compiler/pipeline.rs`, `stages/s01..s08`); `s01_frontmatter` doesn't handle new fields. Blocked on G4.1. |
| G4.3 | Conformance testing | **MISSING** | Zero conformance harness; RFC §23 defines criteria, nothing validates them. |
| G4.4 | Agent template library | **MISSING** | No `*.agent.md` templates for coding/vision/terminal. |
| G4.5 | Documentation overhaul | **PARTIAL** | ~60 docs, mostly 2026-05 vintage; stale vs routing/dialect architecture. |
| G4.6 | Production deployment guide | **PARTIAL** | Strong `k8s/` + `helm/` + Dockerfiles; **no consolidated deploy guide**; `docs/ci-gke-deploy-secrets.md` stale (CI moved to AKS). |
| G4.7 | Benchmark suite (skill/routing/context) | **PARTIAL** | Generic eval harness + CI gate DONE (`evals/starter.yaml`, scorers, two-tier gate); **no targeted skill-activation / routing-accuracy / context-efficiency suites.** |

### G5 — Polish & Release (§17 Phase 5)

Not yet assessable in detail — depends on G1–G4 landing. Items: hot-path perf
(router/context), security audit (prompt-injection/RBAC), load test (1000+ agents),
docs finalization, RC→GA. `server.rs` monolith (4,848 LOC, §14) is a P2 refactor
candidate under this group.

### Carry-over & hygiene (from prior reflections)

| Item | Verdict | Note |
|---|---------|------|
| Dirty working tree | **PARTIAL / needs commit** | Uncommitted work is **real features**: dual-stack companion listener (`server.rs`) + multiplexed `/api/live` SSE (`live.rs`, fixes 6-connection HTTP/1.1 exhaustion) + shared-EventSource frontend adapter. Should be reviewed + committed, not stashed away. |
| Eval gate activation | **OPERATOR** | `eval-nightly.yml` uses `--require-baseline`; `evals/results/starter.baseline.json` **not committed** → gate fails-loud by design until seeded. Human-only (needs `UAR_LLM__API_KEY` secret + `workflow_dispatch update_baseline=true`). |
| Artifact-refiner QA gate automation | **PARTIAL** | Defined as an orchestrator skill step; **no committed script/hook/CI wiring** in the project. Carried 3 phases. |
| Formatting | **MISSING (quick fix)** | `cargo fmt` needed on `src/llm/registry.rs`, `src/uar/api/live.rs`, `src/uar/api/routes.rs`. |

---

## Tally

- **DONE (verify-only):** G1.7, G2.1, G2.5, G3.1, G3.3, H3, H8, PMPO compiler core, generic eval harness — **9 items already satisfied**; several were listed as "critical gaps" in the doc.
- **PARTIAL (completion work):** G1.3, G1.5, G2.2, G2.3, G2.6, G2.7, G2.8, G3.2, G3.6, G4.2, G4.5, G4.6, G4.7, artifact-refiner — **14 items**.
- **MISSING (net-new):** G1.1, G1.2, G1.4, G2.4, G3.4, G3.5, G4.1, G4.3, G4.4 — **9 items**.
- **DECISION / OPERATOR / hygiene:** G1.6 (unpin — recommend reject), G1.5 (enable MemPalace?), G2.6 (extract-vs-harden RAG), eval-baseline (operator), working-tree commit, fmt.

## Open questions for planning (→ analyze/plan)

1. **Scope discipline.** The doc's 20-week/8–10-person plan assumes a greenfield that
   doesn't match reality. Plan should sequence only genuine gaps + completions, not
   re-do done work. Confirm the phase targets "close every *real* gap" not "execute
   every doc row verbatim."
2. **Three headline items (§19) reality-check.** #1 model routing is ~80% done
   (needs health-driven selection); #2 dialect engine is the biggest true net-new
   backend gap; #3 A2A gRPC + LibreFang is partly an **external-team dependency**
   (LibreFang side). How much of #3 is in-scope for this repo?
2b. **Dialect engine vs. existing tool normalizer.** `tool_normalizer`/`xml_tool_injector`
   already do per-capability tool formatting. Is the "prompt dialect engine" net-new,
   or an extension of that subsystem? (Affects G1.4 sizing.)
3. **RAG (G2.6):** extract to a standalone Knowledge Service MCP (§5.3, large) or
   harden in-process with decomposition/verification/audit (§5.4, smaller)? Product call.
4. **Dep pins (G1.6):** confirm we **reject** the doc's unpin recommendation (pins are
   deliberate and load-bearing) and instead just document rationale.
5. **Spec v2.0 (G4.1) sequencing:** it gates G4.2/G4.3/G4.4 and conceptually depends on
   G1.4 (dialect) + G1.3 (model_requirements) existing to declare against. Likely a
   later sub-phase.
6. **Nesting:** given ~23 partial/missing items across 5 groups, this should spawn
   nested child phases (`/kbd-new-child`) — recommend starting with a **G1 foundation**
   child (A2A gRPC, Postgres creds, dialect engine, router health) since it unblocks
   the most downstream work.

Suggested first-child ordering: **A2A gRPC enablement** and **prompt dialect engine**
are the highest-leverage net-new backend items; **Postgres credential store** is a
contained P0; **router health/failover wiring** completes an ~80%-done feature by
connecting the already-existing `with_failover` + `health_check_secs` pieces.
