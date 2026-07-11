## Why

Chat and Runtime Console currently ingest overlapping event representations without a declared AG-UI profile or complete reconnect/snapshot conformance.

## What Changes

- Declare UAR's AG-UI profile and event mapping.
- Build one typed adapter consumed by chat and console.
- Certify lifecycle, messages, tools, state, cancellation, replay, ordering, and errors.

## Capabilities
### New Capabilities
- `ag-ui-chat-conformance`

## Impact
Rust normalized events/SSE, React adapter and stores, protocol docs and fixtures.
