## ADDED Requirements

### Requirement: Public Observation Surface
`prometheus-entity-management` SHALL export `subscribeDevtoolsEvent(cb: (event: DevtoolsEvent) => void): () => void` from `src/engine.ts`, re-exported from `src/index.ts`. The `DevtoolsEvent` discriminated union type SHALL also be re-exported.

#### Scenario: Public export resolves
- **WHEN** a consumer imports `subscribeDevtoolsEvent` from `@prometheus-ags/prometheus-entity-management`
- **THEN** the import MUST resolve and return a function whose signature accepts a single listener and returns a no-arg unsubscribe function.

#### Scenario: Type export resolves
- **WHEN** a consumer imports `type { DevtoolsEvent }` from the same package
- **THEN** the type MUST resolve to a discriminated union with `kind: "upsert" | "patch" | "unpatch" | "clearPatch" | "list"` as the discriminator.

#### Scenario: Multiple subscribers
- **WHEN** two distinct callbacks are passed to `subscribeDevtoolsEvent` and an event fires
- **THEN** each callback MUST be invoked exactly once with the same `DevtoolsEvent` payload, in registration order.

#### Scenario: Unsubscribe stops delivery
- **WHEN** a callback's `UnsubscribeFn` is invoked, and an event subsequently fires
- **THEN** the unsubscribed callback MUST NOT be invoked; any remaining subscribers MUST still receive the event.

### Requirement: Event Payload Shape
The `DevtoolsEvent` union SHALL distinguish event kinds by a `kind` discriminator field and SHALL carry the documented per-kind payload.

#### Scenario: upsert payload
- **WHEN** an `upsertEntity` op fires a devtools event
- **THEN** the event payload MUST be exactly `{ kind: "upsert", type: EntityType, id: EntityId, data: Record<string, unknown>, at: string }`, where `at` is an ISO-8601 UTC timestamp.

#### Scenario: patch payload
- **WHEN** a `patchEntity` op fires a devtools event
- **THEN** the event payload MUST be exactly `{ kind: "patch", type: EntityType, id: EntityId, patch: Record<string, unknown>, at: string }`.

#### Scenario: clearPatch payload
- **WHEN** a `clearPatch` op fires a devtools event
- **THEN** the event payload MUST be exactly `{ kind: "clearPatch", type: EntityType, id: EntityId, at: string }`.

#### Scenario: Forward-compatible kinds
- **WHEN** the `DevtoolsEvent` union is consulted
- **THEN** it MUST include `kind: "unpatch"` and `kind: "list"` as valid discriminator values, even though this change does not instrument those op sites; downstream consumers (W5+ event bus, W6+ panel) rely on the union being forward-compatible without further type-only changes.

### Requirement: Op-Site Instrumentation
Every mutating operation exposed through `src/graph-actions.ts` SHALL call `notifyDevtools(event)` immediately after the underlying store mutation.

#### Scenario: upsertEntity instrumentation
- **WHEN** `graph-actions.ts` `upsertEntity(type, id, data)` is invoked
- **THEN** after `useGraphStore.getState().upsertEntity(type, id, data)` returns, `notifyDevtools({ kind: "upsert", type, id, data, at: <now> })` MUST be called exactly once before the function returns its transaction.

#### Scenario: patchEntity instrumentation
- **WHEN** `graph-actions.ts` `patchEntity(type, id, patch)` is invoked
- **THEN** after the store mutation, `notifyDevtools({ kind: "patch", type, id, patch, at: <now> })` MUST be called exactly once.

#### Scenario: clearPatch instrumentation
- **WHEN** `graph-actions.ts` `clearPatch(type, id)` is invoked
- **THEN** after the store mutation, `notifyDevtools({ kind: "clearPatch", type, id, at: <now> })` MUST be called exactly once.

#### Scenario: Notification order matches op order
- **WHEN** a caller invokes the three ops sequentially (upsert → patch → clearPatch)
- **THEN** subscribers MUST receive exactly three events in that order, each with the documented payload, and each only after the corresponding store mutation has completed.

### Requirement: Hot-Path No-Op
`notifyDevtools` SHALL incur zero observable cost when no subscribers are registered.

