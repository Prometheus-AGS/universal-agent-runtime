# Design

## Context

The child architecture analysis identified the verified-local governance bypass before `ApprovalClass::Required` is included in the manager approval decision. It also found durable approval CAS, protected continuation and an exact-approval-ID host route already present, AG-UI approval projected as CUSTOM-only, OpenAI-shaped ingress, and MCP client unit handlers across inspected paths. The existing governance-Off spec says risk approvals are bypassed; this change narrows that contract for an explicit descriptor Required class rather than silently contradicting it.

## Goals / Non-Goals

Meet issue #379 items 3, 4 and 6 with one host-owned interactive authority. A2A approval must be visible as input-required and resolve once; AG-UI must accept pinned `RunAgentInput`, emit the specified interruption outcome and consume `resume[]`; MCP client elicitation must reach the current run and durably return accept/decline/cancel. A new approval ledger, executor, checkpoint state machine, frontend approval authority or implicit tool authorization from elicitation is outside scope.

## Decisions

1. Before implementation, record authoritative AG-UI `RunAgentInput`/SSE/interrupt/`resume[]` contract sources and the selected MCP elicitation capability/request/response contract. Do not infer version-specific field names from current CUSTOM events. The exact schemas become implementation fixtures, authored before but executed only after the full-phase boundary.
2. Descriptor `ApprovalClass::Required` is an explicit host/tool contract. It must be evaluated before any eligible governance-Off bypass. Cedar, effective-policy denial and risk-derived human approval retain their existing On/Off policy semantics; recovered, child, host-required and explicit Ask/Deny conditions retain their stronger existing checks. This is a deliberate correction to the existing governance-Off spec, including its operator warning text.
3. Bind A2A task input-required to the exact durable approval ID and protected continuation. The client reads current pending/expired/invalidated state. A decision requires current owner, descriptor, payload, binding and epoch; CAS admits one valid continuation. Denial terminates without invoking the tool. An A2A task mapping must not store a second decision.
4. AG-UI ingress authenticates and admits a `RunAgentInput` through the existing host run path. A pending decision ends the current stream with the selected spec `RUN_FINISHED` interruption outcome and durable identity. `resume[]` resolves that exact host interaction under current authority; replay and reconnect retain stable event/task attribution without fabricating execution completion.
5. Install a run-scoped MCP client handler on every supported transport construction path. Its advertised elicitation capability is available only when the run's governed interaction channel is available. The elicitation request and accept/decline/cancel response are distinct from a tool approval record; even accept proceeds only through normal descriptor/policy/approval admission. Disconnect or restart leaves a durable pending/terminal disposition, not silent acceptance or replay.

## Risks / Trade-offs

The uncomfortable thing: governance Off is documented today as permitting risk-based execution without approval. A broad change to all Off-mode tools would break that behavior. This design changes only explicit descriptor Required precedence and updates the old requirement. Cross-protocol projections can disagree unless all read the same committed approval/interaction record; final proof must race decisions and restart two runtimes.

## Migration Plan

Keep existing approval IDs, CAS records, checkpoint claims and MCP run bindings. Add wire projections and ingress additively after dot-07, dot-11 and dot-18 production contracts. Give `src/uar/runtime/manager.rs`, `src/server.rs` and `src/mcp/registry.rs` one serial writer each. The 19 task 1 AG-UI/approval contract precedes task 2 MCP elicitation; neither waits for unrelated schedule/follow-up verification. No legacy approval record may be silently upgraded into an executable request.

## Verification

Author protocol fixtures and restart/race scenarios during production implementation. Execute no tests, builds, validators, lint, typecheck, QA detectors or real-path probes until the lead records every Dot production item complete. At that boundary, validate A2A and AG-UI against selected schemas; prove Required never bypasses with governance On or Off; race duplicate approve/deny, expiration and changed bindings; prove at most one effect. Exercise each supported MCP transport's accept/decline/cancel, interruption, restart and tool-admission separation in local and shared profiles. Dot-17 repeats the conformance through the installed published artifact.

## MCP task 2 source contract

The client profile is MCP2025-11-25 on exact rmcp3.4.1 source commit
9427a929959e665e0d12e9395f674026baf4bd48. See
[the governed elicitation contract](../../../docs/protocols/mcp-elicitation.md)
for official source links, every client construction path, durable CAS schema,
authenticated host routes, AG-UI response union, root/child channel lifetime and
unexecuted fixtures. The existing SDK high-level tool method can drive automatic
multi-round calls; this implementation deliberately selects its single-call peer
method so human input never becomes a hidden tool resend. A channel UUID partitions
run connections; the root finalizer owns closing/reaping them. Input waiting is
bounded separately from ordinary execution time and never extends credential or
delegation validity. Source is authored; no verification has run.
