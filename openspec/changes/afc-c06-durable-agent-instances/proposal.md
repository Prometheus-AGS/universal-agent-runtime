# Proposal

## Why

UAR's current `ThreadService` is attached to one fresh root run, and actor handles are process-local. Neither is a durable address for later turns after passivation or restart. Agent Fabric Convergence C06 requires an owner- and workspace-scoped logical agent instance that survives those boundaries without keeping a model loop alive.

## What Changes

- Add durable ordinary-agent instance records with distinct request, on-demand, and opt-in resident activation profiles. Each admitted turn gets a fresh root run through the existing kernel.
- Persist bounded incoming work, activation state, and single-host ownership epochs. Serialize mutating turns per instance while keeping authorized status and cancellation available during a blocked turn.
- Define activate, passivate, drain, disable, and restart behavior with inspectable failure states, repeat-safe lifecycle hooks, and bounded retry policy.
- Expose additive authenticated instance administration and capability discovery. Existing run, actor, A2A, and service-instance placement contracts remain supported.

## Capabilities

### New Capabilities

- `durable-agent-instances`: Logical instance identity, admission, bounded turns, lifecycle, recovery, and observability for ordinary agents within one UAR process.

### Modified Capabilities

None. `agent-thread-kernel` remains root-run scoped; `service-instance-placement` identifies the UAR service process; and `collaboration-package-catalog` continues to report team activation as unsupported. This change composes those contracts without changing their existing requirements.

## Impact

This is the UAR product slice of initiative C06. It touches trusted runtime admission, actor/thread integration, persistence providers, and authenticated administration/discovery. Realtime clients need additive committed instance/turn state and a truthful replay posture; provider/model selection continues through existing binding and run policy rather than a new model path. The runtime UI may show instance state and recovery controls later, but no frontend redesign is required for this C06 slice. KBD initiative tasks C06.1-C06.3 must be reconciled to the completed product evidence and exact source revision; this proposal alone does not advance them.

The uncomfortable boundary: a durable logical identity does not make a partially executed external effect safe to replay. Restart must preserve uncertainty and require reconciliation before another effectful turn. Cross-host takeover, team activation, remote member execution, and Surreal Memory API changes are outside this change.
