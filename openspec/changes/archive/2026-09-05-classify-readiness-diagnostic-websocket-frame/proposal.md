## Why

The database-readiness diagnostic stopped because its restricted WebSocket client rejected the first sign-in response without retaining enough structural metadata to identify the frame form. The historical frame cannot be recovered, so one separately authorized header-only observation is needed to remove that ambiguity without retaining application content or changing runtime behavior.

## What Changes

- Add a bounded, child-local observer that captures only allowlisted WebSocket handshake and framing metadata from one new sign-in response.
- Classify the new observation separately against RFC 6455 header constraints and the prior collector's narrower acceptance predicate.
- Require credential-safe evidence, no payload retention, stop-on-first-error behavior, and fresh-context independent review.
- Register one KBD diagnostic change with four sequential tasks; Plan itself performs no connection or product mutation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readiness-latency-diagnostics`: Add a bounded, payload-free contract for classifying a newly observed WebSocket frame that blocked the existing diagnostic.

## Impact

Writes are limited to this OpenSpec change, the active KBD child, and KBD workflow projections. Execute may make one authenticated WebSocket attempt to the already configured database endpoint, but it makes no readiness, query, inference, record-mutation, restart, build, install, or deployment request. Runtime UX, provider compatibility, realtime state, product APIs, dependencies, configuration, database records, and service lifecycle are unchanged.

The uncomfortable limit is that a new observation cannot prove the discarded historical frame had the same structure. A non-reproducing or incomplete attempt remains explicitly inconclusive.