#### Scenario: Zero subscribers — no event work
- **WHEN** `notifyDevtools(event)` is called with the subscriber set empty
- **THEN** the function MUST return immediately without iterating an empty collection, without allocating the event timestamp string, and without producing any side effect.

#### Scenario: Implementation early-return
- **WHEN** the implementation is inspected
- **THEN** it MUST contain an explicit early-return guard on `subscribers.size === 0` (or equivalent), placed at the top of the function before any other work.

### Requirement: Production Tree-Shake Gate
Op-site `notifyDevtools` calls SHALL be wrapped in a build-time tree-shakeable guard so production bundles can elide them.

#### Scenario: NODE_ENV guard on every call site
- **WHEN** a `notifyDevtools` call appears in any non-test file under `src/`
- **THEN** the call MUST be wrapped in a `if (process.env.NODE_ENV !== "production")` block (or equivalent build-time gate consistent with `tsup` config), so a minifier with dead-code elimination removes the block in prod builds.

#### Scenario: Symbol remains exported
- **WHEN** the production bundle is inspected after build
- **THEN** the `subscribeDevtoolsEvent` export MUST still exist as a symbol (it is part of the public API); only the *internal* `notifyDevtools` call-sites under the NODE_ENV guard MAY be elided.

#### Scenario: Change 9 verifies elision
- **WHEN** the W7 tree-shake gate (`seim-em-explorer-production-treeshake-check`) runs against a production build
- **THEN** the gate MUST be able to assert zero string matches for `kind: "upsert"` / `kind: "patch"` / `kind: "clearPatch"` payload literals in the prod bundle output (because every notify call carrying those literals is under a NODE_ENV guard).

### Requirement: Subscriber Lifecycle
Subscribers SHALL be tracked in a `Set` with stable add/remove semantics.

#### Scenario: Subscribe is idempotent for distinct callbacks
- **WHEN** two distinct functions are passed to `subscribeDevtoolsEvent`
- **THEN** both are tracked; each subsequent event invokes both. Returning multiple `UnsubscribeFn`s — one per call — is the contract.

#### Scenario: Subscribe of the same function twice
- **WHEN** the SAME function reference is passed to `subscribeDevtoolsEvent` twice
- **THEN** the second registration MUST be a no-op (Set semantics dedupe); subsequent events MUST invoke the function exactly once per event. Each returned `UnsubscribeFn` removes the same registration.

#### Scenario: Unsubscribe of the same UnsubscribeFn twice
- **WHEN** an `UnsubscribeFn` is invoked twice
- **THEN** the second invocation MUST be a no-op; no error MUST be thrown.

### Requirement: Re-Entrancy Safety
A subscriber callback that itself triggers graph-actions ops SHALL NOT cause infinite recursion or unbounded recursion.

#### Scenario: Subscriber calls upsertEntity inside its callback
- **WHEN** a subscriber receives a `kind: "upsert"` event and calls `graph-actions.upsertEntity(...)` inside its callback
- **THEN** the recursive op MAY trigger another `notifyDevtools` (the spec does not prohibit re-entry), BUT the implementation MUST ensure that re-entry while iterating the subscriber set does not mutate the iteration in flight. A snapshot-of-subscribers iteration pattern (or equivalent) is REQUIRED.

#### Scenario: Subscriber throws
- **WHEN** a subscriber callback throws
- **THEN** the engine MUST NOT abort delivery to other subscribers; the throw is caught at the `notifyDevtools` level, logged via `console.warn`, and other subscribers receive the event.

### Requirement: Test Coverage
A vitest file SHALL exercise every requirement above using fixture-driven tests.

#### Scenario: Tests file exists
- **WHEN** the test surface is inspected
- **THEN** a vitest file at `src/engine-devtools-tap.test.ts` (or appended to `src/engine.test.ts`) MUST exist with one or more `describe` blocks per requirement above, and a total of at least 12 assertions across all `it` cases.

#### Scenario: Tests pass on the W2 worktree
- **WHEN** `pnpm test` runs in `~/.claude/worktrees/seim-entity-management`
- **THEN** all newly-introduced devtools-tap tests MUST pass; total suite count MUST grow by at least the count of new tests, with no regressions in the pre-existing 104-test suite.
