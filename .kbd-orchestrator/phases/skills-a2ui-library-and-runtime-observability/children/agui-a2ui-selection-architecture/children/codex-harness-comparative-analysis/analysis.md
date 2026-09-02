# Analysis — codex-harness-comparative-analysis

Date: 2026-09-02. Mode: stack specified (Rust, tokio, axum, liter-llm, rmcp, Cedar). Input: `assessment.md` and its `evidence/` files. Output: build-versus-adopt calls per gap, a candidate set in `library-candidates.json`, and the open questions the spec and plan stages must answer.

Citation convention is the same as the assessment: `src/`, `tests/`, `Cargo.toml`, `versions.toml` are this repository at `dce44e78`; `core/`, `tools/`, `ext/`, `utils/`, `context-fragments/`, `sandboxing/`, `codex-mcp/` are the Codex CLI workspace at `986ff1cc`, licensed Apache-2.0 (`/Users/gqadonis/Projects/references/codex/LICENSE`). Registry paths are the installed crate sources under `~/.cargo/registry/src/`.

## Research record

| Tier | Queries used | Cap | Result |
|---|---|---|---|
| 1 GitHub | 8 | 8 | Five useful hits (AG-UI Rust crates, circuit breakers, MCP Apps code references, rmcp roadmap); three empty (A2A Rust SDK, generic harness, agentskills Rust) |
| 2 Docs | 6 docfork attempts all failed with `fetch failed`; 3 Context7 resolves + 3 queries succeeded | 8 | rmcp protocol negotiation, backon `adjust` and jitter, failsafe async usage |
| 3 Registry | 13 `cargo search` calls | 8 | Over cap by five because the calls were batched in one command; results are reported at lowered confidence where they are the only evidence |
| 4 Web | 0 | 8 | Not needed; the assessment's harness survey already covers the landscape |
| Local | registry source reads for rmcp 3.1.2/3.1.4, jsonschema 0.49.4, tiktoken-rs 0.12.0, backon 1.6.0, json-patch 3.0.1; `cargo tree -i` for backon | — | Grounded the pinned-version facts that matter most |

Total wall time for research was under the 20-minute budget. The deep-research server remains unusable (defunct worker), so no pipeline-scored sources were added in this stage.

## The filter applied to every candidate

Three tests, in order. A candidate that fails the first is rejected regardless of quality.

1. Provider neutrality. Does adopting it bind UAR to one transport or vendor? Codex's `previous_response_id` resumption and sticky routing fail this; a backoff crate does not.
2. Existing seam. Does UAR already have the seam the candidate needs, or does the candidate create a second parallel mechanism? The `uar-harness-parity` phase killed the parking-lot `HookBus` for exactly this reason. A second event bus, a second token counter, or a second registry is a reject.
3. Named failure. Does the gap trace to an observed defect in the assessment, a spec requirement, or an operator instruction? The CLAUDE.md evidentiary standard forbids defensive code without one. Two candidates below are deferred on this test alone.
4. Maintenance. Every crate given an adopt verdict must show recent activity, a permissive license, and no archive flag, read from the GitHub API on 2026-09-02, not from recall.

| Crate | Version here / latest | Stars | Last push | Open issues | License | Note |
|---|---|---|---|---|---|---|
| backon | 1.6.0 / 1.6.0 | 1,052 | 2026-06-19 | 25 | Apache-2.0 | already transitive via redis |
| jsonschema | 0.49.4 / 0.52.1 | 811 | 2026-09-01 | 12 | MIT | latest release rust-v0.52.1 on 2026-08-30 |
| wiremock | 0.6.5 / 0.6.5 | 798 | 2025-08-24 | 31 | Apache-2.0 | no GitHub releases; more than a year without a push. Adopt for dev only; if it blocks, `httpmock` is the fallback, unresearched |
| insta | 1.48.0 / 1.48.0 | 2,952 | 2026-08-17 | 73 | Apache-2.0 | release 1.48.0 on 2026-06-11 |
| rmcp | 3.1.2 / 3.2.0 | 3,863 | 2026-09-02 | 55 | not asserted by GitHub; crate is Apache-2.0 on crates.io | kept, not bumped |

The wiremock row is the one maintenance concern; it is tolerable for a dev-dependency and is flagged in the candidate file.

## Gap-by-gap decisions

### G1. Context history integrity

Defects: tool-call pairs severable (`src/uar/context/strategy.rs:119, 135, 298, 331`), system prompt trimmable (`src/uar/runtime/manager.rs:1471` with `:1517`), identical user messages deduplicated (`src/uar/runtime/context/manager.rs:193-198`), two tokenizers on one run (`strategy.rs:100` vs `context/token_service.rs:9`), unbounded tool output (`src/uar/tools/terminal_exec.rs:75-81`, `src/llm/orchestrator.rs:1010-1021`).

Decision: BUILD, porting two algorithms from Codex.

