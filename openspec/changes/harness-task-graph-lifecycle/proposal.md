## Why

Protocol adapters and graph execution need shared durable identity and observable terminal behavior, including recovery while approvals or child work are outstanding.

## What Changes

- Unify owner-scoped A2A task/context and AG-UI run/thread lifecycle and persisted artifacts.
- Correlate graph, node, iteration, tool and child events with one terminal outcome.
- Preserve deny-final approval, shared limits, cancellation cleanup and recovery without tool replay.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `agent-thread-kernel`: Extend the existing behavior with this change's testable contracts.
- `multi-agent-orchestration`: Extend the existing behavior with this change's testable contracts.

## Impact

scope: Existing trusted host thread service, A2A/AG-UI adapters, graph scheduler, checkpoint and RuntimeStep event projections; no UI redesign.

Dependencies: harness-model-profiles-routing The phase index defines the shared-file serialization order. All mutation authority remains in trusted hosts; dependency pins remain unchanged.

Runtime UX: explicit typed failures and additive redacted events; preserve existing wire contracts and frontend/entity ownership. Provider compatibility: only documented and tested destinations qualify; unsupported paths remain explicit. Realtime state derives from persisted host state, never agent-only memory.

KBD impact: this Spec stage supplies proposals for Plan; implementation tasks remain unchecked. Five F4–F8 external-candidate research prerequisites and the exact provider-profile evidence matrix must be resolved before affected implementation commitments. No deployment, release, runtime test result or completed runtime fix is claimed.

