# Assessment: UAR Production-Readiness Gaps

**Phase:** `uar-production-readiness-gaps`
**Date:** 2026-06-02
**Project:** Universal Agent Runtime (`src/` + `frontend/`)
**Assessed by:** kbd-assess + 4 parallel codebase/research subagents
**Assessment discipline:** Sycophancy-corrected — findings are stated bluntly, evidence-cited, and not softened. (Note: the requested "sycophancy correction skill" is **not** an installed Claude skill; the only matches are build artifacts of the `sycophancy-core` Rust crate already vendored into UAR via `Cargo.toml:111`. The honest-assessment *intent* is honored here; the crate itself is a separate runtime guardrail, not a tooling dependency for this report.)

---

## Executive Summary

The headline complaints are all real and all have concrete, traceable root causes. None of them are mysteries:

1. **The "parking lot bug" is a dependency-version problem, not a code problem in UAR.** UAR is pinned to `prometheus_parking_lot` rev `32b481d6` (`Cargo.toml:253`) — a commit that **predates the entire certification-remediation PR #1** that fixed exactly the symptoms reported. The fixes live on `origin/main` HEAD `ebb7c3c`. The "can't shut down / had to `kill`" behavior maps directly to two upstream fixes UAR does not yet have: **CR-01** (`retrieve_async` blocking-thread leak) and **CR-02** (`worker_pool shutdown()` honoring join timeout + detaching wedged workers).

2. **But bumping the crate alone will NOT fix shutdown** — because UAR never actually shuts the pool down. `IngestionWorkerPool::shutdown(self)` takes `self` by value while the pool is stored behind an `Arc` (`server.rs:771, 840`), making it **structurally unreachable dead code**. The server's "graceful shutdown" is a fixed `tokio::time::sleep` that only governs Axum sockets and never touches the worker threads (`server.rs:1105-1142`). Two independent pools are spun up and both are leaked.

3. **Config does not flow env/YAML → datastore → UI reliably.** There are **two unreconciled config-mirroring paths** with a "DB-wins-after-first-boot" rule (`manager.rs:416-419`). After the first boot, editing `UAR_LLM__*` or `config.yaml` has **no effect** on what the providers/models UI shows. Worse, `OPENAI_API_KEY`-style shortcuts are written to a **dead config key** (`config.rs:1060` → `llm._provider_keys.*`, a field that does not exist on `LlmConfig`), so a provider can render as "configured" in the UI while having no usable key.

4. **The two built-in agents are second-class citizens.** `default-agent` and `orchestrator-agent` are code-only constructors (`src/uar/defaults.rs:9, 76`), never persisted. `GET /api/agents` *does* inject them via an `ensure_builtin_agent` shim, so the endpoint is not the bug — but the frontend hydrates them once via REST into an entity graph that a wildcard realtime subscription (`sync.ts {type:"*"}`) can **evict with no re-emit**, because they have no backing DB row. Their visibility is therefore unreliable by construction.

5. **A large slice of the admin UI is a non-functional facade.** The entire **Runtime Console** (4 of 17 admin sections: Cockpit, Runs, Approvals, Protocols) renders exclusively from `Runtime*` entity types that **no backend code ever emits** — they are fed only by a DEV-only `window` replay helper and Vitest fixtures. The Approvals page's Approve/Deny buttons **have no onClick handlers at all** (`runtime-console-page.tsx:357-358`). In production these pages permanently show empty states.

**Honest positioning vs. the market:** UAR's *plumbing* (Rust streaming, MCP tools, provider registry, sessions, worker pool) is competitive with Mastra/VoltAgent at the primitive level. Where it is **materially behind** a "releasable production-ready" bar: no OTel tracing or live debug console, no cancellation/lifecycle hooks wired through tool/agent execution, no durable/resumable workflows, no human-in-the-loop, no evals/guardrails surfaced, and a shutdown story that is currently broken rather than merely incomplete. The gap is closable — Rig proves a Rust runtime can hit the observability bar — but it is real and should not be undersold.

**Complexity:** High · **Risk:** Medium-High (shutdown + config-mirroring touch startup wiring) · **Recommended decomposition:** 6 change-sets (see §7).

---

## Scope & Goals (from the request)