- Pair normalization: port the invariant set from `core/src/context_manager/normalize.rs:21-138` (every call has an output, every output has a call, synthetic aborted outputs inserted in reverse index order). No crate does this; it is about 120 lines and depends only on UAR's message type.
- Output truncation: port `utils/output-truncation/src/lib.rs` (209 lines, depends only on two Codex utility crates, replaceable with `String` helpers). Middle-out truncation with the "Warning: truncated output (original token count: N)" header, applied once at ingest, per Codex `history.rs:246-282`. Apply the same policy to MCP results, native results, and `terminal_exec`.
- Single token service: keep `tiktoken-rs` 0.12.0 (`Cargo.toml:378`); it exposes `get_bpe_from_model` and `o200k_base_singleton` (registry `tiktoken-rs-0.12.0/src/api.rs:270`, `src/singleton.rs:55`), so model-keyed selection with `cl100k_base` as the documented fallback needs no new dependency. Delete the `len/4` estimator. The optional `tokenizers` feature (`Cargo.toml:377`) stays optional for Hugging Face models.
- System-message pinning and the dedup removal are edits to existing code, not new components.

Rejected: `tiktoken-rs` upgrade (0.12.0 is current on crates.io), any second context strategy enum. The two enums (`uar::context::ContextStrategy` and `uar::domain::context::ContextStrategy`) collapse to one; which survives is a spec decision, and the `progressive-summarization` spec's requirements bind the survivor.

Confidence: high. Every input is a local code read.

### G2. Fail-closed tool arguments and effect classification

Defects: malformed JSON executes as `{}` (`orchestrator.rs:982-983`, `:1032-1033`); effects inferred from name prefixes (`orchestrator.rs:138-148`, `manager.rs:366-370`, `:115-134`); `NativeSkill` has no effect or output-limit field (`src/uar/runtime/native_skill.rs:49-61`); MCP merge duplicates tools (`src/mcp/registry.rs:733-734`).

Decision: ADOPT `jsonschema` for validation; BUILD the descriptor and error types.

- `jsonschema` 0.49.4 is already a dependency with `default-features = false` (`Cargo.toml:418`), which removes HTTP and file resolvers and is correct for inline tool schemas. The installed API compiles once with `validator_for` and validates with `validate`, `iter_errors`, `is_valid` (registry `jsonschema-0.49.4/src/lib.rs:1346`, `src/validator.rs:482-500`). Compile one validator per tool descriptor at assembly time, never per call. crates.io has 0.52.1; upgrading is a separate decision with no defect driving it, so keep 0.49.4 and record it in `versions.toml`, which currently omits it.
- Port the two-variant error shape from `tools/src/function_call_error.rs:5-10`: a model-visible `invalid_arguments` result versus a turn-fatal harness error. Parse failure and schema failure both return to the model as a failed tool result with the validator's message, matching Codex `core/src/tools/handlers/mod.rs:83-90` and `core/src/tools/parallel.rs:219-243`. Neither executes.
- Effect classification: a `ToolEffect` field on the descriptor with `ReadOnly`, `ExternalMutation`, `CodeExecution`, `Unknown`; legacy tools default to `Unknown`, which is sequential and governed. MCP tools map `readOnlyHint` to `ReadOnly` the way Codex does (`core/src/tools/handlers/mcp.rs:128-139`), with the MCP 2026-07-28 caveat that annotations are untrusted, so `ReadOnly` from an annotation only unlocks parallelism, never approval bypass.
- Parallelism: Codex's single `RwLock` where read-only tools take the read lock and everything else the write lock (`core/src/tools/parallel.rs:42-62`, `:155-159`) replaces the prefix allowlist and the `.buffered(8)` block at `orchestrator.rs:974-1030`. Deterministic result ordering is preserved by collecting in call order.

Rejected: name-prefix inference in any form after migration; a second tool registry. The existing native-wins collision rule at `orchestrator.rs:505` becomes a descriptor-level rule that also covers MCP-versus-MCP.

Confidence: high on validation and errors; medium on the effect taxonomy until the spec names the approval class per effect.

### G3. Deterministic prompt assembly and provenance

Defects: skill overlay order from `HashMap` iteration (`src/uar/runtime/skills/registry.rs:16`, `:209-215`) defeating Anthropic prefix caching (`src/llm/anthropic_cache.rs:55-73`); no authority marker between operator, retrieved, and skill text; `AgentPrompt.instructions` write-only (`src/uar/domain/artifact.rs:118-122`); no world state.

Decision: BUILD, porting the design (not the code) of Codex `context-fragments`.

