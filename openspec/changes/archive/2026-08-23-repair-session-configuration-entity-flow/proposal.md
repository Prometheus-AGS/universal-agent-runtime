## Why

Opening Session Configuration currently downloads and publishes all 7,248 catalog models row by row, freezing the browser, while the saved fields do not match the backend contract and do not control inference. The repair must make the sheet responsive and make every retained control function through one explicit entity authority.

## What Changes

- Register configured Provider, Model, AgentSession, and AgentSessionDraft entity contracts and transports at the application boundary, exposed through domain hooks behind `frontend/src/platform/entities`.
- Load only configured provider/model records for the selector; opening the sheet must never request the full `/api/models` catalog.
- Replace component-local session business state and duplicate REST-wrapper Zustand caches with canonical AgentSession state plus an isolated, inspectable AgentSessionDraft keyed by session and editor.
- Align the frontend and backend on the typed `model` field, load persisted session configuration, and make the saved session model govern effective inference routing.
- Implement any retained context controls end to end or remove them instead of sending ignored fields.
- Apply the design-system-resolved sheet body spacing at compact and desktop widths.

## Capabilities

### New Capabilities

- `session-configuration`: Defines responsive editing, configured-model selection, draft isolation, persistence, inference effect, and sheet spacing.

### Modified Capabilities

- `frontend-architecture-boundaries`: Require registered entity transports and domain hooks behind the entity platform facade rather than direct feature-owned graph mutations or duplicate entity caches.

## Impact

- Affects the chat session-configuration UI and view model, entity platform facade/contracts, configured provider/model projection, session API types and handlers, and inference policy resolution.
- Preserves provider API compatibility and existing realtime behavior outside the named entities.
- Expands the originally reported visual defect only as required to eliminate the observed dead-facade behavior; decorative or silently ignored controls are not acceptable.
- KBD records the frontend/backend contract and functional inference behavior as one vertical slice.