| # | Goal | Source |
|---|---|---|
| G1 | Update `prometheus_parking_lot` reference to latest upstream | "update our reference … via the github link" |
| G2 | Adopt the new parking-lot APIs for graceful shutdown + stuck-task handling | "research ways we can use the new API's … graceful shutdowns, stuck tasks" |
| G3 | Fix the broken shutdown UX (had to `kill` processes) | "we cannot shutdown the processes easily" |
| G4 | Make env/YAML config storage in datastores coherent; UI must reflect actual providers/models | "treatment of environment variable and YAML file data … UI/UX reflects … providers and models specified" |
| G5 | Make built-in agents discoverable and chat-able; enable agent switching | "two built in that do not show up in any lists" |
| G6 | Close the UI-vs-reality gap (too much unimplemented) | "too much in the UI/UX that is not implemented" |
| G7 | Reach harness parity with Mastra/Volt/peers | "study mastra, volt … make sure we are on the same level" |

---

## D1 — `prometheus_parking_lot` version drift & shutdown (G1, G2, G3)

### Current state
- **Pinned rev:** `32b481d6c5694545d35789894f6feecf5ac4ca3e` with `features = ["tokio-runtime"]` (`Cargo.toml:253-255`).
- **Upstream HEAD (origin/main):** `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0` — verified pushed; local clone at `/Users/gqadonis/Projects/prometheus/prometheus-parking-lot-rs`.
- The pinned rev is a valid commit object but is **not in origin/main's recent history** — it predates PR #1.

### New APIs available after the bump (re-exported from crate root, `src/lib.rs:98-108`)
| API | Commit | Solves |
|---|---|---|
| `core::shutdown::{ShutdownHandle, ShutdownPolicy, wait_for_signal}` | CR-03, CR-13 | Signal-driven graceful shutdown with bounded join |
| `core::cancellation::{CancellationToken, CancellationSink}` + bounded handoff + DLQ replay | CR-14 | Cancelling/stuck in-flight tasks; dead-letter recovery |
| `core::hooks::{Hook, LifecycleEvent, AuditSink}` | CR-07 | Lifecycle/audit visibility into task state |
| `WorkerPool::shutdown()` honors join timeout, detaches wedged workers | CR-02 | The "stuck task blocks exit" symptom |
| `retrieve_async` blocking-thread leak fix | CR-01 | The "process won't die, had to `kill`" symptom |
| unified `#[non_exhaustive]` error taxonomy + `TaskMetadata.idempotency_key` | CR-05 | Safer error handling / idempotent retries |
| Mailbox hardening: TTL/expiry, consume-once, idempotency | CR-06 | Reliable result handoff |

### Usage inventory in UAR (`prometheus_parking_lot` is touched in exactly ONE module)
| File:Line | Symbol | Note |
|---|---|---|
| `src/uar/rag/ingestion_worker.rs:15-19` | `WorkerPool`, `WorkerExecutor`, `TaskMetadata`, `PoolError`, `WorkerPoolConfig`, serde keys | only importer |
| `ingestion_worker.rs:80-81` | `execute(&self, job, _meta: TaskMetadata)` | **`_meta` ignored** — no deadline/cancellation consulted |
| `ingestion_worker.rs:347-353` | `WorkerPoolConfig::new().with_worker_count(..).with_max_units(1000)..` | no shutdown handle, no join-timeout |
| `ingestion_worker.rs:377-389` | `TaskMetadata{ deadline_ms: None, mailbox: None, .. }`; `submit_async` | **deadline always None** |
| `ingestion_worker.rs:401` | `pool.retrieve_async(key, timeout)` | wrapper **never called** anywhere |
| `ingestion_worker.rs:405-407` | `fn shutdown(self){ drop(self.pool); }` | **dead code** — unreachable behind `Arc` |
| `server.rs:762-771` | `IngestionWorkerPool::new(..)` → `Arc` | **pool #1** (knowledge-bases router) |
| `server.rs:833-840` | `IngestionWorkerPool::new(..)` → `Arc` | **pool #2** (knowledge alias router) — duplicate |
| `src/uar/api/knowledge.rs:423` | `pool.submit(..).await` | fire-and-forget; key discarded |

