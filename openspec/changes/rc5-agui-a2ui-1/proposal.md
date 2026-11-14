## Why

The RC5 native chat repair ships alongside the final AG-UI 1.0 and A2UI 1.0 wire contracts, using the released Prometheus Entity Management 5.0.0 graph and compatibility consumer.

## What Changes

- Align the dedicated AG-UI SSE endpoint with the final upstream 1.0 schema.
- Add A2UI 1.0 publication, actions and replay to the existing trusted host and UAR renderer.
- Retain legacy A2UI history and certify the UAR Linux amd64 lane.

## Capabilities

### Modified Capabilities

- `ag-ui-chat-conformance`
- `a2ui-uar-renderer`

## Impact

Rust protocol handling and replay projection, UAR's bundled renderer, dependency pins, installed API acceptance, and operator documentation.
