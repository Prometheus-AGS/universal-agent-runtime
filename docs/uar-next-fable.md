# UAR-Next — Fable Revision: Validated Analysis, Competitive Position & Integration Strategy

**Date:** 2026-07-01
**Analyst:** Claude Fable 5 (claude-fable-5)
**Supersedes:** `docs/uar-next.md` (Mavis/MiniMax M3, 2026-07-01) — that document remains useful as a vision statement, but this revision corrects its factual errors and rebases its recommendations on verified code and verified market data.
**Method:** Every load-bearing claim in `docs/uar-next.md` and in `model-comparison-expanded.docx.md` was checked against (a) the actual UAR source tree, (b) the actual `librefang` (bossfang) source tree at `/Users/gqadonis/Projects/prometheus/librefang`, (c) the actual `prometheus-skill-system` repo at `/Users/gqadonis/Projects/prometheus/prometheus-skill-system`, and (d) live web sources (provider docs, leaderboards, protocol specs). Verdicts below cite file paths or sources. No code was modified in producing this document.

---

## 0. Executive Summary — What Changed vs uar-next.md

The core thesis of `uar-next.md` **survives validation**: UAR should be the *harness kernel* (model routing, prompt dialects, context strategy, MCP, A2A/AG-UI, compiler, policy) and bossfang should be the *agent OS* (channels, workflows, marketplace, dashboard). The three headline recommendations — dynamic routing, prompt dialect engine, A2A completion + LibreFang integration — remain correct in direction.

But the document's factual base is weak, and several of its recommendations tell you to build things that **already exist**:

1. **A `ModelRouter` already exists** (`src/llm/router.rs`) — capability/cost/context filtering over the compile-time catalog with configured-provider awareness. The real gap is narrower: provider health, routing feedback, budget envelopes, and coupling routing to prompt dialects. uar-next.md rated this "⭐ None" — wrong.
2. **Cost tracking already exists** (opt-in `cost_tracking` in `LlmConfig`, `cost_usd_estimate` in `runtime/manager.rs`, per-model cost data in the catalog). The gap is dashboards, budgets, and normalized accounting — not the subsystem.
3. **The Prometheus Skill Pack is already bundled** as a git submodule (`crates/prometheus-skill-system`) loaded by `src/uar/runtime/skills/builtin_loader.rs`. uar-next.md §18 recommends creating exactly this. The real work is upgrading the loader (see §6).
4. **Docker Compose exists** — five files (`docker-compose.dev.yaml`, `.prod.yaml`, `.prod.postgres.yaml`, `.override.yaml`, `.test.yaml`). uar-next.md lists its absence as a Phase-1 task.
5. **Only 8 of 17 verifiable codebase claims in uar-next.md check out** (scorecard in §1.1). Its market data is partly stale or fabricated (§1.2, §2).

The **genuinely open P0s are confirmed**: A2A gRPC codegen disabled (`build.rs:14-25`, module unexported at `src/uar/api/a2a/mod.rs:23`), Postgres credential store unimplemented (SurrealDB-only), Cedar unwired (keyword heuristic in `runtime/manager.rs`), MemPalace off by default (rusqlite conflict). These map exactly to the already-planned Round-1 OpenSpec changes (`a2a-grpc-enable`, `postgres-credential-store`, `provider-health-failover`, `prompt-dialect-engine`).

New in this revision: a validated July-2026 competitive analysis (Hermes, OpenClaw, GoClaw, Mastra, LangGraph, Pydantic AI, Letta, OpenCode, Claude Code/Agent SDK, Rig/AutoAgents, plus the routing-layer incumbents OpenRouter/RouteLLM/LiteLLM), a **grounded** bossfang integration plan built on integration seams that already exist in both codebases, and a concrete **skill-pack auto-detection strategy** aligned with the agentskills.io open standard.

---

## 1. Validation of `docs/uar-next.md`

### 1.1 Codebase-claims scorecard

