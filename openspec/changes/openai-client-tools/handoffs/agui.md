# AG-UI caller tools handoff

Phase: Execute, active waypoint `uar-outstanding-issues-v1-1-1::function-first-release-certification`. Verification gate CLOSED. No test, lint, formatting, compiler, build, packaging, review gate, or diff check was run. This is source implementation only; no acceptance claim.

Owned implementation: `src/uar/api/agui.rs`. This handoff is the only other authored file. Other teammates share the checkout and retain their edits.

## Native wire contract

The repository-pinned `@ag-ui/core` 1.0.2 sources are under `.kbd-orchestrator/phases/archive/2026-10-14-dot-feature-support/implementation/protocol-sources/`. `agui-schemas.ts` ToolSchema accepts `{name,description,parameters?,metadata?}`. Native frontend tool handoff is `RUN_FINISHED` outcome `{type:"success",pendingToolCallIds:[...]}`; `agui-types.ts` explicitly describes answering those calls in the next input's tool messages. No caller-specific interrupt reason or invented top-level protocol field was added.

Native tools normalize to shared OpenAI function descriptions. They remain caller-owned; descriptions never install host executables. Standard `TOOL_CALL_START`, `TOOL_CALL_ARGS`, and `TOOL_CALL_END` come from the durable `ClientToolsRequested` event. No result is synthesized at handoff. Approval and MCP elicitation remain existing interrupt/resume paths with exact all-open coverage.

Optional private UAR controls live at `forwardedProps.uar.client_tools`:

```json
{"tool_choice":{"type":"named","name":"lookup"},"parallel_tool_calls":false}
```

Choice values use the common enum: `{type:"auto"}`, `{type:"none"}`, `{type:"required"}`, or `{type:"named",name:"lookup"}`. Unknown extension members and unknown choice modes reject. Tool declarations remain in native `RunAgentInput.tools`. Missing continuation controls/declarations inherit the durable original binding; explicitly changed bindings reject in shared preparation.

## Native result continuation

The new wire run remains in the existing owner/agent/thread/workspace context. Its trailing native tool messages use `toolCallId` and text `content`. The adapter loads the exact source native run for verified context, then the durable caller batch for the full session owner, tenant, actual native agent ID, and source run ID. Session, protocol, and ordered pending IDs must match the sealed predecessor.

Full visible message history is normalized before shared `prepare`; tool-results-only input rehydrates the durable visible prefix before appending results. Shared preparation validates exact result coverage, schema binding and predecessor, claims once, and supplies protected canonical history with hidden host results. A new run is admitted through existing routes; caller results do not resume or replay host tool side effects. Native session identity remains the same protected AG-UI session.

Ambient native context/state remains untrusted canonical input. Fresh caller history retains a separate original visible transcript; the canonical last user content includes the existing context/state contribution once. Continuation context/state is appended to canonical history as explicitly untrusted caller content after shared preparation. It does not replace authority or saved history.

## Durability and replay

The host persists the caller batch before emitting `ClientToolsRequested`. Reconciliation seals a durable native frontend handoff. If the process loses the event after committing the batch, scoped durable batch lookup reconstructs its exact calls and handoff without submitting another execution.

Standard terminal `metadata.uar.clientToolCalls` retains the tool projection through loss of bounded native event history. Reconnect emits retained tool frames and the sealed terminal; native token history remains bounded. Caller-batch terminal offsets follow every call frame even for large batches.

For AG-UI only, shared continuation `Completed` stores a **transport admission receipt**: the serialized `AguiRunBinding`, after admission and persisted wire correlation, before exposing the stream. It does not claim model completion or tool success. Replays load that exact persisted native binding and attach to the same stream/terminal, never readmit. Missing or conflicting durable bindings fail closed. Final run state remains authoritative.

The uncomfortable case remains interruption after claim but before confirmed receipt completion: the claim is retained and cannot automatically rerun possibly admitted effects. Exact same wire POST follows its reserved binding rather than submitting another run.

## Integration ownership and unverified work

Parent owns `routes.rs`, common event declarations, protocol documentation, and final integration. Shared runtime owns `ClientToolInput`, no synthetic user append for complete caller history, durable batch publication, graph/budget continuity, and owner identity. Persistence writer owns shared preparation/claim completion, provider implementations, `visible_history`, source-run lookup, and visible-history validation.

Parent must use shared `validate_visible_history` during route admission after resolving the actual artifact, before host effects. This covers fresh AG-UI/caller histories, including catalog selectors; it must not reacquire an already claimed continuation. No final compiler result exists. Runtime behavior, protocol schema acceptance, race behavior, restart recovery, governance, and real provider propagation remain unverified while CLOSED.