`PoolStats`, `core::shutdown`, `core::cancellation`, `core::hooks` are **not used anywhere** (they don't exist in the pinned rev).

### Why shutdown is broken (root cause, blunt)
1. **`shutdown(self)` is unreachable.** The pool is `Arc`-wrapped at construction; you cannot move `self` out of an `Arc`. `shutdown()` is never called from anywhere. → worker threads are abandoned on exit.
2. **The "graceful shutdown" is a sleep.** `shutdown_signal()` (`server.rs:1105-1142`) awaits SIGINT/SIGTERM then `tokio::time::sleep(timeout)` and returns. It holds no pool reference and signals/joins/cancels nothing. It only delays Axum socket close.
3. **Pinned rev lacks the join-timeout/detach logic.** A worker wedged inside `execute()` (Kreuzberg extraction, embedding HTTP, DB I/O at `ingestion_worker.rs:140-222`) blocks join/drop indefinitely → process hangs → `kill` required.
4. **No deadline, no cancellation.** `deadline_ms: None` + ignored `_meta` ⇒ in-flight ingestion cannot be interrupted ("stuck tasks").
5. **Two leaked pools**, each auto-sizing to `num_cpus::get()` threads, doubling abandoned threads.

### Integration points for the new APIs (the actual fix)
- Replace `shutdown_signal()` (`server.rs:1105-1142`) with `wait_for_signal()` + a `ShutdownHandle` driven by a `ShutdownPolicy` carrying `config.server.shutdown_timeout_secs` (`config.rs:155`).
- **Hoist pool ownership**: build ONE pool before router assembly, clone `Arc` into both router states (consolidate `server.rs:762` + `833`), and pass its `ShutdownHandle` into the shutdown sequence. This is the prerequisite for everything else.
- Change `IngestionWorkerPool::shutdown(self)` → `async fn shutdown(&self)` (callable through `Arc`) using the handle's timeout-honoring `shutdown().await`.
- Thread a `CancellationToken` into `DocumentIngestionExecutor::execute` and check it between extraction/chunking/embedding; populate `TaskMetadata.deadline_ms`.
- Optional: register a `Hook`/`AuditSink` at pool construction to emit `LifecycleEvent`s — this also feeds D4 (Runtime Console needs real events).

> **Assess-phase note:** G1 ("update the reference") is a one-line `Cargo.toml` rev bump to `ebb7c3c`, but it is deliberately **left for the execute phase**, because on its own it changes nothing user-visible and may surface compile breaks against the new `#[non_exhaustive]` error taxonomy (CR-05). It must land *with* the wiring work (D1 fix), not before it. Verify whether `shutdown`/`cancellation`/`hooks` are gated behind an extra feature flag once bumped.

---

## D2 — Config → datastore → UI provider/model wiring (G4)

### Config precedence (actual, per `config.rs:836-1090`)
defaults (`:840-910`) → single YAML file (`:912-928`, only one of `--config` / `./config.yaml` / `~/.uar/config.yaml`) → CLI `set_override` (`:929-988`) → targeted env overrides (`:990-1011`) → generic `UAR_*` (`:1020-1025`) → legacy `LLM_*` `set_override` (`:1033-1044`) → provider-key shortcuts (`:1047-1065`) → `api_key_env` indirection (`:1080-1087`).

- **Precedence bug:** CLI `--llm-model` and legacy `LLM_MODEL` both use `set_override` (same priority tier); last-writer (legacy env, `:1033`) wins. This contradicts the documented "CLI > UAR_LLM__ > LLM_*" order in `CLAUDE.md` / `config.rs:1110-1113`.

### Two unreconciled mirroring paths (the core defect)
- **Mechanism A** — `SettingsManager::initialize` (`manager.rs:80-197`) flattens whole `AppConfig` into settings rows (`llm.model` at `:2008`). Drift metadata (`SettingSource`/`is_drift`) is **in-memory only** (`schema.rs:58-72`) → not durable across restart → API-set values can be silently overwritten on next boot.
- **Mechanism B** — provider registry seeding (`manager.rs:398-462`, wired at `server.rs:372-528`): seeds registry from `llm_config` + `config.providers`, persists `provider.{id}` rows **only if absent** (`manager.rs:416-419`), then rehydrates registry from DB (`provider_sync.rs:12-28`).

These two never reconcile. After first boot, **DB wins** and env/YAML edits stop propagating to the provider/model UI.

### Provider/model endpoints (all live, not stubbed)
`GET /api/models` (`server.rs:1389`), `GET /api/catalog` (`server.rs:1463`), `/api/uar/providers*` (`providers.rs:27-38`), `GET /api/uar/resolve-model` (`routes.rs:249` → `manager.rs:1333`). Frontend consumers (`services/{providers,models}-api.ts`, `entities/fetchers/*`, `admin/pages/{providers,models}-page.tsx`) are all live.

### Concrete broken links
1. **"Configured models" = the entire catalog.** `seed_from_llm_config` / `enrich_provider_config` populate `ProviderConfig.models` with *every* catalog model for the provider (`registry.rs:472-497`), not the one the user set. A user who sets `UAR_LLM__MODEL=openai/gpt-4o` sees the whole OpenAI list as "configured." Only `default_model` is correct.
2. **YAML `providers:` array never sets a registry default** (`registry.rs:173-184`); `resolve_default_model` can resolve to a provider the user never configured (falls back to compiled `openai/gpt-4o`).
3. **DB-wins-after-first-boot makes env/YAML edits invisible** to the providers/models UI (`manager.rs:416-419, 451`).
4. **Two unreconciled "default model" representations** — `llm.model` row (Mechanism A) vs `llm.default_provider` + `provider.{id}.default_model` (Mechanism B). The settings page and providers/models pages can disagree. (No frontend reads `llm.model` directly — those rows are effectively orphaned.)
5. **Provider-key shortcuts write to a dead key.** `OPENAI_API_KEY` → `llm._provider_keys.{name}` (`config.rs:1060`), but `LlmConfig` has no such field (`config.rs:1161-1215`) → ignored. `build_client_config` (`config.rs:1129-1139`) only falls back to `LLM_API_KEY`. Result: provider shows `configured` (registry `enabled`) while having **no usable key** — a false-positive "active" state (`provider_catalog_status`, `server.rs:1504+`).
6. **In-memory-only drift metadata** ⇒ UI source/drift indicators unreliable across restarts.

---

## D3 — Built-in agent discovery & switching (G5)

- **The two agents:** `default-agent` (`src/uar/defaults.rs:9`) and `orchestrator-agent` (`defaults.rs:76`). Code-only constructors; **never persisted**.
- **`GET /api/agents`** (`discovery.rs:85`, mounted `server.rs:862`) returns `{runtime_agents, federated_agents}` and **injects both builtins** via `ensure_builtin_agent` (`discovery.rs:90-94, 238-242`). **So the endpoint is NOT the bug** — it returns them.
- **Where they go missing:** frontend hydrates them once via REST into the entity graph (`entities/fetchers/agents.ts:8-15`), but the wildcard realtime subscription (`entities/sync.ts {type:"*"}`) can issue a `replace`/`delete` ChangeSet for `Agent` from the DB-backed bus and **evict the graph-only builtins with no re-emit** (no DB row to replay). Their serialized shape also lacks top-level `name`/`description` (only `metadata.title`, `artifact.rs:5-24`); lists fall back to `metadata?.title ?? id` and do not hard-exclude — so the realtime eviction is the realistic failure vector, not the list rendering.
- **Agent switching is wired but fragile:** selector (`features/chat/agent-selector.tsx`) is **only rendered when `activeThreadId` exists** (`chat-page.tsx:146`) — can't pick an agent before a thread exists. Selection POSTs `{agent_id}` to `/api/uar/sessions/{id}/agent-config` (server stores it, `discovery.rs:464-475`; resolved on next run, `server.rs:3668-3688`). But the chat send body (`chat-stream-store.ts:572-580`) carries **no `agent_id`/`model`** — selection depends entirely on a prior best-effort side-channel POST whose errors are swallowed (`agent-selector.tsx:121`). Any race/failure → silent fallback to `default-agent` (`server.rs:3687`).
- **Robust fix:** seed both builtins into the datastore at startup (a `seed_builtin_agents` alongside `ensure_default_knowledge_base` in `defaults.rs`) so they become normal persisted, realtime-backed entities; render the selector unconditionally; include `agent_id` in the chat request body.

---

## D4 — UI implemented vs. stub (G6)

13 of 17 admin sections + chat are genuinely wired. The gap is concentrated:

| Page | Status | Evidence |
|---|---|---|
| Chat | ✅ Implemented | real SSE `POST /api/chat/completion` |
| Providers / Credentials / Models / Skills / Agents / Tools / Auth / Knowledge / Memory / Compiler / Settings / MCP Health / A2UI Testing | ✅ Implemented | live hooks → real endpoints |
| About | ◻︎ Static by design | marketing cards |
| **Runtime · Cockpit** | ❌ **Dead** | reads `RuntimeRun/Step/ToolCall/...` — zero backend emissions in `src/` |
| **Runtime · Runs** | ❌ **Dead** | always `EmptyRuntimeState` in prod (`runtime-console-page.tsx:291,299`) |
| **Runtime · Approvals** | ❌ **Dead buttons** | Approve/Deny have **no onClick** (`runtime-console-page.tsx:357-358`) |
| **Runtime · Protocols** | ❌ **Dead** | reads `RuntimeAgUiEvent/ModelRouteDecision/A2uiSurface` — never emitted |

The Runtime Console is fed only by a DEV-only `window` replay helper (`main.tsx:16-20`) and Vitest fixtures. **This is the bulk of "UI that isn't implemented."** Fixing it is not a frontend task — it requires the backend to emit real runtime lifecycle events (which the parking-lot `Hook`/`LifecycleEvent` bus from D1 can supply, plus orchestrator run/step/tool-call/approval events).

---

## D5 — Harness parity vs. Mastra / VoltAgent / LangGraph / Vercel AI SDK / Rig (G7)

Synthesized production-readiness checklist (union of mature frameworks), scored against UAR:

| Capability | Bar set by | UAR status |
|---|---|---|
| Central agent registry + lookup-by-id | Mastra, Volt | 🟡 Partial — registry exists; builtins not persisted (D3) |
| REST discovery listing full agent config | Volt `/agents/*` | 🟢 Have `/api/agents` |
| Dynamic per-request agent switching (model/instructions/tools) | Mastra | 🟡 Per-session only, fragile (D3) |
| Multi-agent orchestration / handoffs | Mastra, Volt | 🟡 `orchestrator-agent` exists; depth unverified |
| `provider/model` routing across many providers | Mastra (90+), Volt (30+) | 🟢 Have catalog + registry |
| Gateway / custom gateway support | Mastra | 🔴 Not evident |
| Config surfaced & switchable in UI without code | Volt, Mastra | 🔴 Broken (D2) |
| **OTel/OTLP tracing of every agent+tool+step** | all five | 🔴 **Missing** |
| Prod metrics (tokens/latency/error/cost) linked to traces | Mastra | 🔴 Missing |
| **Live local debug console (WS traces)** | Volt VoltOps | 🔴 Missing (Runtime Console is a dead shell — D4) |
| Prometheus metrics export | Rig | 🟡 openspec `prometheus-metrics` change exists; wiring unverified |
| **Tool/agent lifecycle hooks** (start/end/error/retry/fallback) | Volt | 🔴 Missing in UAR; **available in new parking-lot `hooks` (D1)** |
| **Cancellation / AbortSignal through tools & streams** | Volt, Vercel | 🔴 Missing; **available in new parking-lot `cancellation` (D1)** |
| Graceful suspend/resume from durable state | Mastra, LangGraph | 🔴 Missing |
| Durable execution / checkpointing | LangGraph | 🔴 Missing |
| Resumable streaming (client reconnect) | Volt | 🔴 Missing |
| Human-in-the-loop (pause→approve→resume) | LangGraph, Mastra | 🔴 Missing (Approvals UI is dead — D4) |
| Evals (model-graded/rule/statistical) | Mastra, Volt | 🔴 Missing |
| Guardrails (input/output, prompt-injection) | Mastra, Volt | 🟡 `sycophancy-core` guardrail only; no injection defense |
| MCP tool integration first-class | all TS | 🟢 Have it |
| Small footprint / predictable latency | Rig | 🟢 Rust advantage |
| Multiple deploy targets + scaffolding | Mastra, Volt, LangGraph | 🟡 Docker/k8s/Tauri present; no CLI scaffolder |

**Score: ~5 🟢 / 7 🟡 / 11 🔴.** The 🔴 cluster (observability, cancellation, lifecycle hooks, durability, HITL, evals) is exactly what separates "demo" from "production-ready." Critically, **three of the highest-value 🔴 items (lifecycle hooks, cancellation, a real event source for the console) are unlocked by the D1 parking-lot upgrade** — the single cheapest lever in this assessment.

Sources: mastra.ai/docs, voltagent.dev/docs, docs.langchain.com/oss (LangGraph durable execution), ai-sdk.dev/docs (telemetry/AbortSignal), github.com/0xPlaygrounds/rig.

---

## Root-Cause Synthesis

Three structural root causes explain the majority of symptoms:

1. **Ownership/wiring at startup.** Worker pools and built-in agents are created in the wrong scope (trapped in `Arc` router closures / code-only) so they can neither be shut down nor reliably surfaced. Fix the *ownership graph at boot* and shutdown + agent visibility both improve.
2. **No single source of truth for config.** Two mirroring paths + "DB-wins-after-first-boot" + a dead provider-key code path. Pick one reconciliation model (env/YAML authoritative-on-every-boot, or explicit re-sync) and the provider/model UI becomes truthful.
3. **No real runtime event stream.** The Runtime Console, observability gap, and HITL gap all stem from the backend never emitting lifecycle events. The new parking-lot `Hook`/`LifecycleEvent` bus + orchestrator instrumentation is the shared foundation.

---

## §7 — Recommended gap-closure decomposition (feeds `/kbd-plan`)

Ordered by leverage (highest first). Each is a candidate change-set.

1. **C1 — parking-lot upgrade + real graceful shutdown** (G1, G2, G3). Bump rev `32b481d6` → `ebb7c3c`; hoist + consolidate the two pools to one owned instance; replace `shutdown_signal()` sleep with `wait_for_signal` + `ShutdownHandle`/`ShutdownPolicy`; make `shutdown` `&self`/`Arc`-callable; thread `CancellationToken` + `deadline_ms` into the ingestion executor. *Highest leverage — fixes the hard blocker and unlocks hooks/cancellation for later items.* Verify compile against CR-05 error taxonomy.
2. **C2 — config single-source-of-truth** (G4). Reconcile env/YAML → registry on every boot (or explicit re-sync endpoint); make YAML `providers:` set a default; wire `OPENAI_API_KEY`-style shortcuts into `llm.api_key`; compute provider `configured` from key-presence not just `enabled`; persist source/drift metadata; fix CLI-vs-`LLM_*` precedence.
3. **C3 — persist & robustly surface built-in agents** (G5). `seed_builtin_agents` into the datastore at startup; render the selector unconditionally; include `agent_id` (and `model`) in the chat request body; stop relying on the swallowed side-channel POST.
4. **C4 — real runtime event stream + wire the Runtime Console** (G6, partial G7). Emit `Runtime*` lifecycle events from the orchestrator + the parking-lot `Hook` bus (from C1); implement Approve/Deny handlers (real HITL); delete or gate any panel that cannot be backed.
5. **C5 — observability** (G7). OTel/OTLP tracing of agent+tool+step; confirm/finish the `prometheus-metrics` openspec change; token/latency/cost metrics. (Rig confirms feasible in Rust.)
6. **C6 — parity backlog** (G7, longer horizon). Cancellation/AbortSignal through tool calls + SSE; resumable streaming; then durable/checkpointed workflows + evals/guardrails as separate future phases (likely their own phase, not this one).

**Suggested cut for THIS phase:** C1–C4 (the broken-and-blocking items). C5 partial. C6 explicitly deferred to a follow-on phase to avoid scope blow-out — flag in plan.

---

## Risks & Open Questions

- **R1 — CR-05 breakage:** the new `#[non_exhaustive]` error taxonomy may break UAR's `PoolError` match sites on bump. Mitigate: bump + fix compile in C1's first commit.
- **R2 — feature gating:** confirm `shutdown`/`cancellation`/`hooks` aren't behind an extra cargo feature beyond `tokio-runtime`.
- **R3 — config reconciliation is a behavior change:** "env/YAML authoritative every boot" will overwrite user edits made via the API. Need an explicit product decision: authoritative-config vs. DB-authoritative-with-resync. **This is the one genuine product question — recommend surfacing via /kbd-plan, not guessing.**
- **R4 — Runtime Console scope:** emitting a full `Runtime*` event model is large; C4 may need to ship a minimal real subset (runs + tool calls) and gate the rest rather than fake-complete it.
- **O1 — firecrawl:** the request asked specifically for firecrawl web-search tools; the research subagent used WebSearch/WebFetch (firecrawl MCP tools were not confirmed connected this session). Findings are sound but a firecrawl re-run could deepen the Mastra/Volt source coverage if desired.

---

## Assessment status

- Goals assessed: **7/7** (G1–G7) with file:line evidence.
- Blocking defect identified: **yes** (D1 — shutdown unreachable behind `Arc` + stale crate rev).
- Genuine product decision flagged: **yes** (R3 config authority model).
- Ready for `/kbd-plan`: **yes** — proposed change-sets C1–C6 in §7.