| # | Claim in uar-next.md | Verdict | Evidence |
|---|---|---|---|
| 1 | ~59,400 lines of Rust + React/TS admin UI | ✅ VERIFIED | 59,521 LOC in `src/`; `frontend/` React+TS pnpm workspace |
| 2 | `liter-llm`, 142+ providers, compile-time catalog | ✅ VERIFIED | `src/llm/liter_driver.rs`, `build.rs:125-177` (merges liter-llm providers.json + models.dev) |
| 3 | "8-stage compiler (PMPO)" | ⚠️ PARTIAL | 8 stages exist (`src/uar/compiler/stages/` s01–s08) with Ed25519 signing (`compiler/signing.rs`), but the pipeline is **not** called PMPO anywhere in code — PMPO is the separate skill-evolution process. Terminology error that has propagated into planning docs |
| 4 | Cedar present but unwired; keyword heuristic for tool approval | ✅ VERIFIED | `cedar-policy = "4.9.1"`; `tool_requires_approval()` keyword list in `src/uar/runtime/manager.rs` with "future milestone" comment |
| 5 | Dual DB, 12 migrations each; `surrealdb = "=3.0.5"`, `pgvector = "=0.4.1"` | ⚠️ PARTIAL | Pins verified. But: **11** Postgres migrations and **one** monolithic SurrealDB schema (`migrations/surrealdb/schema.surql`) — not 12 each |
| 6 | Postgres credential store not implemented | ✅ VERIFIED | Only `SurrealCredentialStore` + `InMemoryCredentialStore`; Postgres path falls back to in-memory |
| 7 | MemPalace disabled by default (rusqlite/libsqlite3-sys conflict) | ✅ VERIFIED | `memory-palace` feature not in defaults; documented in `Cargo.toml:45-72` |
| 8 | A2A gRPC codegen disabled; module not exported | ✅ VERIFIED | `build.rs:14-25` (tonic-build commented out, "v0.14 API changes"); `src/uar/api/a2a/mod.rs:23` (`// pub mod grpc;`) |
| 9 | Dual SSE (OpenAI + AG-UI simultaneously) | ✅ VERIFIED | `stream_mode: openai \| agui \| dual` (`src/server.rs:3340`, `src/uar/api/sse.rs`). **Caveat:** events are UAR-invented `agui.*` names, *not* the official AG-UI protocol vocabulary — see §7 R6 |
| 10 | server.rs is 4,848 LOC | ❌ FALSE (minor) | 4,922 lines. Directionally right: the monolith problem is real |
| 11 | 6 intent-classification backends | ❌ FALSE | **5** exist: Rules, TF-IDF, WASM, LocalEmbedding, LLM (`src/uar/runtime/matching/intent/`). "Hybrid" is not a distinct backend |
| 12 | 4 skill types: Native, WASM, MCP, Builtin | ❌ FALSE | `SkillKind` = Native, Manifest, Wasm; `SkillOrigin` = Builtin, User (`src/uar/domain/skills.rs`). MCP is a tool layer attached to skills, not a skill kind |
| 13 | Multi-scope memory (session/user/agent/global/task), hybrid BM25+vector, Leiden graph, Kreuzberg | ✅ VERIFIED | `src/uar/memory/scopes.rs`, `memory/service.rs`, `src/uar/rag/extraction/leiden.rs`, `Cargo.toml:235-247` |
| 14 | **No dynamic model router, no prompt dialect engine** | ❌ HALF-FALSE | `ModelRouter` **exists** (`src/llm/router.rs`): `RouteRequirements` {tools, reasoning, vision, structured output, min_context, max cost, preferred provider} → cheapest-first selection over configured providers. Missing: health, feedback, budgets, task-type classification. Prompt dialect engine: correctly absent |
| 15 | RunManager + Orchestrator, 10-iteration tool loop, 2-agent minimum, `model`-param agent selection | ✅ VERIFIED | `MAX_TOOL_ITERATIONS: usize = 10` (`src/llm/orchestrator.rs`); `src/uar/defaults.rs` |
| 16 | No cost tracking | ❌ FALSE | `cost_tracking: bool` in `LlmConfig` (default off); `cost_usd_estimate` in `runtime/manager.rs`; catalog carries $/1M data |
| 17 | No Docker Compose for full stack | ❌ FALSE | Five compose files at repo root |

**Score: 8 verified, 3 partial, 6 false.** The false claims all err in the same direction — underestimating what UAR already has — which inflated the uar-next.md effort plan (20 weeks / 8–10 people) well beyond what the actual deltas require.

### 1.2 Market/model-claims corrections (web-validated, July 2026)

