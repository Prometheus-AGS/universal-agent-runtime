PLAN: codex-harness-comparative-analysis
Project: Universal Agent Runtime (UAR)
Date: 2026-09-02
OpenSpec available: YES (openspec 1.10.0, schema spec-driven; all ten changes already created and strict-valid in the spec stage)
Changes to implement: 10

Inputs: `assessment.md`, `analysis.md`, `library-candidates.json`, `spec-review-notes.md`, and the ten `openspec/changes/<id>/` directories. Ordering follows the dependency table in `spec-review-notes.md`, then customer value. Library annotations follow `library-candidates.json` (adopt verdicts become reuse tasks, never rebuilds). No evolver bridge exists for this phase.

Complexity score and model class follow `references/model-routing.md`: Low is mechanical plumbing with no new abstraction, Medium crosses one module boundary with bounded design decisions, High introduces a new abstraction or crosses a domain, app, or infrastructure boundary.

CHANGE LIST (ordered)

1. context-history-integrity: one history normalizer, pinned system message, one token service, output truncation at ingest, real checkpoint resume
   - Scope: runtime context and history (`src/uar/context`, `src/uar/runtime/context`, result ingest in `src/llm/orchestrator.rs`, checkpoint route)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High (17 tasks, crosses context, orchestrator, and API layers; the routing rule's task-count threshold governs even though no new public abstraction is introduced)
   - Model class: frontier
   - Customer value: HIGH (provider 400s from severed tool pairs and runs that lose their system prompt are user-visible failures)
   - Library: keep `tiktoken-rs` 0.12.0 (cand G1); port Codex `normalize_history` and `output-truncation` (cand G1 ports, Apache-2.0, attributed)
   - Details: Port the two Codex algorithms as pure functions, collapse the two `ContextStrategy` enums to `uar::domain::context`, delete the `len/4` estimator, pin index 0, remove the repeated-message dedup, and wire `restore_state` into the resume endpoint. Creates the first seam typed-turn-assembly composes.

2. fail-closed-tool-arguments: tool descriptor with declared effect, schema-validated arguments, effect-gated parallelism, collision detection
   - Scope: tool protocol (`src/llm/orchestrator.rs` tool loop, `src/uar/runtime/native_skill.rs`, `src/mcp/registry.rs`, approval gate in `src/uar/runtime/manager.rs`)
   - Depends on: NONE; blocked until the operator records `jsonschema = "0.49.4"` in `versions.toml` (task 0.1 stops otherwise)
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High (new `ToolDescriptor` abstraction crossing native, MCP, and built-in tool sources)
   - Model class: frontier
   - Customer value: HIGH (a truncated argument string today executes a tool with empty arguments)
   - Library: adopt `jsonschema` 0.49.4 (cand G2): it is already a Cargo dependency with a caret range and `default-features = false`, is unused on the tool path, and is absent from `versions.toml`; the central pin is the operator gate above, and the change reuses the existing Cargo dependency without adding one. Port Codex `FunctionCallError` shape and `RwLock` parallelism (cand G2 ports)
   - Details: Compile one validator per descriptor at assembly, return `invalid_arguments` to the model on parse or schema failure, replace all three name-prefix heuristics with descriptor fields, implement the read/write lock with the concurrency-key rule from the spec, and dedupe MCP merges. The throughput drop for unannotated tools is intended and documented.

3. deterministic-prompt-assembly: typed prompt fragments with authority and hashes in a fixed order, redacted turn manifest artifact
   - Scope: prompt assembly (`src/uar/runtime/manager.rs` prompt block, `src/uar/domain/artifact.rs`, `src/uar/runtime/skills/registry.rs` ordering, `src/llm/prompt_dialect.rs`, new `src/uar/runtime/prompt/`)
   - Depends on: NONE
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High (new fragment abstraction that every later change fills)
   - Model class: frontier
   - Customer value: HIGH (restores the prompt caching the operator turned on; the manifest is the first per-turn observability artifact)
   - Library: build from Codex `context-fragments` design (cand G3); `arc-swap` kept; `indexmap` optional per spec; adopt `insta` 1.48.0 as a dev-dependency (testing candidate) for request-diff snapshots of the rendered prompt, the cheapest regression guard for prefix stability
   - Details: Replace the `push_str` sequence with fragment construction, sort within sections by id, hash every fragment, emit `turn_manifest` beside the existing effective-policy artifact, render `AgentPrompt.instructions`, and feed the two unwired dialect predicates into rendering. A byte-equality test on identical inputs plus an `insta` snapshot of the diff between two successive rendered prompts is the acceptance gate.

4. model-path-resiliency: policy-driven retry with jitter and Retry-After, typed provider errors, health-gated selection and multi-hop failover, stream idle timeout, interrupted-turn marker, chat SSE resume
   - Scope: model path (`src/llm/orchestrator.rs` retry and stream loop, three drivers, `src/uar/settings/resilience_policy.rs`, `src/uar/runtime/manager.rs` selection and persistence, `src/server.rs` chat SSE)
   - Depends on: deterministic-prompt-assembly (the `TurnInterrupted` marker is a fragment); task 0.1 must settle liter-llm error typing first
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High (16 tasks across three drivers, settings, orchestrator, manager, and server; task 0.1 is an unresolved decision until the liter-llm error type is read)
   - Model class: frontier
   - Customer value: HIGH (lockstep retries and ignored `Retry-After` hit every user of a rate-limited provider; a stalled stream hangs a run)
   - Library: adopt `backon` 1.6.0, already transitive via redis (cand G5); adopt `wiremock` 0.6.5 as a dev-dependency (testing candidate, maintenance concern noted: last push 2025-08-24) if the Round 2 gate confirms liter-llm honors a base-URL override, so `Retry-After` and status classification are tested at the HTTP boundary; otherwise the tests use the existing driver mock and the orphan-call validator moves to the driver trait, and `wiremock` is deferred with that reason; circuit breaker deferred (cand G5 `failsafe`, no named failure)
   - Details: Build `ExponentialBuilder` from `ResiliencePolicy`, use `Retry::adjust` for `Retry-After`, classify errors into `ProviderError` at the driver boundary and delete the substring matcher, route selection through `ModelRouter`, add `stream_idle_timeout_ms`, persist partial turns with the marker, and pass `Last-Event-ID` on the chat path.

5. progressive-skill-runtime: budgeted catalog, client attachments and model-only `activate_skill`, scored matching with `max_active`, shadow reducers, reclaimable bodies, per-skill attribution
   - Scope: skills runtime (`src/uar/runtime/skills/`, skill block in `src/uar/runtime/manager.rs`, run request decoding, a new native tool, metrics)
   - Depends on: deterministic-prompt-assembly (catalog and bodies are fragments), fail-closed-tool-arguments (`activate_skill` is a `ModelOnly` descriptor)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High (new activation path and catalog abstraction crossing runtime, API, and telemetry)
   - Model class: frontier
   - Customer value: HIGH (the 2,266-skill machine in the 2026-08-09 gotcha cannot be served by full-body injection; explicit activation makes skill use provable)
   - Library: adopt agentskills.io conventions already loaded (cand G4); Codex budget constants as defaults (cand G4)
   - Details: Render the catalog under the budget with round-robin truncation, add `skill_attachments` and `activate_skill`, replace `Vec<Skill>` with scored results and delete the "include anyway" branch, gate implicit activation on `skill_activation_mode` (`legacy_overlay` default now, `catalog` later on recall evidence), make bodies reclaimable with re-attachment after compaction, and add per-skill attribution counters without touching the unlabeled totals.

6. typed-turn-assembly: `RunExecutionRequest`, seven-stage contributor registry, immutable `ResolvedTurn` and per-call `ResolvedStep`, shadow mode with a parity report
   - Scope: run kernel (`src/uar/runtime/manager.rs` entry function, `src/llm/orchestrator.rs` per-step inputs, memory prepend in `src/server.rs`, `src/config.rs`, new `src/uar/runtime/turn/`)
   - Depends on: context-history-integrity, fail-closed-tool-arguments, deterministic-prompt-assembly, model-path-resiliency, progressive-skill-runtime
   - Recommended agent: Claude Code (Plan mode first; this is the architecture change)
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: MEDIUM (no visible behavior change by design; the default stays `legacy`; the value is that changes 7 to 9 become possible and testable)
   - Library: build from Codex `TurnContext`, `StepContext`, and contributor-trait design (cand G6); `arc-swap` kept; WIT ABI out of scope
   - Details: Compose the pure functions extracted by changes 1 to 5 behind contributors, re-project tools and active skills per step, move memory inside the assembler, and produce `parity-report.json` with the intentional-delta allowlist. The default does not flip here.

7. projected-mcp-runtime: catalog separate from connections, binding cache with single-flight refresh, lazy start, required-or-optional failure, deferred exposure with `search_tools`, state events, a sandbox flag that is never inert
   - Scope: MCP runtime (`src/mcp/`, MCP block in `src/uar/runtime/manager.rs`, admin API, metrics, new `search_tools` tool, optional `src/sandbox/os_native/`)
   - Depends on: fail-closed-tool-arguments (descriptor `exposure`), typed-turn-assembly (per-step projection)
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High (lifecycle state machine plus a possible OS-native sandbox port)
   - Model class: frontier
   - Customer value: HIGH (skill-declared servers currently pay up to 30 seconds of respawn per run; every surveyed harness has deferred tool loading)
   - Library: keep `rmcp` =3.1.2 (cand G7, no bump); build lifecycle from Codex design (cand G7); Codex `sandboxing` port is task 0.1's decision (cand G7)
   - Details: Add `McpCatalog`, `projection`, and `binding_cache`, extend the generation-guarded reconnect slot rather than replacing it, gate lazy start on cached catalog completeness, emit the six states, register `search_tools` only when deferred tools exist, and make `sandboxed: true` either real or a load-time error.

8. thread-native-subagents: persisted agent threads and edges, narrow-only fail-closed policy intersection, typed inter-agent messages, tree-wide limits, budget, and cancellation, agent tools, actor, graph, and A2A adapters, lifecycle events
   - Scope: multi-agent (`src/uar/runtime/actor/`, `src/uar/runtime/graph/`, `src/uar/api/a2a/`, `src/uar/api/actors.rs`, persistence trait and three providers, events, AG-UI adapters, cost budget, new `src/uar/runtime/thread/`)
   - Depends on: typed-turn-assembly, fail-closed-tool-arguments, model-path-resiliency
   - Recommended agent: Claude Code (Plan mode first)
   - Est. complexity: L (largest change in the set; the spec is deliberately one change so the three adapters cannot drift)
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH (the `multi-agent-orchestration` spec is met today only for one hardwired agent id; the inbound A2A endpoint cannot run an agent)
   - Library: build from Codex `agent-graph-store` design and governance rules as requirements (cand G8); `tokio-util` kept; AG-UI and A2A crates rejected
   - Details: Add the thread kernel with persistence, intersection, messages, and limits; register the five agent tools; convert actors, graph nodes, and the A2A handler into adapters; record child usage on the root budget; emit lifecycle and AG-UI subagent events; require user context on the actor API. Removes the `[route]` text prefix and unauthenticated actor endpoints, both stated in the proposal.

9. project-instructions-world-state: AGENTS.md discovery root-to-cwd with a trust flag, host-authority fragments, world-state sections with merge-patch diffing and a granular clock
   - Scope: runtime context (new `src/uar/runtime/project_instructions.rs` and `src/uar/runtime/world_state/`, a contributor, config)
   - Depends on: deterministic-prompt-assembly, typed-turn-assembly
   - Recommended agent: Codex (via git worktree; isolated, well-specified, no shared state with 7 or 8)
   - Est. complexity: M
   - Complexity score: High (11 tasks and two new modules; the trust boundary is a design decision with no prior art in `openspec/specs/`)
   - Model class: frontier
   - Customer value: HIGH for the code-generation goal (an agent in a repository today knows neither the repository's rules nor its own working directory)
   - Library: build from Codex `agents_md` and world-state design (cand G9); `walkdir` kept; `json-patch` rejected, generator in-house
   - Details: Implement discovery with override and untrusted-workspace skipping, render as `Host` fragments, add sections with stable ids and a substitutable clock at one-minute granularity, and diff with an in-house RFC 7386 generator so only changed sections re-enter history.

10. typed-turn-default-flip: change the default harness mode to `typed` on recorded parity and live-smoke evidence; keep `legacy` for one minor release
   - Scope: config default, settings schema, release notes, decision log
   - Depends on: typed-turn-assembly plus its parity report and a live smoke run in shadow mode with zero unexpected differences; practically also 7 to 9 so the parity corpus covers them
   - Recommended agent: OpenCode (quick targeted edit) with the evidence attached by the operator
   - Est. complexity: S
   - Complexity score: Medium (8 tasks by count, but a single layer with no new type; the evidence gate, not the code, is the work)
   - Model class: medium
   - Customer value: MEDIUM (unlocks per-step re-projection for every client; risk is the reason it is gated)
   - Library: none
   - Details: Flip the default, add the deprecation note, and record corpus size and smoke set contents in the decision log. Not merged if either evidence record is missing or shows an unexpected difference.

EXECUTION ROUND ORDER
Round 1 (parallel, three isolated worktrees): context-history-integrity, fail-closed-tool-arguments (only once `versions.toml` has the jsonschema entry), deterministic-prompt-assembly
Round 2 (parallel): model-path-resiliency, progressive-skill-runtime
Round 3 (single): typed-turn-assembly
Round 4 (parallel): projected-mcp-runtime, thread-native-subagents, project-instructions-world-state
Round 5 (single, evidence-gated): typed-turn-default-flip

Round 1 changes all touch `src/uar/runtime/manager.rs`, and two of their blocks are adjacent at the system-message push (`:1471-1477`, where the system message is pushed and history appended). That boundary is assigned once: deterministic-prompt-assembly owns `:1229-1477` including the push, because the push is where its rendered fragments enter the message vector; context-history-integrity owns `:1478-1538` (the two reducer calls) and receives the system message as a pinned input rather than editing the push. fail-closed-tool-arguments owns `:366-370` and `:1712-1826`. Development is parallel in three worktrees; merge order is 3 (prompt), then 1 (context), then 2 (tools), each later branch rebasing on the previous merge before its Tier 2 run. The gotcha of 2026-08-25 (assessment line numbers go stale) applies: each change re-reads its block before editing and records the actual lines in its tasks notes.

GATES BETWEEN ROUNDS (blocking)
- Before Round 1 change 2: the operator records `jsonschema = "0.49.4"` in `versions.toml`. Recommended at the same time: `rmcp = "3.1.2"`, `tiktoken-rs = "0.12.0"`, `wasmtime = "47"`, `tonic = "0.14"`, `a2a_protocol = "rc-1.0 / grpc-0.3"`, `agui_profile = "uar.agui/1"`, so later changes do not each stop for one line.
- Before Round 2 change 4: the liter-llm error-typing decision (change 4 task 0.1). The executor reads the vendored liter-llm 1.18.2 error type, records whether HTTP status and `Retry-After` are exposed, and records the classification site (liter error type or driver HTTP boundary) in the change's tasks notes and the phase decision log. Round 2 does not start change 4 until that entry exists. The same read settles whether liter-llm honors a per-request base-URL override, which decides the `wiremock` test strategy below.
- Before Round 4 change 7: the sandbox decision (change 7 task 0.1: port Codex `sandboxing` or reject `sandboxed: true` at load), recorded in the decision log.
- Before Round 5: the parity report from change 6 and a live smoke run in shadow mode, both with zero unexpected differences, recorded in the decision log with corpus size and smoke set contents.

VERIFICATION PER CHANGE
Tier 0 on every edit: `cargo check --locked --no-default-features --features server-full`. Tier 1: only the change's new test target. Tier 2 at each change boundary: `cargo fmt --all -- --check` and `cargo test --locked --no-default-features --features server-full`, then `openspec validate <change> --strict`. Live smoke items are recorded in each change's tasks with the exact command and output when run; none go into GitHub Actions. `clippy --all-targets` is scoped to `-p universal-agent-runtime` per the standing gotcha.

DEFERRED AND CUT
- Circuit breaker (`failsafe`, `recloser`): no named failure; the health monitor's cooldown is wired by change 4 instead.
- `rmcp` bump and MCP 2026-07-28 semantics, MCP Apps, A2A v1.0.1, AG-UI legacy vocabulary retirement, A2UI v1.0: each has a trigger in `analysis.md` G10.
- `/readyz` dependency probes: a separate change against `deep-health-probes`.
- Guardian-style model reviewer behind Cedar: not this phase.
- `skill_activation_mode` default flip to `catalog`: a later change on Recall@10 evidence, mirroring change 10.

THE UNCOMFORTABLE THING
The immediately-valuable set is Rounds 1 and 2, five changes, and it delivers no new user-facing capability; it fixes correctness and observability defects that users experience as "the agent got confused" or "the run hung". The capability the operator asked about, subagents with real threads and skills that provably run, is Round 4 and cannot be reached sooner without building on a monolith the assessment showed has no seams. Estimated effort for a skilled agent is roughly five M-sized and four L-sized changes plus one S; if the operator wants visible capability earlier, the only honest shortcut is to run change 9 in Round 2 against the legacy path and accept re-plumbing it in Round 3.

COMMANDS TO RUN
Changes already exist; do not run `/opsx:new`. Register and execute through the KBD-owned driver, one task per turn:
/kbd-apply context-history-integrity
/kbd-apply fail-closed-tool-arguments
/kbd-apply deterministic-prompt-assembly
/kbd-apply model-path-resiliency
/kbd-apply progressive-skill-runtime
/kbd-apply typed-turn-assembly
/kbd-apply projected-mcp-runtime
/kbd-apply thread-native-subagents
/kbd-apply project-instructions-world-state
/kbd-apply typed-turn-default-flip

PLAN COMPLETE
