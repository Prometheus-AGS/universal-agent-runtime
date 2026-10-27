# Task 2.3 source design: live roster and visible selection

Date: 2026-10-25. Phase: `uar-post-lifecycle-verification-readiness`.
Waypoint inspected: revision425 / plan116, next task2.1. This is an advance source/design packet for task2.3, not a task transition or implementation-complete inventory. Verification **CLOSED**. Only source inspection and this documentation write were performed. No production files changed, no runtime/provider/service invocation, no test/build/check/lint/format/review gate. Other writers share the checkout; this packet does not authorize editing their files.

## Required result

The mounted orchestrator classifies intent against the current eligible registered local roster, executes the selected persisted artifact through the trusted thread service, retains permissioned follow-up dialogue, and emits a redacted selection event before the parent stream delivers the selected response. AG-UI reconnect consumes retained events without reexecuting the child. A skill match is never an agent selection.

## Observed path and gaps

| Source | Observed behavior |
| --- | --- |
| `src/server.rs:1846` | Mounts `defaults::orchestrator_graph()` on RunManager. |
| `src/uar/defaults.rs:161` | Graph has fixed RouterNode options `general-purpose` and `rust-reviewer`; one AgentNode per option. The factory path is `defaults.rs`, plural. |
| `src/uar/runtime/graph/nodes/router_node.rs` | LLM prompt classifies the latest user request, but only among constructor options. Invalid output falls back to the first option; it does not inspect the registered roster. |
| `src/uar/runtime/manager.rs:2921` | `resolve_registered_agent` and `list_registered_agents` already delegate to the catalog domain functions. |
| `src/uar/domain/agent_store.rs` | Listing validates persisted definitions and reconciles built-ins; explicit resolution prefers persisted definitions. Unknown custom IDs fail. Deleted nonprotected built-ins are synthesized again, so deleting `rust-reviewer` is not a suitable custom-removal fixture. |
| `src/uar/runtime/manager.rs:7246,7503` | Installs opaque GraphThreadDelegate in GraphContext; graph executes under host checkpoint/event boundaries. |
| `src/uar/runtime/graph/delegation.rs` | Checks root identity, explicit spawn/wait controls, host admission, approval and root tool budget; `controls.spawn[_graph]` returns a persisted child before `wait_first_turn`. Caller event sink currently exists only inside the optional caller-tools host. |
| `src/uar/runtime/thread/service.rs:670` | `spawn_inner` reloads artifact through executor, intersects parent policy with it, checks budgets/admission, filters fork history, persists child, launches retained worker. Keep this path. |
| `src/uar/runtime/thread/kernel.rs:446` | `CapturedThreadKernel::artifact` checks owner/live root and resolves the current registered artifact through manager. |
| `src/uar/runtime/graph/nodes/agent_node.rs` | Uses thread delegate; persists typed outcome/thread/output into graph state. Defaults to HistoryForkMode::None. Node ID determines `_agent_output_{node_id}`. |
| `src/uar/runtime/manager.rs:7615` | Reads `_route`, then `_agent_output_{route}`, records and emits `[route]` attribution after child completion. Dynamic node IDs require adjusting this lookup. |
| `src/uar/runtime/thread/spawn.rs` | Existing Full/LastTurns fork excludes parent system prompts and tool traffic and retains final assistant responses. Use this mechanism rather than raw graph messages as child authority. |
| `src/normalized.rs:120` | Lower-level LLM NormalizedEvent has Custom. This is a different enum from runtime events. |
| `src/uar/domain/events.rs` | Runtime NormalizedEvent has SkillActivated but no generic Custom or agent-selection variant. |
| `src/uar/runtime/graph/turn.rs` | Transcript observer drops lower-level Custom in catch-all; adding a lower-level Custom producer alone cannot deliver selection. |
| `src/uar/api/adapters.rs:69,137` | Transport-free official adapter maps typed runtime events to CUSTOM. Existing selection-shaped event is skill activation only; no equivalent agent event observed. |
| `src/uar/api/agui.rs:635` | Mounted run API uses its own Projector; no generic Custom pass-through. Both this projector and adapters must be changed. |
| `src/uar/api/agui.rs:660` | Stream uses retained native history, native cursor plus per-frame index, wire run/thread IDs, and Last-Event-ID filtering. Do not dispatch routing from reconnect. |
| `frontend/src/platform/agui/runtime-chunk-projection.ts:106` | Unknown CUSTOM events become raw chunks. Merely sending a new event will not produce a visible selected-agent indicator. |

The uncomfortable constraint: current catalog listing is global and has no per-agent enabled/owner field. Do not invent an ownership filter or claim listing alone proves execution eligibility. Eligibility must use existing parent policy intersection and host execution admission; final spawn remains authoritative.