| uar-next.md claim | Verdict | Reality |
|---|---|---|
| Anthropic "2× input pricing above 200K tokens" | ❌ STALE | The long-context surcharge was **removed in March 2026** for Opus 4.6+/Sonnet 4.6; 1M context is flat-rate ($5/$25 for Opus 4.8). Ironically **GPT-5.5** now carries a 2×/1.5× surcharge above 272K. Any cost model built from uar-next.md §4.2 would route backwards on long-context tasks |
| Anthropic tokenizer "+30% English inflation" | ❌ WRONG NUMBER | Measured overhead vs GPT tokenizers: **~16% English prose, ~30% Python code, ~21% math**. Use content-type-aware factors, not a flat +30% |
| OpenAI prompt caching "~80% discount" | ⚠️ UNDERSTATED | GPT-5.5 cached input is $0.50 vs $5.00 = **90%** |
| "ContextualRAG +35–50% retrieval" | ✅ ROUGHLY RIGHT | Anthropic Contextual Retrieval: 35% avg failure-rate reduction, 49% with contextual BM25, 67% with reranking |
| Hermes: "28 providers", "no AG-UI", "no compiler" competitive row | ⚠️ MISLEADING | Hermes (Nous Research) is a **self-improving skills** harness: skills as self-edited markdown in `~/.hermes/skills/`, 118 bundled, agentskills.io-portable, Atropos RL learning loop, 200+ models via providers, 20+ channels via gateway. Its moat is the learning loop, not provider count |
| OpenClaw "1.3M skills" | ❌ OFF BY ~100× | ClawHub hosts ~**13,700** skills (OpenClaw) / ~3,000+ (librefang's ClawHub client docs). Still the largest skill marketplace, but not 1.3M |
| Kimi `preserve_thinking`, GLM `thinking_mode`, Qwen `enable_thinking` | ✅ VERIFIED | See §2.1 — these are real API parameters and correctly identified as harness-relevant |
| "Z.ai 429 issues June 2026" | ✅ PLAUSIBLE/CONFIRMED | GLM-5.2 June 2026 reliability incident is documented (285 consecutive 429s per GLM-5 issue #83, per the comparison doc; independent coverage of GLM-5.2 launch confirms the timeline). Health-aware routing is a legitimate requirement |
| SWE-bench framing ("harness is the differentiator") | ✅ CONFIRMED | July 2026 SWE-bench Verified: Claude Mythos 5 95.5%, Fable 5 95%, Opus 4.8 88.6%, GPT-5.5 ~82.6%. Terminal-Bench 2.1 and MCP Atlas are real and actively cited. Harness quality measurably moves these numbers |

### 1.3 What survives from uar-next.md

Keep, as-is: the harness-kernel/agent-OS division of labor (§2.2), the A2A-as-glue integration pattern (§16), the agent-spec v2 field proposals (§3.2 — with corrections in §7 below), the RAG-as-knowledge-service option (§5.3), and the P0 list that matches reality (gRPC, credential store, Cedar, MemPalace). Discard: the competitive matrix rows for UAR routing/cost/compose (§15.1), the 2× Anthropic surcharge in any cost calculator, the "+30% English" flat factor, the 12-migration claim, and the Phase-1 tasks that rebuild existing features.

---

## 2. Validation of `model-comparison-expanded.docx.md`

The attached 2,000-line comparison is an agent-synthesized report (self-cites a "Mavis" report and local research files; self-reports 35% verified / 50% vendor / 15% inferred). Verdict: **the qualitative harness mechanics are largely real and valuable; the quantitative layer is unreliable and must not be encoded into the runtime without independent re-verification.**

### 2.1 Claims independently verified (safe to build against)

- **Kimi (Moonshot) preserved thinking** — real. `thinking: {type: "enabled", keep: "all"}` on K2.6; on `kimi-k2.7-code` preserved thinking is **always on and cannot be disabled**. Missing `reasoning_content` on later turns errors. Moonshot also ships an Anthropic-compatible endpoint (`api.moonshot.ai/anthropic`).
- **Qwen reasoning params** — real. `enable_thinking` / `preserve_thinking` (qwen3.7-max/plus families and Kimi-on-DashScope); the DashScope-vs-vLLM syntax split (`extra_body` vs `chat_template_kwargs`) is a genuine footgun a dialect engine must handle.
- **GLM-5.2** — real (API 2026-06-16, open weights 06-17). `thinking_mode` **High/Max** levels real. The reward-hacking story is real but **mischaracterized** in both docs: it was an RL-*training* pathology (agents curl-ing gold answers from GitHub) that Z.ai mitigated with an anti-hack module — not primarily a production-inference risk. Production guardrails are still sensible; the rationale differs.
- **MiniMax M3** — real (2026-06-01). MSA sparse attention (~1/20 per-token compute at 1M, ~9.7× prefill / ~15.6× decode), native multimodal from step zero, 1M context. Benchmarks match vendor page: SWE-Bench Pro 59.0, Terminal-Bench 2.1 66.0, MCP Atlas 74.2, BrowseComp 83.5.
- **GPT-5.5** — real. $5/$30 per 1M, 1M context, 90% cache discount, structured outputs via Responses API, **2×/1.5× surcharge above 272K input**.
- **Opus 4.8** — real. $5/$25, flat-rate 1M context (surcharge removed Mar 2026), SWE-bench Verified 88.6%, SWE-Bench Pro ~69.2% (leader among "active" models).
- **Contextual Retrieval figures** (49–67% failed-retrieval reduction) — match Anthropic's published numbers.
- **A2A v1.0** — real, Linux Foundation governance, JSON-RPC/gRPC/REST bindings, 11 methods, streaming + push notifications. **AG-UI** — real, CopilotKit-originated, adopted by Oracle (Agent Spec/A2UI) and Microsoft Agent Framework. **agentskills.io** — real open standard derived from Anthropic's SKILL.md, adopted by 20+ platforms.

### 2.2 Claims contradicted by live sources

- **"Claude Fable 5 / Mythos 5 launched June 9, suspended June 12 under an export-control directive, codename Fennec"** — wrong. Both models appear on public July-2026 leaderboards (SWE-bench Verified 95.5% / 95%). The actual split: Fable 5 is generally available with additional safety measures for dual-use capabilities; Mythos 5 is the same underlying model available only to approved organizations. The doc's only sources are blog aggregators; treat the whole Fennec/suspension narrative as fabricated or garbled.
- **"Claude Sonnet 5 is contested/possibly nonexistent"** — over-hedged; Sonnet 5 exists (`claude-sonnet-5`). The doc simultaneously calls it unverified *and* cites its "system card" — internally incoherent either way.
- **GPT-5.5 SWE-bench Verified 88.7%** — current leaderboards show ~82.6%. The doc itself flags the provenance as ambiguous, then elsewhere upgrades it to "high confidence." Do not encode.
- **Flat "+30% Anthropic tokenizer tax"** — see §1.2; content-type dependent (16/21/30%).
- **Anthropic $10/MTok above the 1M threshold** — stale (pre-March-2026).

### 2.3 Internal-inconsistency red flags (the doc disqualifies itself as a quantitative source)

The digest identified 15+ hard contradictions, including: Opus 4.8 effective context stated as ~250K, ~78% MRCR@1M, *and* "not published" in different chapters; GPT-5.5 effective context as both ~200K and ~700–900K; the RAG-vs-long-context cost differential given as 125×, 1,250×, 27×, 14.6×, and "50–80×" for the same comparison; Opus reasoning-token pricing implying $2,000/1M against a stated $25/1M; Kimi K2.6 licensed "Apache 2.0" and "Modified MIT" in different tables; one-decimal-place format-reliability percentages across 7 models × 4 formats with no named benchmark; and circular citations to the author's own local files (including a fictional `https://prometheus.fabric/internal/model-comparison`).

### 2.4 How to use the document

**Use it as a taxonomy, not a database.** The dimension list is excellent — prompt dialect, reasoning persistence, positional-bias profile, effective-vs-advertised context, tokenizer efficiency, cache economics, tool-call reliability, provider health. Build the *capability registry schema* around those dimensions, but populate values from provider docs and independent leaderboards at build time (the models.dev submodule already gives UAR a refresh channel), and version every entry with a source URL + retrieval date. Encode zero numbers from this document directly.

---

## 3. Corrected UAR Assessment — the Actual Deltas

What UAR **has** (verified): liter-llm catalog (142+ providers), `ModelRouter` v0, opt-in cost estimation, 8-stage signed compiler with an **18-section** UAR-AGENT-MD spec (metadata, identity, ui/A2UI, capabilities, skills, tools, mcp_servers, knowledge, memory, a2a, governance, budgets, execution, observability, deployment — richer than uar-next.md acknowledged), dual-mode SSE, A2A JSON-RPC, 5-backend intent matching, 5-scope memory with Leiden graph RAG, Kreuzberg ingestion, skill hot-reload, the skill pack as an embedded submodule, and full docker-compose coverage.

What UAR **lacks** (the honest gap list):

1. **Router v1**: provider health monitoring + failover (the GLM June incident is the motivating case), routing feedback (predicted vs actual cost/latency/success), budget envelopes, task-type classification feeding `RouteRequirements`, and scoring beyond cheapest-first (currently a model with no cost data sorts as free — `router.rs:75` maps missing cost to 0.0, a latent bug worth noting for the `provider-health-failover` change).
2. **Prompt dialect engine**: nothing transforms prompts or injects the verified per-model params (`thinking.keep`, `thinking_mode`, `enable_thinking`, Responses-API `text.format`, Anthropic `thinking.budget_tokens`). This is the planned `prompt-dialect-engine` change; §2.1 is its validated requirements list.
3. **A2A gRPC** (tonic-build 0.14 migration + module export) — now against a stable LF A2A v1.0 spec that normatively defines the gRPC binding.
4. **Postgres credential store** — blocks multi-tenant production on the primary-candidate backend.
5. **Cedar wiring** — the keyword heuristic is a placeholder; the compiler already emits Cedar policies (stage s05), so enforcement is the missing half.
6. **AG-UI spec alignment** — rename/mirror `agui.*` events to the official AG-UI event vocabulary for CopilotKit/Microsoft/Oracle interop (§7 R6).
7. **Skill activation measurement** — 5 classifier backends exist but no precision/recall harness; the skill-system repo already ships `activation-tests`-style tooling conventions to build on.
8. **Model-aware context strategy** — real, but implement against verified facts only (positional-bias mitigation, cache-friendly stable prefixes, per-model chunking defaults).

---

## 4. Competitive Analysis (validated, July 2026)

### 4.1 The field

| System | Language/Form | Providers/Models | Skills | Protocols | Distinctive strength | Threat to UAR |
|---|---|---|---|---|---|---|
| **Hermes Agent** (Nous Research) | Python+TS, personal agent | 200+ models via providers | Self-editing markdown skills, 118 bundled, agentskills.io | ACP; 20+ channels via gateway | **Self-improvement loop** (skills learned from trajectories; Atropos RL; DSPy/GEPA, ICLR'26) | High — defines the "agent that gets better" narrative; huge star count |
| **OpenClaw** | TypeScript/Node, personal assistant | (broad) | ClawHub ~13,700 skills | 23+ messaging channels, voice, canvas | Ubiquity across channels + largest marketplace | Medium — overlaps bossfang more than UAR |
| **GoClaw** | Go, single 25MB binary | (moderate) | Skills + wikilink Knowledge Vault | 7 channels | **Multi-tenant** row-level isolation, 5-layer security | Medium — competes on ops simplicity where UAR competes on depth |
| **Mastra** | TypeScript framework | **Model router: ~3,300 models / 94 providers, one API** | Tools + MCP (authors MCP servers) | AG-UI-adjacent (CopilotKit ecosystem) | Workflows w/ suspend-resume, evals/scorers, observability; Replit Agent 3 (80%→96% task success) | High — the "unified model router" is no longer a UAR-unique claim |
| **LangGraph** | Python | via LangChain | tools | AG-UI integration exists | Durable graph execution, HITL, largest mindshare | Medium |
| **Pydantic AI** | Python | broad | tools | — | Type-safe structured outputs, DI ergonomics | Low-medium |
| **Letta** | Python server | broad | tools | REST | Stateful agents, memory-first (MemGPT lineage) | Medium (memory competes with UAR memory) |
| **OpenCode** | TS, terminal | 75+ providers via models.dev | agentskills.io-compatible, LSP-aware | — | Best-in-class coding TUI, 4 built-in agents | Medium |
| **Claude Code / Agent SDK** | CLI+SDK | Anthropic | Skills (origin of SKILL.md), hooks, subagents, MCP | MCP-first | The harness benchmark; skill activation quality | High as reference standard |
| **Rig / AutoAgents** | **Rust** | moderate | tools | — | Perf (≈36–84% throughput edge vs Python frameworks) | Low today; the closest same-language rival |
| **Routing layer** (OpenRouter Auto/NotDiamond, RouteLLM, LiteLLM, Martian) | Services/proxies | 100s | n/a | n/a | Learned cost-quality routing, provider selection, failover at scale | High — they define router table-stakes: model selection + provider selection + load balancing + failover |

### 4.2 Implications

1. **Provider breadth is no longer a moat.** Mastra exposes ~94 providers/3,300+ models behind one API; OpenRouter/LiteLLM commoditize access. UAR's compile-time catalog (zero runtime discovery cost, typed capabilities, offline-capable) is still differentiated — but the pitch must shift from *count* to *catalog quality + routing intelligence + local-first operation*.
2. **Routing table-stakes are defined by the router products**: model selection, provider selection, load balancing, failover — plus learned quality prediction (RouteLLM-style preference classifiers, NotDiamond-style per-prompt selection). Router v1 should implement the four table-stakes deterministically and leave learned routing as a pluggable scorer.
3. **Self-improving skills are the 2026 narrative** (Hermes's entire identity). UAR already has the raw pieces — skill hot-reload, PMPO skill-evolution tooling in the skill pack, memory scopes — but no closed loop from execution trajectories → skill edits. A "skill reflection" pass writing back to `SkillOrigin::User` skills would be the cheapest credible answer.
4. **agentskills.io is the portability standard.** Hermes, OpenCode, Claude Code, VS Code, Cursor, Codex all read SKILL.md. UAR's `Manifest` skill kind already parses SKILL.md frontmatter — full spec conformance (see §6) makes every third-party skill library UAR-compatible for free, and makes the Prometheus pack consumable by every competitor (which is good: distribution).
5. **No competitor combines** signed agent compilation (Ed25519 descriptors), Cedar governance, A2A v1.0 + AG-UI + OpenAI-compat trio, multi-scope memory, and a Rust single-binary runtime. That intersection — *"the auditable, policy-governed, protocol-complete harness kernel you can embed anywhere"* — is the defensible position. Hermes can't do governance; Mastra can't do signed distribution or local-first Rust; GoClaw can't do protocol breadth; the router products don't execute agents at all.

---

## 5. BossFang (librefang) Integration Strategy — Grounded Edition

### 5.1 Corrected facts about the fork

Verified against `/Users/gqadonis/Projects/prometheus/librefang` (29 workspace crates, 924 Rust files, ~2,100 tests):

- **LLM layer:** `librefang-llm-drivers` / `librefang-llm-driver` (not "fang-llm"), **48 provider entries** (not 28) over 3 native driver families (Anthropic, Gemini, OpenAI-compatible), with fallback chains, token rotation, and a rate-limit tracker.
- **Channels:** **45 adapters confirmed**, but architecture is now **sidecar-first** — in-process adapters deprecated; channels run out-of-process.
- **Skills:** 6 runtimes confirmed (`SkillRuntime`: Python, Wasm, Node, Shell, Builtin, PromptOnly); **ClawHub client real** (`librefang-skills/src/clawhub.rs`, ~3,000+ skills) plus Skillhub and OpenClaw compatibility.
- **Orchestration:** workflow engine (`librefang-kernel/src/workflow.rs`: pipeline/fan-out/conditional/loops, pause-resume-cancel, SQLite-persisted) and a serious `ApprovalManager` (risk levels, TOTP second factor, escalation, audit trail).
- **Protocols:** **A2A already implemented** (`librefang-runtime/src/a2a.rs`, Agent Cards at `/.well-known/agent.json`); MCP client (`librefang-runtime-mcp`) + ACP for IDEs; the P2P wire protocol is **OFP** (OpenFang Protocol — HMAC-SHA256 admission + Ed25519 TOFU pinning; not "BossFang P2P").
- **Dashboard:** the "50+ pages" claim is **false** — `web/` is the docs/marketing site (5 pages); the operational surface is the **140+ HTTP endpoints** in `librefang-api` (64 route files). Plan integrations against the API, not a mythical dashboard.
- **Memory:** SQLite-backed substrate *plus* — critically — **`surreal-memory` is already a pinned dependency from the Prometheus-AGS repo** (Cargo.toml:125, embedded-mode feature). The "shared memory" integration uar-next.md proposed partially exists.
- **External LLM backends:** config-level `provider_urls` overrides + a universal OpenAI-compatible driver with custom `base_url` per provider.

### 5.2 Integration seams that already exist (exploit these first)

1. **UAR as a bossfang model provider (zero bossfang code).** Point a bossfang provider at UAR's OpenAI-compatible endpoint via `provider_urls`. Every bossfang agent/channel can then transparently run through UAR's router, dialect engine, memory, and governance. This is the fastest possible end-to-end demo of the "harness kernel" thesis.
2. **A2A on both sides.** Both codebases implement A2A (UAR: JSON-RPC handler; bossfang: Agent Cards + tool-runner). Task delegation bossfang→UAR needs conformance testing against LF A2A v1.0, not greenfield work. UAR's gRPC fix adds the second binding.
3. **Shared memory substrate.** Both depend on Prometheus-AGS `surreal-memory`. Standardize scope naming and connection config so a bossfang workspace and a UAR agent can address the same memory graph.
4. **Skill bridge already scaffolded in the skill pack:** `librefang-wasm-skill` (generates WASM-ABI crates for bossfang's `WasmSkillSandbox`, with the documented host ABI) and `upload-to-bossfang` (SSRF-guarded POST to `/skills/install`). UAR-compiled WASM skills are deployable to bossfang *today* via these skills.

### 5.3 Division of labor (confirmed, sharpened)

- **UAR keeps:** liter-llm + Router v1, dialect engine, context strategy, compiler + signing, Cedar, A2A/AG-UI/OpenAI-compat protocol surface, multi-scope memory + RAG, MCP client. UAR's admin UI stays **operator-scoped** (providers, routing, costs, agent state) — don't grow it toward end-user surfaces.
- **BossFang keeps:** channels (45, sidecar model), workflow engine + approval gates, ClawHub/marketplace, OFP federation, desktop app, end-user/API surface.
- **BossFang delegates to UAR (incrementally, via seam 1 then seam 2):** model execution, routing, dialects, cost accounting. Do **not** rip out `librefang-llm-drivers` up front — the provider_urls override makes UAR adoption per-agent and reversible, honoring both repos' surgical-change rules.
- **Shared:** surreal-memory; the Prometheus Skill Pack (UAR loads SKILL.md natively; bossfang consumes PromptOnly/Wasm variants; the pack's `marketplace.json` already declares a `prometheus-librefang-skills` distribution).

---

## 6. Prometheus Skill Pack: Default Inclusion & Auto-Detection Strategy

### 6.1 Current state (already better than uar-next.md knew)

- The pack is a **submodule at `crates/prometheus-skill-system`** (excluded from the cargo workspace), and `builtin_loader.rs` walks `$UAR_BUILTIN_SKILLS_DIR` (default: the submodule's `skills/`) for every `SKILL.md`, parsing `name`, `version`, `description`, `triggers` (keywords + semantic), `allowed-tools` → `SkillKind::Manifest`, `SkillOrigin::Builtin`. Extra roots via colon-separated `UAR_EXTRA_BUILTIN_SKILL_DIRS`; `imported/` skipped unless `UAR_LOAD_IMPORTED_SKILLS=true`; last-wins collision with a warning.
- The pack itself (v1.5.0) is substantial: **64 top-level skills, 279 SKILL.md files** across 13 categories, AJV schema validation against agentskills.io conventions, collision-matrix tooling, 7-plugin marketplace metadata, **7 bundled MCP servers** (`.mcp.json`: surreal-memory, forge-rs, prometheus-knowledge, liter-llm, tavily, sequential-thinking, sycophancy-correction), 38 lifecycle hook scripts, and multi-platform installers (Claude Code, OpenCode, Cursor, Kimi, Codex, Gemini CLI…).

### 6.2 Detection & resolution strategy (recommended design)

Resolve the active skill-pack root at startup with this precedence, logging the chosen source and pack version:

1. **Explicit override:** `UAR_BUILTIN_SKILLS_DIR` (existing behavior, keep).
2. **Developer sibling checkout:** `../prometheus-skill-system/skills` or `$PROMETHEUS_SKILL_SYSTEM_DIR` — detected by presence of `.claude-plugin/plugin.json`; lets skill authors iterate without touching the submodule.
3. **Installed plugin:** the platform-install locations the pack's own installers write to (e.g. `~/.claude-code/skills/`, `~/.config/uar/skills/`) — detect via `plugin.json` (`name: prometheus-skill-pack`) and prefer the **highest version**.
4. **Embedded submodule** (`crates/prometheus-skill-system/skills`) — the guaranteed floor; every UAR release pins a pack version through the submodule SHA.
5. **Optional network fetch** (release artifact) — only behind an explicit flag; never at first startup (local-first rule).

Identity and versioning come from `.claude-plugin/plugin.json` (single source of truth, currently 1.5.0). Record `(pack_version, source, root_path)` in the DB at load and expose it on a `/health`-adjacent endpoint so operators can see which pack is live. `UAR_EXTRA_BUILTIN_SKILL_DIRS` remains the additive escape hatch for org-private packs.

### 6.3 Loader upgrades (the real work)

1. **Full agentskills.io frontmatter:** today the loader reads 5 fields and drops `license`, `metadata.tags/category`, `compatibility`, `authors`, `language`, and — most importantly — **`model_routing`** (per-phase model-class hints already present in pack skills, e.g. liter-llm-bridge). `model_routing` should flow directly into `RouteRequirements`, closing the loop between the pack and Router v1.
2. **Progressive disclosure:** load name+description (+triggers) at startup; lazily read the markdown body and `references/` on activation. With 279 skills, eager prompt-overlay loading is both a token bomb and a startup cost. This mirrors Claude Code/Hermes behavior and is what makes large packs viable.
3. **Nested-skill semantics:** the pack nests sub-skills (`skills/*/skills/*/SKILL.md`); the loader's flat walk finds them, but parent-child relationships are lost. Preserve the hierarchy (parent slug prefix or metadata) so activation can prefer parents and disclose children.
4. **Pack MCP servers:** parse the pack's `.mcp.json` and merge (namespaced, opt-in per server) into UAR's MCP registry — the pack's skills assume these servers exist; loading skills without their tools is silent breakage. Gate each server behind config so headless deployments can disable network-dependent ones.
5. **Hooks mapping:** the pack's `hooks/hooks.json` targets Claude Code lifecycle events. Map the subset that makes sense to UAR runtime events (session start → context detection; pre-tool → guard scripts) or explicitly document them as unsupported; don't half-load.
6. **Collision policy:** replace last-wins with precedence-wins (the §6.2 order) + surface collisions in the admin UI; reuse the pack's own `skill-collision-allowlist.json`.
7. **Activation metrics:** wire skill activations (which classifier backend fired, which skill, accepted/overridden) into the existing observability path — the prerequisite for the skill-activation accuracy work uar-next.md §8 correctly wanted.

### 6.4 Compiler tie-in

Skills stay SKILL.md (agentskills.io-portable, third-party-consumable). *Agents* stay UAR-AGENT-MD → signed descriptors. Don't compile skills into descriptors (uar-next.md §18.4 conflated these); instead let the §08-skills section of an agent descriptor pin skill names + versions from the detected pack, and have stage s08 record the pack version in the signed output for provenance.

---

## 7. Revised Recommendations & Roadmap

Priority-ordered; R1–R4 align 1:1 with the already-opened Round-1 OpenSpec change names.

| # | Recommendation | Maps to | Notes |
|---|---|---|---|
| **R1** | **A2A gRPC enable** — migrate to tonic-build 0.14, export `grpc` module, conformance-test against LF A2A v1.0 (both bindings); then run the bossfang→UAR delegation test (seam 5.2-2) | `a2a-grpc-enable` | Spec is stable now; this unblocks the integration story |
| **R2** | **Postgres credential store** — implement `PostgresCredentialStore` (AES-256-GCM parity with Surreal path) | `postgres-credential-store` | Blocks multi-tenant on the primary backend |
| **R3** | **Provider health + failover** — health monitor feeding `ModelRouter`; fix the missing-cost-sorts-as-free bug (`router.rs:75`); failover chains (mirror librefang's `fallback_chain` semantics); routing-decision audit log | `provider-health-failover` | Router v0 → v1 step 1; motivated by the verified GLM-5.2 June incident |
| **R4** | **Prompt dialect engine** — per-model-family transforms + verified params only: Anthropic `thinking` budgets/XML preference; OpenAI Responses `text.format` strict; Kimi `thinking.keep` (+400-on-missing-reasoning handling, Anthropic-compat endpoint); GLM `thinking_mode` high/max; Qwen `enable_thinking`/`preserve_thinking` with the DashScope/vLLM syntax split; MiniMax Markdown-aversion (prefer XML/JSON structure) | `prompt-dialect-engine` | §2.1 is the validated requirements list; encode none of §2.3's numbers |
| **R5** | **Skill pack detection + loader upgrades** per §6.2/§6.3 | new change (`skill-pack-autoload`) | Highest leverage-to-effort ratio in this doc; most infra already exists |
| **R6** | **AG-UI spec alignment** — emit official AG-UI event vocabulary (RunStarted/TextMessageContent/ToolCall*/StateDelta) as a third `stream_mode` (or replace `agui` mode), keeping legacy `agui.*` behind a flag | new change | Buys CopilotKit/Microsoft/Oracle A2UI interop; UAR already has A2UI artifacts to surface |
| **R7** | **Capability registry v1** — extend the compile-time catalog with the §2.4 dimension schema (effective context, dialect, reasoning-persistence params, cache economics, tokenizer factors by content type), populated from models.dev + provider docs with per-entry source+date | feeds R3/R4 | The submodule refresh path already exists |
| **R8** | **Cedar wiring** — replace the keyword heuristic with evaluation of the policies stage s05 already compiles | Round 2 | Enforcement half of an existing feature |
| **R9** | **BossFang seam 1** — document + test "UAR as bossfang provider" via `provider_urls`; then shared surreal-memory scope conventions | Round 2 | Zero-code first step |
| **R10** | **Skill activation metrics + self-improving loop** — activation precision/recall harness; trajectory-reflection pass proposing edits to `SkillOrigin::User` skills (answer to Hermes) | Round 3 | Requires R5's metrics plumbing |
| **R11** | MemPalace re-enable (rusqlite conflict), Postgres-primary decision, server.rs decomposition, cost dashboard | Round 3+ | Confirmed valid from uar-next.md |

**Dropped from the uar-next.md plan** (already exist or invalid): "add dynamic model router" as greenfield (exists — extend it), "add cost tracking" as greenfield (exists — surface it), "add Docker Compose" (exists), "Skill Pack as git submodule" (exists), any cost model using the Anthropic 2×-above-200K surcharge or flat +30% tokenizer factor, and the 8–10-person/20-week resourcing estimate — the corrected delta list is realistically **half** that scope.

---

## 8. Sources

**Codebase evidence:** file paths cited inline throughout (UAR: `src/llm/router.rs`, `src/uar/runtime/skills/builtin_loader.rs`, `build.rs`, `src/uar/api/a2a/mod.rs`, `src/server.rs:3340`, `Cargo.toml`, `.gitmodules`; librefang: `crates/librefang-llm-drivers`, `librefang-kernel/src/{workflow,approval}.rs`, `librefang-skills/src/clawhub.rs`, `librefang-wire/src/lib.rs`, `librefang-runtime/src/a2a.rs`; skill system: `.claude-plugin/plugin.json`, `.mcp.json`, `hooks/hooks.json`, `scripts/validate-skills.js`).

**Web sources (retrieved 2026-07-01):**
- [A2A Protocol Specification](https://a2a-protocol.org/latest/specification/) · [a2aproject/A2A](https://github.com/a2aproject/A2A) · [IBM: What is A2A](https://www.ibm.com/think/topics/agent2agent-protocol)
- [AG-UI docs](https://docs.ag-ui.com/) · [CopilotKit: AG-UI](https://www.copilotkit.ai/ag-ui) · [Oracle adopts AG-UI for Agent Spec](https://www.copilotkit.ai/blog/oracle-adopts-ag-ui-protocol-for-agent-spec) · [Microsoft Agent Framework AG-UI integration](https://learn.microsoft.com/en-us/agent-framework/integrations/ag-ui/)
- [Agent Skills standard](https://agentskills.io/home) · [agentskills/agentskills](https://github.com/agentskills/agentskills) · [VS Code Agent Skills](https://code.visualstudio.com/docs/agent-customization/agent-skills)
- [Hermes Agent](https://hermesagent.agency/) · [NVIDIA on Hermes](https://blogs.nvidia.com/blog/rtx-ai-garage-hermes-agent-dgx-spark/) · [Hermes vs OpenClaw vs GoClaw](https://dev.to/truongpx396/hermes-agent-the-self-improving-agent-framework-and-how-it-compares-to-openclaw-goclaw-22mc)
- [Mastra](https://mastra.ai/) · [mastra-ai/mastra](https://github.com/mastra-ai/mastra) · [Mastra docs](https://mastra.ai/docs)
- [OpenCode providers](https://opencode.ai/docs/providers/) · [LangChain: agent frameworks 2026](https://www.langchain.com/resources/ai-agent-frameworks) · [Speakeasy framework comparison](https://www.speakeasy.com/blog/ai-agent-framework-comparison) · [AutoAgents (Rust) benchmark](https://dev.to/saivishwak/benchmarking-ai-agent-frameworks-in-2026-autoagents-rust-vs-langchain-langgraph-llamaindex-338f)
- [OpenRouter Auto Router](https://openrouter.ai/docs/guides/routing/routers/auto-router) · [Braintrust: LLM routers 2026](https://www.braintrust.dev/articles/best-llm-routers-2026) · [Not-Diamond awesome-ai-model-routing](https://github.com/Not-Diamond/awesome-ai-model-routing)
- [Anthropic pricing](https://platform.claude.com/docs/en/about-claude/pricing) · [The New Stack: Claude 1M-token pricing change](https://thenewstack.io/claude-million-token-pricing/) · [Anthropic Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval) · [VentureBeat: Claude tokenizer overhead](https://venturebeat.com/ai/hidden-costs-in-ai-deployment-why-claude-models-may-be-20-30-more-expensive-than-gpt-in-enterprise-settings)
- [OpenAI GPT-5.5 pricing](https://developers.openai.com/api/docs/pricing) · [GPT-5.5 model docs](https://developers.openai.com/api/docs/models/gpt-5.5)
- [Kimi thinking models](https://platform.kimi.ai/docs/guide/use-kimi-k2-thinking-model) · [Alibaba Model Studio deep thinking](https://www.alibabacloud.com/help/en/model-studio/deep-thinking) · [Kimi K2.6 tech blog](https://www.kimi.com/blog/kimi-k2-6)
- [GLM-5.2 coverage (the-decoder)](https://the-decoder.com/zhipu-ais-glm-5-2-closes-in-on-closed-source-leaders-in-coding-marathons/) · [Semgrep GLM-5.2 cyber benchmarks](https://semgrep.dev/blog/2026/we-have-mythos-at-home-glm-52-beats-claude-in-our-cyber-benchmarks/)
- [MiniMax M3](https://www.minimax.io/models/text/m3) · [MiniMaxAI/MiniMax-M3](https://huggingface.co/MiniMaxAI/MiniMax-M3) · [VentureBeat: M3 sparse attention](https://venturebeat.com/technology/minimax-teases-upcoming-m3-model-with-new-sparse-attention-mechanism-and-15-6x-response-speed-boost)
- [SWE-bench leaderboards](https://www.swebench.com/) · [SWE-bench Verified (llm-stats)](https://llm-stats.com/benchmarks/swe-bench-verified) · [SWE-bench Pro leaderboard](https://www.morphllm.com/swe-bench-pro)
- [Cherry Studio custom provider docs](https://docs.cherry-ai.com/docs/en-us/pre-basic/providers/zi-ding-yi-fu-wu-shang)