- `context-fragments/src/fragment.rs` is 135 lines plus two helpers (346 total) and depends on Codex protocol types, so it is a design reference rather than a vendoring target. The shape to keep: a fragment has a role, a stable content-kind id, start and end markers, a body, and a `requires_separate_message` flag; rendering concatenates without adding separators; a static matcher recognizes the fragment later so compaction and rollback can remove it (`fragment.rs:58-67`).
- Fixed section order, one sort key per section, and a stable content hash per fragment give a diffable `TurnManifest` and make the prefix byte-stable for identical inputs. `indexmap` 2.14 is the natural container for ordered world-state sections (Codex uses it at `core/src/context/world_state/mod.rs:288`); it is not in UAR's direct dependencies, and a `Vec` with explicit sort keys is acceptable if the spec prefers no new crate.
- Authority classes: `System`, `Policy`, `Host`, `Skill`, `Retrieved`, `User`. Retrieved documents and skill bodies carry markers so a later injection screen can treat them as data. This is the prompt-side half of the guardrails gap (tool results unscreened, `src/server.rs:4702`).
- `AgentPrompt.instructions` becomes a rendered fragment or is deleted; the compiler currently hardcodes it empty (`src/uar/compiler/to_artifact.rs:148`).

Rejected: sourcing base instructions from a vendor model catalog (`protocol/src/openai_models.rs:518-535`). UAR's artifact plus the prompt-dialect engine (`src/llm/prompt_dialect.rs`) is the provider-neutral equivalent; its two unwired predicates (`prefers_xml_envelope` `:64`, `markdown_averse` `:71`) become fragment-rendering inputs.

Confidence: high on ordering and hashing; medium on the authority taxonomy.

### G4. Progressive skill runtime: catalog, activation, use

Defects: full overlay for every match with no budget (`manager.rs:1448-1454`); threshold unread (`src/uar/runtime/skills/service.rs:698`); below-threshold "include anyway" (`manager.rs:1383-1394`); `max_active` and `prefer` dead (`artifact.rs:96-105`); no explicit activation; overlay-only skills excluded from outcome telemetry (`manager.rs:1441-1445`); skill MCP servers respawned per run (`manager.rs:1457`).

Decision: BUILD; adopt the agentskills.io conventions UAR already loads from `~/.agents/skills` (decision 2026-08-29) and Codex's budget constants as starting defaults.

- Catalog line per eligible skill under a budget of 2% of the model context window, configurable cap 10,000 tokens, 8,000-character fallback when the window is unknown, 1,024 characters per description, 8,000 bytes per activated body (`ext/skills/src/render.rs:17-22`). Round-robin description truncation before any omission (`render.rs:325-366`, `:408-447`) so a 2,000-skill catalog degrades gracefully; the gotcha of 2026-08-09 measured 2,266 skills machine-wide, so this is an observed scale, not a hypothetical.
- Explicit activation on two paths: `skill_attachments` on the run request, intersected with the effective eligible set; and a model-only `activate_skill(skill_id)` tool that loads the body, validates MCP dependencies, records exact usage, and updates the next step's tool set. Gemini CLI and OpenCode use a model tool; Codex and Claude Code use mentions; UAR gets both because its clients are programmatic.
- Scored `SkillMatchResult` with threshold and margin enforced; remove the "include top matches anyway" branch. The TF-IDF and local-embedding reducers stay in shadow mode with recall telemetry until Recall@10 against explicit activations is measured; Codex ships its selector the same way (`ext/skills/src/shadow_selection_experiment/mod.rs:1`).
- Activated bodies persist as conversation items with a contextual marker so compaction can reclaim them first (Codex `ext/skills/src/lib.rs:50-54`); Claude Code's re-attachment budget (5,000 tokens per skill, 25,000 total) is the alternative the spec should weigh, because prompt-only skills are UAR's common case.
- `max_active` enforced; `preferred_tools` either applied as a per-step tool narrowing or removed.

Rejected: any Rust crate for skills (none found in Tier 1 or Tier 3); the Codex shadow selector as a gate.

Confidence: high.

### G5. Model-path resiliency

Defects: jitter and Retry-After validated but unread (`src/uar/settings/resilience_policy.rs:97-100`, `:39` vs `orchestrator.rs:640-643`); retryability by substring (`orchestrator.rs:150-175`); one-hop failover (`manager.rs:1665-1690`); health monitor off the hot path (`manager.rs:1584-1607`); no stall timeout (`orchestrator.rs:628`); partial turn persisted as complete (`manager.rs:2397-2399`); chat SSE ignores `Last-Event-ID` (`src/server.rs:5019`).

Decision: ADOPT `backon` for retry mechanics; BUILD the error classification and the rest.

