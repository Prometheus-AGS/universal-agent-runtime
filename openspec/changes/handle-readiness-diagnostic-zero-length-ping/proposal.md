## Why

The accepted readiness diagnostic now reaches the database sign-in response but its restricted collector cannot answer the exact final, RSV-clear, server-unmasked, zero-length Ping observed before the Text result. One separately authorized, narrowly bounded Pong-and-continue path is needed to remove that diagnostic compatibility boundary without turning the collector into general WebSocket support.

## What Changes

- Add a child-local one-shot collector that accepts either one direct Text sign-in response or exactly one zero-length Ping followed by one Text response.
- Answer that exact Ping with a final, masked, zero-length Pong using a fresh four-byte masking key, without retaining frame or payload content.
- Carry one non-resetting 15-second sign-in-exchange deadline across sign-in write, Ping read, Pong write and the optional following Text response, inside the existing 30-second outer cap.
- Require credential/identity continuity, strict frame cardinality, content-free evidence, phase-boundary diagnostic tests and fresh-context independent acceptance.
- Stop at sign-in success/error. Do not resume readiness measurement, retry, generalize frame handling or modify product/runtime behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readiness-latency-diagnostics`: Add an exact, bounded Ping/Pong continuation contract for the already classified zero-length control frame.

## Impact

Writes are limited to this OpenSpec change, the active KBD child and generated KBD workflow projections. Execute may make one authenticated WebSocket attempt to the already confirmed database endpoint and may send at most one empty Pong, but it sends no namespace/database selection, query, readiness, health, inference, mutation or retry request. Runtime UX, provider compatibility, realtime entity state, product APIs, dependencies, configuration, database records and service lifecycle are unchanged.

The uncomfortable limit is that a different frame, a second Ping or a successful sign-in still leaves the database-readiness cause and recovery unresolved. This change proves only whether the exact observed Ping boundary can be crossed under its one-shot contract.
