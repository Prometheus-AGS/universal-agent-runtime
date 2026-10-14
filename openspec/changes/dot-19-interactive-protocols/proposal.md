# Proposal

## Why

Issue #379 requires human approval through A2A, AG-UI `RunAgentInput` interruption/resume, and MCP client elicitation. Current source projects approval as custom AG-UI output, lacks the requested ingress/elicitation handler, and can take a governance-off return before descriptor Required approval is evaluated. Existing durable approval CAS, protected continuation and host decision routes should own these interactions.

## What Changes

Correct Required approval precedence, project the exact pending decision as A2A input-required, and map AG-UI interrupt/resume to the same durable host decision. Add a run-scoped MCP client elicitation handler on supported transports that returns accept/decline/cancel through durable interaction state. Elicitation acceptance does not authorize a tool call. Pin the exact AG-UI and MCP contract versions/schemas during the implementation task; no unverified version is asserted here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-approval-workflow`: descriptor Required approval remains effective and interactive decisions retain exact identity.
- `runtime-console-governance-certification`: modify the existing governance-Off bypass and warning requirements so explicit descriptor Required approval is never bypassed, while existing risk-policy bypass remains scoped to eligible local Off mode.
- `ag-ui-chat-conformance`: add `RunAgentInput` interruption and `resume[]` behavior.
- `mcp-runtime-projection`: add run-scoped client elicitation without widening tool authority.

## Impact

The existing actor, approval ledger, continuation claim, MCP run bindings and host admission remain authoritative. Runtime clients see durable pending/denied/resumed outcomes rather than inferred success; no new UI is required. Provider/model routing is unchanged. Realtime A2A/AG-UI views project committed host decisions, not a separate frontend state machine. KBD task state is updated only by the lead through typed task import/execution. All product checks wait for the full Dot production boundary.