- `backon` 1.6.0 is already in the dependency graph through `redis` via `opendal-service-redis` (`cargo tree -i backon`), so promoting it to a direct dependency adds no new crate. Its `ExponentialBuilder` has `with_jitter`, `with_max_delay`, `with_total_delay`, and `with_max_times` (Context7, docs.rs/backon/1.6.0), which map one-to-one onto the existing `ResiliencePolicy` fields, and `Retry::adjust` (registry `backon-1.6.0/src/retry.rs:329`) lets a `Retry-After` value override the computed delay, which is what Codex does at `core/src/responses_retry.rs:105`.
- Retryability must come from a typed provider error, not a string. Whether liter-llm 1.18.2 exposes status and `Retry-After` on its error type is the first open question below; if it does not, the driver layer classifies at the HTTP boundary.
- Stall timeout on an established stream: Codex applies a per-provider idle timeout on SSE and WebSocket reads (`codex-api/src/endpoint/responses.rs:187`). UAR adds one field to `ResiliencePolicy`.
- Partial-turn marker: persist a truncated assistant message with a marker fragment the model can see, per Codex `core/src/context/turn_aborted.rs:10-11`.
- Health gating: route `start_run_with_policy_and_history` through the existing `ModelRouter` so a provider in cooldown is skipped; the router already exists (`src/llm/router.rs:54, 119`) and is only used by the `/route` endpoint.
- Chat SSE resume: reuse the `Last-Event-ID` path that `src/uar/api/routes.rs:94-99` already implements.

Deferred on the named-failure test: a circuit breaker. `failsafe` 1.3.0 (207 stars, updated 2026-08-19, `futures-support` feature, `consecutive_failures` policy) and `recloser` (130 stars) are both viable, but `ProviderHealthMonitor` (`src/llm/health.rs:69-94`) already implements consecutive-failure cooldown. The observed defect is that it is not consulted, not that it is missing. Revisit only if a live incident shows cooldown is insufficient.

Confidence: high on backon and the wiring; low on liter-llm error typing until verified.

### G6. Typed turn and step assembly with contributors

Defects: one 1,510-line function (`manager.rs:1094-2604`); tool list frozen before the loop (`orchestrator.rs:498-512`, `:601`); skills matched once against the first input (`manager.rs:1334`, `:1360`); no extension can touch prompt, context, or sequencing (`src/uar/runtime/wasm/plugin_loader.rs:108-109`).

Decision: BUILD. This is architecture, and no crate provides it.

- Two frozen snapshots, per Codex `TurnContext` (`core/src/session/turn_context.rs:194-246`) and `StepContext` (`core/src/session/step_context.rs:15-34`): the turn freezes policy, artifact, environment, and credentials; each model call freezes settings, the projected tool set, the token budget, and the MCP catalog. `arc-swap` 1.6 is already pinned (`Cargo.toml:411`) for the "next step settings" swap Codex does with `ArcSwap` (`turn_context.rs:211`).
- Contributor traits are internal Rust interfaces with fixed stages: artifact instructions, effective policy, memory and RAG, skills, MCP and tools, context, lifecycle observation. The Codex list of twelve (`ext/extension-api/src/contributors.rs:77-380`) is the reference; UAR needs the seven above now. Contributors return owned data and cannot broaden `EffectiveRunPolicy` or bypass Cedar.
- Shadow mode is mandatory for migration and is the expensive part: the monolith has no seams, so the first change must cut seams (extract the prompt build, the skill match, the MCP merge, and the context pass into functions with typed inputs) before a typed assembler can run beside them. That is why G1 through G5 are sequenced before G6: each one extracts a seam as a side effect of fixing a defect.

Rejected: the WIT plugin ABI as the contributor mechanism in this phase (`plugin-loader-wit-contract` and `wasm-component-skill-runtime` remain separate changes); resurrecting a hook bus.

Confidence: medium. The design is clear; the migration cost is not measurable until the seams exist.

### G7. Projected MCP runtime

Defects: per-run respawn of skill servers (`manager.rs:1457`), inert `sandboxed` flag (`src/mcp/config.rs:21`; `src/mcp/registry.rs:423-428`), no deferred exposure, `merge` duplication (`registry.rs:733-734`).

Decision: BUILD on the existing registry; keep `rmcp` 3.1.x; plan the 2026-07-28 move separately.

- The installed `rmcp` 3.1.2 already defines `ProtocolVersion::V_2026_07_28` but negotiates `LATEST = V_2025_11_25` (registry `rmcp-3.1.2/src/model.rs:170-175`); 3.1.4 is in the registry with the same constants, Codex pins `=3.1.3` (`codex-rs/Cargo.toml:416`), and 3.2.0 is on crates.io. rmcp's docs state that versions from 2026-07-28 are served statelessly by default and that `legacy_session_mode` exists for older clients (Context7, `StreamableHttpServerConfig`). So the crate path to the new spec exists without a major bump, but every one of UAR's MCP servers and every surveyed client still speaks 2025-11-25, and Dynamic Client Registration is deprecated in the new spec. Verdict: no bump in this phase; a dedicated `mcp-2026-07-28-alignment` change later, gated on at least one real server UAR needs speaking the new version.
- Lazy startup, connection reuse keyed on config hash plus auth identity, and single-flight refresh extend the generation-guarded reconnect slot that already exists (`registry.rs:64-161`, decision 2026-08-21). Deferred exposure and a `search_tools` tool follow the G2 descriptor's `exposure` field; Codex registers the search tool only when a deferred tool exists (`core/src/tools/spec_plan.rs:371-406`).
- Stdio sandbox: microsandbox was removed for a vulnerable transitive dependency (`src/sandbox/mod.rs:1-6`). Codex's `sandboxing` crate uses OS-native Seatbelt, Landlock, and bwrap with SBPL policy files (`sandboxing/src/manager.rs:36-75`) and is Apache-2.0. Porting it is the credible path to making `sandboxed: true` real, and Claude Code and Cursor ship the same OS-native approach per the survey. This is a candidate to evaluate in spec, not a decision here: it adds platform-specific code and its own test matrix.