## Minimal production shape (proposed additions, not existing APIs)

Preserve fixed RouterNode and AgentNode behavior for existing consumers. Add a live-roster mode to RouterNode and a selected-local-target mode to AgentNode, used only by the mounted default orchestrator. Keep the graph topology static: `router -> selected-agent`; make its selected artifact dynamic. This avoids rebuilding a startup graph for every catalog mutation.

1. Extend GraphThreadDelegate with an always-installed routing host holding the manager and parent RuntimeEventSink, separate from optional caller-tools capture. Proposed methods: `eligible_local_agents()` and `execute_selected(...)`. These names are new interfaces, not claims that such methods exist. Host roster loading uses `manager.list_registered_agents()` on each routing invocation. Candidate descriptions contain only ID, title, description, tags and catalog revision; exclude self/orchestrator graph entry to avoid recursive selection, malformed definitions and targets whose existing parent `ThreadPolicy::intersect` fails. Use existing host admission checks for executable bindings if made available through the host adapter; do not duplicate policy decoding. No hidden prompt or credentials enter routing metadata.
2. RouterNode live-roster mode obtains this host-owned candidate set through `ctx.thread_delegate`, builds a closed-option intent-classification prompt from descriptions/tags and the permitted conversational input, and uses the existing `ctx.driver` path. It stores `_route` as exact selected artifact ID and a typed host-comparable selection receipt in state. Reject empty roster, driver error and invalid/nonmember output. Do not retain the existing first-option fallback in this new mode. Candidate content is untrusted data, not instructions. No skill-registry conversion or synthetic Skill records.
3. AgentNode selected-target mode reads the chosen ID but delegates through the new selected execution method, which checks membership/current catalog and ordinary spawn admission. Use `HistoryForkMode::Full` explicitly for this orchestrator path; ThreadService filters canonical dialogue and destination context assembly continues to enforce token/policy limits. Fixed AgentNode users retain existing defaults. New mode writes `_agent_output_selected-agent`, `_agent_thread_id_selected-agent` and typed result using its stable node ID. Manager reads the stable output key while keeping `_route` for attribution. A missing route/output is an error, never a default specialist.
4. In the host delegate, after successful `controls.spawn[_graph]` returns and before `wait_first_turn`, publish typed AgentSelected through the root sink using returned child thread ID. Never claim a thread identity before persistence confirms it. The requesting parent stream does not emit selected output until after this event. The service launches the child before returning, so this placement guarantees ordering of parent-visible selection and parent-visible response, not that the child cannot finish internally first. If the intended promise is selection before any child model execution, the service needs an explicit host hook between child persistence and launch; that is a larger contract and must be chosen explicitly.
5. For roster mutation during classification, re-resolve the selected ID and compare its catalog revision with the classified candidate before spawning. Missing/changed selection fails this invocation; next request reloads current roster. Existing spawn re-resolves again, so a revision check outside ThreadService alone does not atomically bind the classification revision to the admitted artifact. To promise exact revision binding, add an optional expected-artifact revision to host-only spawn capture/admission and compare inside `spawn_inner` immediately after artifact resolution. Do not add it as model-supplied authority. If this is deferred, document that selection classifies a request-time roster while execution uses the latest artifact at spawn; record executed source identity from the child run snapshot, not candidate metadata.

No new dependency, background refresh task, retry loop, kernel write capability, or GraphContext persistence writer is needed. Preserve cancellation ownership and existing retained worker lifecycle; do not hold a roster/catalog lock across LLM await. Existing GraphContext can remain unchanged if the new capability lives on its opaque delegate.

## Additive wire contract to freeze before producer edits

Proposed runtime variant: `AgentSelected { run_id, agent_id, agent_title, thread_id, selection_method }`. `thread_id` is the confirmed selected child thread. `selection_method` is the closed public outcome label `intent`; it is not a rationale or chain of thought. Add source revision only if it is the actually admitted child artifact revision.

Official AG-UI CUSTOM name: `uar.agent.selected`.

```json
{
  "type": "CUSTOM",
  "name": "uar.agent.selected",
  "runId": "requesting-wire-run-id",
  "threadId": "requesting-wire-thread-id",
  "value": {
    "agent": { "id": "selected-registered-id", "title": "Public catalog title" },
    "threadId": "persisted-selected-child-thread-id",
    "selectionMethod": "intent"
  }
}
```

The outer run/thread identify the requesting protocol run; the nested thread identifies the actual selected child. For native adapters without wire binding, outer run remains native run ID and outer thread should use the actual parent session when supplied; do not label child thread as the parent. Existing `profile`, event ID and sequence enrichment stays owned by existing framing. Titles and IDs must be scrubbed using the run secret scrubber before emission; emit no prompt, full roster, user text, classification model response, credentials or hidden reasoning. A selected event reports accepted child dispatch, not successful completion.

