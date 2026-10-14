# Proposal

## Why

Issue #379 requires an anonymous A2A 1.0 card, 1.0 task exchange, and a card version tied to the actual published release. Source analysis found a legacy card route and shape, older task wire values, process-local task correlation, and a hardcoded card version. Kubo's reader needs the card and version, but its Topology B gate requires all nine issue items in one published artifact.

## What Changes

Expose a public `/.well-known/agent-card.json` with a usable A2A 1.0 interface while limiting anonymous access to the card. Pin the exact 1.0 card, method, part, role, state and error contracts before implementation. Adapt the public task surface to those contracts and bind task identity to the existing durable thread and `A2AThreadService` actor execution. Derive the card's plain `x.y.z` from actual release/package provenance; do not select a tag in this proposal. Preserve an older `/agent.json` alias only as a documented compatibility path and do not advertise an older transport as 1.0.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `a2a-grpc`: add public 1.0 card and task requirements while preserving the existing legacy gRPC requirement until its selected binding is explicitly migrated or kept unadvertised.

## Impact

Production ownership is `src/uar/api/a2a/**` with a serial `src/server.rs` route/auth handoff and release-version provenance. Runtime UX gains truthful task states and errors, including pending interaction and insufficient-scope outcomes; no new UI is required. Provider routing and model execution stay with the existing actor path. Realtime task projection must come from committed thread/interaction state rather than process-local inference. KBD workflow state is unchanged by this planning artifact; the lead imports the proposed tasks through its typed lifecycle. Acceptance is deferred to the full Dot production boundary and the final installed published artifact.