Rejected: an eager prewarm of every skill-declared server; any weakening of the reconnect generation guard.

Confidence: high on lifecycle; medium on the sandbox port.

### G8. Thread-native subagents

Defects: three divergent paths (`src/uar/runtime/actor/agent_actor.rs:115`, `src/uar/runtime/graph/nodes/agent_node.rs:119`, `:213`), no policy, budget, persistence, limits, or cancellation for children, text-prepended identity (`agent_actor.rs:201-202`), inbound A2A bound to the compiler (`src/uar/api/a2a/handler.rs:41-47`).

Decision: BUILD one kernel with adapters; adopt Codex's governance rules as requirements.

- Durable `AgentThread` and `AgentEdge` records on the existing `PersistenceLayer` trait for all three providers; Codex's `agent-graph-store` (`agent-graph-store/src/store.rs:17-59`) is the shape, including the stable-ordering requirement and status filtering on every traversed edge.
- Requirements lifted directly from Codex, because they are provider-neutral safety properties: child runtime policy is forced from the live turn and re-applied after any role or artifact overlay (`core/src/tools/handlers/multi_agents_common.rs:235-262`; `multi_agents_v2/spawn.rs:145`); roles and child artifacts can only narrow (`core/src/agent/role.rs:1-4`); permission intersection fails closed (`protocol/src/permission_profile_intersection.rs:31-68`); user authorization is read from the root, never from a child's text (`core/src/agent/control/user_authorization.rs:22-94`); inter-agent identity is metadata (`protocol/src/protocol.rs:803-819`).
- Limits: four concurrent children, depth three, sixteen total per root run, as the supplied analysis proposed; the surveyed range supports it (Codex three children default, Claude Code twenty concurrent and depth three, Cursor depth two).
- Cancellation: child tokens from the run's token (`tokio-util` is already a dependency); A2A children get a `tasks/cancel` on parent cancel, using the client method that exists and is never called (`src/uar/api/a2a/client.rs:121`).
- AG-UI `SubagentStarted/Finished/Error` events become emittable once a lifecycle exists. The Rust AG-UI crates found (`ag-ui-rs` 0.3.1, `ag-ui` 0.1.0, `agentsyaml/agui-rs` at 5 stars) are immature and UAR already self-hosts its profile; not adopted. A2A crates (`a2a-core` 0.0.0, `a2a-agents` 0.7.0) likewise; UAR's own JSON-RPC and gRPC implementation stays, and A2A v1.0.1 alignment (`tasks/list`, "canceled") is a later change.

Rejected: Codex agent identity against a vendor backend; the guardian model reviewer as a requirement (see open questions).

Confidence: high on the requirements; medium on the adapter migration for actors and the graph.

### G9. Project instructions and world state

Gap: zero hits for AGENTS.md, CLAUDE.md, cwd, time, or environment injection in `src/`. The operator's code-generation goal needs this, and every surveyed harness has it.

Decision: BUILD; `walkdir` 2.5 is already pinned (`Cargo.toml:397`).

- Discovery per Codex `core/src/agents_md.rs:1-64`: walk up to a project-root marker, collect root-to-cwd, prefer an override file, skip untrusted projects. Add `CLAUDE.md` and `GEMINI.md` as configurable file names the way Gemini CLI's `context.fileName` list does; UAR's users run Claude Code, Codex, and others side by side.
- World-state sections with stable ids and merge-patch diffing so only changes are re-sent (`core/src/context/world_state/mod.rs:228-348`). `json-patch` 3.0.1 in the registry has `merge` (apply, `json-patch-3.0.1/src/lib.rs:661`) but no merge-patch generator; Codex wrote its own `merge_patch_from`. A generator is under fifty lines; build it rather than add a crate for half the job.

Confidence: high.

### G10. MCP Apps and other protocol alignment

Gap: MCP-UI / MCP Apps absent; A2A at RC v1.0 versus upstream v1.0.1; AG-UI dual vocabulary; A2UI at v0.9.1 with v1.0 candidate.

Decision: DEFER all four to their own changes with explicit triggers; no candidate crates.

