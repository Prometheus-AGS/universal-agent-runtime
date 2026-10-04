## Why

C09 executes durable team tasks but does not own the approved classify → draft → human-decision operation. C10.1 requires pinned workflow progression and an exact-artifact decision that survives restart without another model turn or external effect.

## What Changes

- Add the closed `prometheus.workflow-execution` version 1.0.0 interpretation and supported-definition diagnostics.
- Persist scoped runs, original task/attempt links, waits and idempotent control receipts in the existing catalog CAS.
- Reuse C09 admission, dispatch, settlement, budget and recovery; narrow workflow context and disable tools.
- Add authenticated workflow administration routes for Boss; qualification remains distinct from implementation.

## Capabilities

### New Capabilities
- `workflow-execution`: pinned two-step internal feedback drafts and durable operator decisions.

### Modified Capabilities
None.

## Impact

UAR owns `src/uar/domain/workflow_execution.rs`, `src/uar/compiler/collaboration/workflow_execution/`, `src/uar/api/collaboration/workflow_execution.rs`, and additive catalog/module/capability registration. Minimal C09 admission, context, policy and runtime progression integration preserves ordinary teams. Boss binds typed routes in its separate repository. No dependency, provider routing, service or port changes. Runtime UX receives inspectable revisioned records; no new realtime transport. Initiative KBD updates remain parent-owned; the unrelated local waypoint is unchanged. Approved source comparison selected UAR; completed-path measurement remains required and is not a source-code claim.