Implement runtime variant mapping in transport-free `adapters.rs` plus mounted `api/agui.rs::Projector::project`. The generic adapter closure currently sets threadId equal to runId; a selection-specific mapping can retain true parent thread when supplied. Legacy server AG-UI projection should use its documented custom extension path. OpenAI/Anthropic token streams must keep valid protocol output; do not inject unsupported frames. Lower-level `src/normalized.rs::Custom` may carry the event through an existing adapter path only if that path is explicitly mapped; the runtime producer should use the typed event directly.

Reconnect: store this event in the normal ordered root event history. Rebuild Projector from retained history and filter already-seen frames with existing cursor logic. No special spawn call, reclassification or synthetic selection on reconnect. Existing retention-gap behavior remains truthful. Client support requires an explicit `uar.agent.selected` projection and visible chunk/indicator using nested child ID for correlation; unknown raw-chunk storage is insufficient. Frontend implementation must separately follow UI skill routing and normalized entity-hook rules.

## Proposed ownership and integration order

| Owner boundary | Exact files | Change |
| --- | --- | --- |
| Runtime worker, after lead assignment | `src/uar/runtime/graph/delegation.rs` | Host current roster, membership/revision re-resolution, post-spawn pre-wait selection emission; preserve controls/admission/caller capture. |
| Runtime worker, after lead assignment | `src/uar/runtime/graph/nodes/router_node.rs`, `src/uar/runtime/graph/nodes/agent_node.rs` | Add modes above without changing existing fixed constructors. No module exports needed if modes use associated constructors. |
| Defaults writer | `src/uar/defaults.rs` | Two-node dynamic graph, filtered dialogue fork, remove static two-ID routing instruction while preserving durable operator fields under existing seed rules. |
| Lead | `src/uar/runtime/manager.rs` | Install always-available routing host/sink, capture safe parent session, stable output-node lookup, preserve `_route` attribution, scrub public metadata. |
| Lead event/schema | `src/uar/domain/events.rs`, `src/uar/api/adapters.rs`, `src/uar/api/agui.rs`, applicable exhaustive event matches | Typed AgentSelected and both official CUSTOM projections; include runtime event retention/projection match updates required by the new enum variant. |
| Lead protocol integration if required | `src/normalized.rs`, `src/server.rs` | Only necessary lower-level/custom/legacy interface bridge, not the sole event producer. |
| Host-only exact-revision binding if selected | `src/uar/runtime/thread/service.rs`, `src/uar/runtime/thread/control.rs`, host capture type in `src/uar/runtime/turn/client_tools.rs` or a distinct host-only input | Concrete capture design must be agreed before edits; current `AgentSpawnRequest` has no expected revision field. |
| UI owner, later explicit allocation | `frontend/src/platform/agui/runtime-chunk-projection.ts`, `frontend/src/shared/content/chunk.ts` and existing renderer selected after source tracing | Visible typed selected-agent chunk and correlation. Do not invent a renderer path before inspection. |

Integrate schema first, then delegate contract and graph modes, then manager/default graph, then consumers and acceptance fixtures. Source inspection may continue under CLOSED; all compilation, tests, review-driven verification and product acceptance remain deferred to whole-phase inventory and exact operator permission.

## Acceptance fixture requirements for the lead

Author, do not execute yet: genuine custom specialist with unique public description/tags and task-specific response predicate; classifier choice must equal the custom ID and actual child artifact snapshot must match the registered artifact. Update description/prompt, route next request, then remove custom and prove it is no longer executable. Include invalid classifier output, unknown ID, denied intersection/admission, roster read failure, and persisted child failure as failure cases rather than fallback success. A follow-up must consume a nonce from prior permitted dialogue and retain selected-agent attribution. Capture ordered native/AG-UI selection and response events; assert child ID is real, selection precedes visible response completion, reconnect uses the same native run/child and does not increment child count. Both durable adapters require the same positive predicates. No runtime outcome is established by this design packet.

## Handoff status

Changed only this file. Commands were `cat`, `sed`, `rg` and source file listings plus `apply_patch` for this documentation. Rust workspace, best-practices and async-pattern skills were read; applied rules are opaque host capability boundaries, typed fallible results, preserving retained cancellation ownership, immutable candidate snapshots and no locks across classifier await. No unrequested production change or new guard was added. Proposed guards trace to explicit requirements: no unregistered/stale execution, policy intersection, privacy, real persisted child attribution and failure visibility. All proposed production behavior remains unimplemented and unverified. Exact source commit identity is intentionally left to the lead's permitted whole-phase inventory; concurrent writers can change source after inspection.

## Implementation handoff — supersedes initial design-only status