- MCP Apps has no Rust SDK (Tier 1 found only Codex and Goose protocol type references and a preview server crate); the reference SDK is TypeScript `@modelcontextprotocol/ext-apps` 1.7.5. UAR's frontend would adopt it; the Rust side only needs `ui://` resource passthrough and the `io.modelcontextprotocol/ui` extension capability. Trigger: an operator need for an MCP server that ships an app.
- A2UI stays at v0.9.1 (decision 2026-08-28) until v1.0 leaves candidate status.
- AG-UI: retire the legacy dotted vocabulary (`src/uar/api/sse.rs:295`, `:345-444`) on a deprecation schedule once the G8 subagent lifecycle exists, because the upstream `SubagentStarted/Finished/Error` events are the first additions UAR cannot express in the old vocabulary. Trigger: G8 merged. Evidence needed: one release of dual emission with the `agui_spec` flag defaulting to the official vocabulary and no client regression in `frontend/src/platform/agui/`.
- A2A: UAR implements RC v1.0 JSON-RPC and gRPC v0.3 (`src/uar/api/a2a/mod.rs:1`; `openspec/specs/a2a-grpc/spec.md`); upstream is v1.0.1 (2026-05-28) with `tasks/list`, modernized OAuth, removed deprecated fields, and a standardized `canceled` state. Alignment is deferred because both UAR transports currently front the compiler, not the agent runtime (`src/uar/api/a2a/handler.rs:41-47`), so aligning the wire contract before G8 would certify an endpoint that cannot run an agent. Trigger: G8 maps A2A task, status, cancel, and result onto the thread service. Evidence needed: the upstream v1.0.1 specification diff against `openspec/specs/a2a-grpc/spec.md`, and at least one external A2A client (Gemini CLI remote subagents are the surveyed example) exercising `message/send` and `tasks/cancel` against UAR.

### G11. Runtime observability

Gap from the assessment: graph runs fabricate `RuntimeStep` pairs after completion (`src/uar/runtime/manager.rs:1993-2009`); nothing attributes tokens, cost, or tool calls to a skill (`manager.rs:136-151`, `:1441-1445`); prompt bodies and retrieved content have no redaction boundary in telemetry; there is no per-turn manifest of what was assembled. This is the parent phase's own subject ("runtime observability"), so it cannot be left implicit.

Decision: BUILD on the existing OTLP, metrics, and normalized-event bus. No new crate; the seams exist.

- Redacted `TurnManifest`: identifiers, fragment hashes, counts, budgets, provenance, selected skills and tools, warnings; never prompt bodies, credentials, hidden reasoning, or raw retrieved content. Stored in `Run.context` and emitted as an additive artifact, as the supplied analysis proposed. The G3 fragment hashes are what make this cheap.
- Log versus trace boundary, ported as a rule from Codex `otel/src/events/session_telemetry.rs:585-596` and `:1053-1072`: high-cardinality or sensitive values (prompt text, MCP server names) only in logs behind an explicit `log_user_prompts`-style switch; traces carry lengths and counts. Inter-agent message bodies are never logged (`core/src/agent_communication.rs:59-62`).
- Skill attribution: a `codex.skill.injected`-style counter tagged with skill, source, model, and invoke type (`core/src/skills.rs:64-75`), extended to overlay-only skills, which Codex also cannot attribute. Claude Code tags every token and cost metric with `skill.name` (survey); UAR should tag `uar_llm_cost_usd` and token metrics the same way once G4 records exact activation.
- Live `RuntimeStep` emission for graph runs replaces the post-hoc replay; this is the `emit-runtime-step-events` carry-over from `uar-harness-parity` finally closed for the graph path. Subagent lifecycle events (G8) reuse the same emitter and feed AG-UI `SubagentStarted/Finished/Error`.
- Existing metrics recorders that are still dead (sandbox and MCP-status, per `uar-harness-parity/reflection.md:71`) get real sources from G7's server state machine.

Rejected: a second telemetry pipeline or analytics client; Codex's separate `analytics` crate is a product-analytics channel, not a runtime need.

Confidence: high. Every item is an extension of a recorder that already exists.

## Testing practices to adopt

- `wiremock` 0.6.5 (Codex pins 0.6) and `insta` 1.48.0 as dev-dependencies. UAR has `mock_driver.rs` but no HTTP-level provider mock and no snapshot tests. The single most valuable Codex practice is the orphaned-call validator inside the mock (`core/tests/common/responses.rs:711`, `:1675-1740`): every model-backed test becomes a wire-format conformance test. Open question: whether liter-llm's client honors a base-URL override so wiremock can sit in front of it.
- Request-diff snapshots (`core/tests/suite/snapshots/*_request_diff.snap`) are the cheapest regression guard for the G3 prefix-stability guarantee.
- Content-matched mock responses (`responses.rs:1103`) are what make deterministic multi-agent tests possible for G8.

## Ranking for spec and plan

Immediate, each traceable to an observed defect and each extracting a seam the later items need:

1. G1 context history integrity
2. G2 fail-closed tool arguments and effect classification
3. G3 deterministic prompt assembly and provenance
4. G4 progressive skill runtime
5. G5 model-path resiliency

