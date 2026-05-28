## Why

`createUarSseAdapter` is the single boundary between SurrealDB live-query events (via SSE) and the entity graph. Its payload-shape contract is consumed by every `useEntity*` reader transitively. A regression in the event-name → `EntityChange.op` mapping would silently swallow updates without any compile-time signal.

## What Changes

Author `frontend/src/lib/realtime/__tests__/uar-sse-adapter.test.ts`:

- Replace global `EventSource` with `FakeEventSource` via `vi.stubGlobal("EventSource", FakeEventSource)` in `beforeEach`. The fake supports `addEventListener(name, handler)` and `dispatch(name, dataObj)` so the test can synthesize SSE frames.
- `const adapter = createUarSseAdapter({ topic: "providers", entityType: "Provider" });`
- `const handler = vi.fn();`
- `adapter.subscribe({ label: "test" }, handler);`
- Dispatch each event name and assert handler payload:
  - `create` → `{ changes: [{ op: "insert", type: "Provider", id: "p1", data: { id: "p1" } }] }`
  - `update` → `{ op: "update", ... }`
  - `delete` → `{ op: "delete", ... }`
- Unsubscribe path: call the returned `UnsubscribeFn`; further dispatched events do NOT call the handler.
- Status callback: register a status listener; verify it transitions to `connected` on `onopen`.

## Acceptance

- Test passes.
- Changing the event-name mapping (e.g. accidentally `op: "insert"` on `update` events) makes the test fail.
