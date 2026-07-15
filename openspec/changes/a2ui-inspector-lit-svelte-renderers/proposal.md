## Why

UAR has a certified React A2UI renderer but lacks framework-parity evidence and an operator tool for understanding live A2UI traffic. Developers need to inspect, freeze, and correlate protocol messages with rendered output, while Lit and Svelte consumers need first-party renderers over the same `web_core` state model rather than divergent protocol implementations.

## What Changes

- Add `frontend/packages/a2ui-inspector/`, a development-only React tool that consumes A2UI SSE messages, synchronizes a message timeline with rendered preview and source JSON, preserves the last-good preview on malformed input, and supports explicit freeze/resume behavior.
- Export the Inspector as a Storybook addon entry without taking ownership of the full Storybook installation/configuration assigned to Change 25.
- Add `frontend/packages/a2ui-lit/` and `frontend/packages/a2ui-svelte/`, both built on `@prometheus-ags/a2ui-core` / `@a2ui/web_core`.
- Add a shared semantic conformance fixture that asserts equivalent roles, accessible names, states, and text across React, Lit, and Svelte output.
- Keep runtime UX dev-only: no production route, provider contract, or runtime API behavior changes.
- Update KBD state as Change 22 advances and passes its quality gates.

## Capabilities

### New Capabilities

- `a2ui-devtools`: Live A2UI message inspection, freeze/recovery behavior, synchronized preview/source navigation, and Storybook addon integration.
- `a2ui-lit-renderer`: Lit renderer behavior over the UAR A2UI catalog and `web_core` state.
- `a2ui-svelte-renderer`: Svelte renderer behavior over the UAR A2UI catalog and `web_core` state.
- `a2ui-cross-renderer-conformance`: Framework-neutral semantic parity requirements and fixtures across React, Lit, and Svelte.

### Modified Capabilities

None.

## Impact

- Adds three frontend workspace packages and their package-local tests/build configuration.
- Adds verified Lit and Svelte dependencies; React remains the product renderer.
- Adds no production fetches, routes, provider behavior, or backend protocol changes. Inspector SSE input is injected through a service boundary and is disabled from production bundles by package ownership and explicit entrypoints.
- Realtime state is represented as an append-only inspected-message stream with connection and freeze state; freezing pauses presentation while ingestion remains observable and resumable.
- KBD waypoint/progress records Change 22 as Codex-executed under an operator-directed harness override.