Lead assigned and confirmed interfaces before edits. Ownership remained exactly the following files plus this handoff; all other edits belong to teammates.

| File | Authored production change |
| --- | --- |
| `src/uar/runtime/graph/delegation.rs` | Adds host-only `with_agent_routing(manager, source_agent_id, events)`, independent of optional caller-tool capture. Adds serializable candidate/roster types and `eligible_local_agents()`: loads registered catalog each call, enforces source/run/native spawn+wait scope, excludes source and orchestrator entry, filters parent-policy intersection, sorts by ID and hashes exact candidate JSON (including full artifact content revision) with SHA-256. Adds `execute_selected(run_id, step, request, graph_state, roster_revision)`; revalidates roster hash and exact target after admission/approval immediately before calling the existing trusted spawn, closes failed admission on revalidation error, emits typed AgentSelected after confirmed spawn and before wait/response forwarding. Fixed execute and remote behavior remain their existing paths. |
| `src/uar/runtime/graph/nodes/router_node.rs` | Adds `RouterNode::for_registered_agents(id)` while preserving fixed constructor behavior. Uses the existing governed `ctx.driver` with no tools, JSON public catalog metadata and canonical dialogue filtered through existing `fork_history(Full)`. Dynamic mode accepts only exact roster member IDs, fails without first-option fallback and stores `_route` plus `_agent_roster_revision`. Invalid output is not echoed in the public error or dynamic warning log. |
| `src/uar/runtime/graph/nodes/agent_node.rs` | Adds `AgentNode::selected_local(id)`. Requires route and roster revision and invokes execute_selected; stable node ID still determines outcome/thread/output keys. Existing explicit local and remote constructors retain behavior and history defaults. Root factory must opt into `with_history_fork(Full)`. |

### Confirmed lead integration contract

The final typed runtime variant differs from the initial JSON sketch above. The lead owns this schema and all consumer changes:

```rust
AgentSelected {
    run_id: String,
    source_agent_id: String,
    selected_agent_id: String,
    selected_agent_name: String,
    selection_method: String,
    roster_revision: String,
}
```

The delegate emits `selection_method = "intent"`. `roster_revision` identifies the entire eligible roster snapshot, not an atomic child artifact pin. This agreed event has no child-thread field. The persisted child identity remains in existing graph result keys; do not describe this schema as delivering child-thread correlation that it does not contain. Parent run/thread correlation comes from the protocol projection. The lead must explicitly resolve whether the selected-child identity requirement needs an additive event field or a separate existing projection.

Manager must install `.with_agent_routing(self.clone(), artifact.id.clone(), Arc::new(emitter.clone()))` on the GraphThreadDelegate regardless of caller-tool presence. Default graph must use `RouterNode::for_registered_agents("router")`, `AgentNode::selected_local("selected-agent").with_history_fork(HistoryForkMode::Full)`, and direct `router -> selected-agent` edge. Manager reads `_agent_output_selected-agent` while `_route` supplies actual selected attribution. Defaults' old two-specialist instructions must describe the registered roster. Lead owns `domain/events.rs`, both official AG-UI event projections, any exhaustive-match integration, and secret projection of public selected-agent display text. Lower-level Custom alone is insufficient.

### Explicit remaining limitations and unverified claims

- Revalidation occurs after potentially long approval waits and fails a changed eligible roster without retry. The subsequent thread spawn independently re-resolves and intersects the current artifact. A mutation in the small interval between roster revalidation and ThreadService artifact resolution is not atomically excluded; there is no exact revision pin in the existing host spawn contract. This limitation was explicitly accepted by the lead for this file-owned slice.
- Event emission is before parent-visible child output/handoff. Child worker launch happens inside spawn before spawn returns, so this does not guarantee the child has not already produced or internally completed its response.
- Policy intersection filters the candidate roster; full execution-binding/resource admission remains authoritative in ThreadService, so an otherwise eligible classified candidate can still fail execution admission. No fallback masks this failure.
- Follow-up history is filtered by the existing host history-fork implementation, not raw system/tool inheritance. Real multi-call usefulness remains unproven.
- No separate test file was authored: a meaningful real-path test needs lead-owned runtime/persistence/provider fixture setup; mock-only helper tests would not establish this outcome. The lead's user-outcome acceptance fixtures remain required and must cover the cases above.
- No tests, build, compiler, formatter, lint, diff check, provider call, service startup, import probe or review-driven verification was run. Source reads (`cat`, `sed`, `rg`) and `apply_patch` were the only actions. Gate remains **CLOSED**. Compilation, compatibility and production acceptance are explicitly unverified.
- No unrequested changes were added. New rejection conditions map to current-roster membership, correct host/run identity, existing spawn/wait authority, valid permitted dialogue and no invented selection; no speculative retry or fallback was added.