Structural, after the seams exist:

6. G6 typed turn and step assembly with contributors
7. G7 projected MCP runtime
8. G8 thread-native subagents
9. G9 project instructions and world state

Cross-cutting, delivered incrementally inside items 3, 4, 7, and 8 rather than as one change:

- G11 runtime observability: the turn manifest lands with G3, skill attribution with G4, MCP state events with G7, live step and subagent events with G8.

Later, trigger-gated:

10. G10 MCP 2026-07-28 alignment, MCP Apps, A2A v1.0.1, AG-UI vocabulary retirement

This differs from the supplied analysis in two ways. It puts three small correctness changes before `typed-turn-assembly` instead of after it, because the monolith has no seams to shadow behind and those changes create them. And it splits resiliency out as its own immediate item, because two of its fixes are one-line reads of config that already exists.

## Open questions for spec

1. Does liter-llm 1.18.2 expose a typed error with HTTP status and `Retry-After`? If not, retry classification lives in `liter_driver.rs` at the HTTP boundary. Verify against the vendored source before writing the G5 spec.
2. Does liter-llm's client accept a base-URL override per request so `wiremock` can front it in tests? If not, the conformance validator moves to the driver trait boundary.
3. Which `ContextStrategy` enum survives, and does `progressive-summarization`'s spec become the only compaction requirement set?
4. Authority taxonomy and approval class per tool effect: `Unknown` is sequential and governed, but is `ExternalMutation` always `Ask` under `ToolApprovalPolicy::Auto`?
5. Should a guardian-style model reviewer be specified as an optional tier behind Cedar? Codex fails closed with a circuit breaker (`core/src/guardian/mod.rs:12`, `:196-246`); every surveyed classifier tier is documented as best-effort. Recommendation: not in this phase; record as a future capability.
6. Port Codex `sandboxing` for stdio MCP children, or leave `sandboxed` inert with a documented warning? The port is the only credible path to the flag meaning something.
7. `versions.toml` needs entries for `jsonschema`, `rmcp`, `tiktoken-rs`, `wasmtime`, `tonic`, and the A2A and AG-UI protocol versions before any of these changes lands; the file is operator-edited only.

## Appendix: verified Codex excerpts

The Codex workspace is outside this repository, so the review packet cannot resolve its paths. These excerpts were read directly from `/Users/gqadonis/Projects/references/codex/codex-rs` at `986ff1cc` on 2026-09-02 and are the lines each decision above depends on. Apache-2.0.

`tools/src/function_call_error.rs:5-10` (G2 error shape):
```rust
pub enum FunctionCallError {
    #[error("{0}")]
    RespondToModel(String),
    #[error("Fatal error: {0}")]
    Fatal(String),
}
```

`core/src/tools/parallel.rs:155-159` and `tools/src/tool_executor.rs:122-124` (G2 parallelism, safe default):
```rust
let _guard = if supports_parallel {
    Either::Left(lock.read().await)
} else {
    Either::Right(lock.write().await)
};
// ...
fn supports_parallel_tool_calls(&self) -> bool {
    false
}
```

`core/src/tools/handlers/mcp.rs:128-139` (G2 readOnlyHint mapping):
```rust
fn supports_parallel_tool_calls(&self) -> bool {
    // Correctly implemented MCP servers should tolerate parallel calls to
    // tools that advertise themselves as read-only.
    self.tool_info.supports_parallel_tool_calls
        || self.tool_info.tool.annotations.as_ref()
            .and_then(|annotations| annotations.read_only_hint)
            .unwrap_or(false)
}
```

`core/src/context_manager/history.rs:563-566` and `normalize.rs:21-24` (G1 pairing invariants):
```rust
/// This function enforces a couple of invariants on the in-memory history:
/// 1. every call (function/custom) has a corresponding output entry
/// 2. every output has a corresponding call entry or names an external tool event
/// 3. unsupported image and audio content is stripped from messages and tool outputs
// ...
pub(crate) fn ensure_call_outputs_present(items: &mut Vec<ResponseItemEnvelope>) {
    let mut function_output_ids = HashSet::new();
```

`utils/output-truncation/src/lib.rs:14-24` (G1 truncation header):
```rust
pub fn formatted_truncate_text(content: &str, policy: TruncationPolicy) -> String {
    if content.len() <= policy.byte_budget() {
        return content.to_string();
    }
    let original_token_count = approx_token_count(content);
    let total_lines = content.lines().count();
    let result = truncate_text(content, policy);
    format!(
        "Warning: truncated output (original token count: {original_token_count})\nTotal output lines: {total_lines}\n\n{result}"
    )
}
```

`context-fragments/src/fragment.rs:56-69` (G3 fragment contract):
```rust
/// Implementations own the response role and provide the exact fragment body.
/// Marked fragments also provide start/end markers used to recognize injected
/// context later. `render()` concatenates markers and body without adding
/// separators, ...
pub trait ContextualUserFragment {
    fn role(&self) -> &'static str;
    /// Returns a stable `<feature>.<name>` classification, using `generic` for shared fragments.
    fn content_kind(&self) -> ContentItemKind;
```

