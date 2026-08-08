## Why

Runtime runs and normalized AG-UI events currently exist only in the in-memory
entity graph, so a refresh loses the trace that C-11 must inspect and replay.
Persisting that data now also closes the gap between the installed PGlite/PEM
capabilities and the product's declared local-first ownership model.

## What Changes

- Add versioned PGlite tables for runs and ordered run events, including
  terminal phase timings and stable official event identity.
- Persist lifecycle, tool, state, custom, and raw events incrementally while
  coalescing text and reasoning content deltas into one durable row per logical
  message span.
- Flush coalesced spans at their official end frame and at run termination when
  the current transport supplies no explicit end frame.
- Initialize PEM's PGlite persistence adapter and local-first graph runtime
  before realtime sync, replacing any need for an application-owned outbox.
- Generate the RuntimeRun and RuntimeAgUiEvent JSON schemas from the migration
  SQL through the existing platform entity facade.
- Record completion in canonical KBD state before archiving this OpenSpec
  change.

## Capabilities

### New Capabilities

- `frontend-local-first-persistence`: durable client-owned run/event records,
  bounded delta write behavior, graph snapshot hydration, and offline action
  replay ownership.

### Modified Capabilities

None.

## Impact

- **Runtime UX:** completed and interrupted traces survive refresh and can be
  read offline by the later run-trace surface.
- **Provider compatibility:** provider/model APIs and routing do not change;
  persistence consumes the already-normalized AG-UI profile.
- **Realtime state:** PGlite graph hydration completes before the existing
  realtime transport starts, preventing an older local snapshot from
  overwriting newly-arrived server state.
- **Frontend platform:** `platform/pglite/`, the entity bootstrap/facade, and
  the chat stream store gain the durable run/event path; no new dependency is
  introduced.
- **Workflow:** C-07 completion must be written through the canonical KBD
  change transition before OpenSpec archive.