`ext/skills/src/render.rs:17-22` (G4 budget constants):
```rust
const DEFAULT_SKILL_METADATA_CHAR_BUDGET: usize = 8_000;
const MAX_CONFIGURED_SKILL_METADATA_TOKEN_BUDGET: usize = 10_000;
const MAX_SKILL_PROMPT_BYTES: usize = 8_000;
const SKILL_METADATA_CONTEXT_WINDOW_PERCENT: usize = 2;
const MAX_CATALOG_SKILL_DESCRIPTION_CHARS: usize = 1_024;
const TRUNCATED_SKILL_DESCRIPTION_SUFFIX: &str = "...";
```

`core/src/responses_retry.rs:105` (G5 Retry-After precedence):
```rust
let delay = err.retry_delay().unwrap_or_else(|| backoff(retry_count));
```

`core/src/session/step_context.rs:15-34`, abridged (G6 step snapshot):
```rust
pub(crate) struct StepContext {
    pub(crate) turn: Arc<TurnContext>,
    /// One immutable settings version captured before request preparation.
    pub(crate) settings: Arc<ResolvedStepSettings>,
    pub(crate) token_budget: Option<TokenBudgetConfig>,
    pub(crate) environments: TurnEnvironmentSnapshot,
    /// The exact MCP connections, configuration, and catalog captured for this step.
    pub(crate) mcp: Arc<McpBinding>,
    /// The finalized tool plan advertised and executed for this exact sampling request.
    pub(crate) tool_router: Arc<ToolRouter>,
    pub(crate) loaded_agents_md: Option<Arc<LoadedAgentsMd>>,
}
```

`codex-mcp/src/connection_manager.rs:258-259` (G7 lazy startup gate):
```rust
let allow_deferred_startup =
    startup_policy == McpStartupPolicy::LazyWhenCached && previous.is_some();
```

`core/src/agent/role.rs:1-4` (G8 roles narrow only):
```rust
//! Applies bounded agent-role overrides to an existing session config.
//!
//! Roles may customize the child or reduce its capabilities, but never replace the parent
//! session's authority. A projected layer keeps existing layer-based consumers in sync.
```

`protocol/src/permission_profile_intersection.rs:31-46`, abridged (G8 fail-closed intersection):
```rust
/// Concrete grant paths are canonicalized before comparison and in the
/// result, so symlinks cannot acquire authority beyond either input.
/// Unsupported policy shapes fail closed.
pub fn intersect_effective_permission_profiles(authority: &PermissionProfile, requested: &PermissionProfile, cwd: &Path)
    -> Result<PermissionProfile, PermissionIntersectionError> {
    if matches!(authority, PermissionProfile::External { .. }) || matches!(requested, PermissionProfile::External { .. }) {
        return Err(PermissionIntersectionError::ExternalSandbox);
    }
```

`core/src/agent/control/user_authorization.rs:22-29` (G8 root-only authorization):
```rust
pub(crate) async fn root_user_authorization(&self, thread_id: ThreadId) -> Option<GuardianRootSnapshot> {
    let root_thread_id = self.state.agent_id_for_path(&AgentPath::root())?;
    if root_thread_id == thread_id {
        return None;
    }
```

`core/src/agents_md.rs:1-17`, abridged (G9 discovery algorithm):
```
//! 1.  Determine the project root by walking upwards from the current working
//!     directory until a configured `project_root_markers` entry is found. ... (`.git`).
//! 2.  Collect every `AGENTS.md` found from the project root down to the
//!     current working directory (inclusive) and concatenate their contents in that order.
//! 3.  We do **not** walk past the project root.
```

## Unresolved review findings

Two rounds of isolated cross-model adversarial review ran (`review/analyze/findings.round1.json`, `review/analyze/findings.json`; judge `kbd-judge`, producer `claude-fable-5-1`, cross-model check verified-distinct).

- Round 1 CRITICAL, Codex paths unverifiable by the packet: addressed by the verbatim-excerpt appendix above, which puts the decisive lines inside the artifact. Round 1 WARNINGs on missing A2A and AG-UI entries: addressed in G10 and `library-candidates.json`.
- Round 2 CRITICAL, no observability decision: addressed by G11 and the cross-cutting ranking entry. Round 2 WARNING, no maintenance evidence for adopted crates: addressed by the maintenance criterion and table.
- Both round-2 fixes were applied after the two-round cap and were not re-vetted by the judge. The spec stage's adversarial review will see them.

## Sycophancy self-check

Two candidates were deferred against the supplied analysis and the assessment's own list (circuit breaker, MCP bump) because no observed failure requires them. The ranking contradicts the supplied analysis's ordering and says why. Three of the ten gaps carry medium or low confidence and name the fact that would raise it.
