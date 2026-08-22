ASSESSMENT: fix-embedded-sse-offline-reconnect
Project: universal-agent-runtime
Date: 2026-08-20
Codebase baseline: Commit `7736c797b36c2972727228208a186d075fdaffd2` contains an embedded SurrealDB SSE endpoint and client adapter, but the endpoint and adapter do not share an event-name or payload contract, and the strengthened no-reload reconnect scenario fails before observing a second stream request.
Cross-tool progress: the child KBD scaffold exists at runtime revision 170 with zero registered changes and zero implementation tasks; the parent `screen-by-screen-validation` change remains in progress at 70/79 with its local-first task truthfully unchecked.

IMPLEMENTATION STATUS
- Embedded SSE endpoint: PARTIAL — `src/server.rs` serves `/api/uar/sync/stream` for embedded persistence, sends an initial named `connected` event, polls every five seconds, emits named `entity.change` events with `{table, action, id, record, ts}`, and emits named `heartbeat` events.
- Embedded SSE event ingestion: MISSING — the private adapter in `frontend/src/entities/sync.ts` installs only `EventSource.onmessage` and expects `{entity_type, action, id, data}`. Current EventSource documentation states that `onmessage` receives unnamed or explicitly `message` events, while other named events require `addEventListener(<name>, ...)`. The server's named `entity.change` event therefore does not reach this handler, and its `{table, record}` payload would not satisfy the handler even if delivered.
- Embedded reconnect/status behavior: PARTIAL — the private adapter has no `onopen`, `onerror`, `onStatusChange`, explicit reconnect scheduling, or online-event recovery. Native EventSource supports reconnection after a detected connection loss, but the observed browser run did not establish a second request within 15 seconds after Playwright toggled offline and online. That timeout does not by itself distinguish an application defect from a test that failed to force the idle stream into an error state.
- Existing robust SSE precedent: DONE OUTSIDE THIS PATH — `frontend/src/lib/realtime/uar-sse-adapter.ts` implements explicit error handling, connection status, cleanup, and bounded exponential backoff for the multiplexed `/api/live` endpoint. It cannot be substituted unchanged because the embedded endpoint and payload are different.
- Reconnect acceptance coverage: PARTIAL AND FAILING — `tests/bdd/steps/local-first-resilience.steps.ts` opens a separate probe EventSource, toggles browser connectivity, and waits for the probe's second open. It does not observe the adapter registered by `initSyncTransport`, does not deliver a real backend `entity.change`, and checks duplication using separately replayed synthetic cockpit data. The stronger no-reload form correctly removed the prior reload/manual-replay false positive, but it currently times out.
- Unit coverage: MISSING — the shared `/api/live` adapter has FakeEventSource contract tests; the private embedded adapter has no event mapping, reconnect, status, or cleanup tests.

CROSS-TOOL PROGRESS
- KBD: child created and activated; Assess is the current stage target. No child goal, plan, change, or task has been registered.
- OpenSpec: no `fix-embedded-sse-offline-reconnect` change exists yet. The parent `screen-by-screen-validation` task 2.3 and certification task 3.1 remain unchecked after the observed failure.
- Parent evidence: the existing `screen-by-screen-validation` verification and matrix still claim SSE reconnect restoration. Those claims are superseded by the strengthened failing negative observation and cannot be retained as passing evidence until the child returns a real transport proof.
- Child scaffold: `goals.md` and `handoff-in.md` still contain generated placeholders. Plan must replace them with the observed defect, bounded scope, success criteria, and return condition before Execute.

SPEC GAP SUMMARY
- `openspec/changes/screen-by-screen-validation/specs/product-validation-evidence/spec.md` requires a supported screen's primary function to succeed end-to-end and requires an observed supported-product defect to be repaired before it is reported as passing.
- The parent design requires live SSE assertions to change a visible surface without reload. The current scenario instead observes a separate transport probe and synthetic runtime replay data, so it does not bind a visible state change to the embedded adapter.
- No active requirement defines the embedded endpoint's event name, payload shape, reconnect trigger, cleanup behavior, or duplication guarantee. The child needs a narrow OpenSpec delta for that contract rather than broad changes to the shared realtime fabric.
- A reconnect-only repair is insufficient because the confirmed named-event/payload mismatch prevents normal embedded change delivery even while connected.
- An event-mapping-only repair is insufficient because the observed offline/online scenario still lacks a deterministic disconnect trigger and a fail-closed proof that the registered adapter reconnects without duplicate delivery.

OBSERVED FAILURE EVIDENCE
- Command:
  `CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts tests/bdd/.features-gen/features/product-screen-validation.feature.spec.js tests/bdd/.features-gen/features/cross-screen-security.feature.spec.js tests/bdd/.features-gen/features/local-first-resilience.feature.spec.js --grep 'Providers changes|Default and orchestrator|embedded SSE connection'`
- Relevant output: `Local-first browser resilience > The embedded SSE connection reconnects without duplicating runtime state` failed on the initial run and retry with `TimeoutError: page.waitForRequest: Timeout 15000ms exceeded while waiting for event "request"` at `tests/bdd/steps/local-first-resilience.steps.ts:104-109`.
- Retained artifact: `tests/bdd/test-results/features-local-first-resil-d7113-t-duplicating-runtime-state/error-context.md` records the exact timeout, step lines, and page state; the retry records the same failure.
- Documentation check: Context7 `/mdn/content` confirms that named SSE events require `addEventListener(eventName, ...)`; `onmessage` does not receive other event types. It also documents the stream `retry` field as the reconnection delay after connection loss.

BUILD HEALTH
- TypeScript Tier 0: PASS before child creation — `pnpm typecheck && pnpm lint` exited 0 after the parent test edits that exposed this defect.
- Focused browser acceptance: FAIL — the embedded reconnect scenario timed out on both its initial attempt and retry before a second `/api/uar/sync/stream` request was observed.
- Rust build: UNKNOWN — no Rust source changed and no Rust build was run during this assessment.
- Test coverage: PARTIAL — the robust shared adapter has focused unit coverage; the embedded adapter has none, and the browser test does not exercise its event delivery contract.
- Known violations: endpoint/adapter event-name mismatch; endpoint/adapter payload mismatch; no deterministic registered-adapter reconnect proof; existing parent pass claim is stale.

CONSTRAINT CHECK
- AGENTS.md violations: NONE introduced by this assessment. The child exists before product execution, only observed behavior and inspected contracts are reported, and no product code was changed.
- Parent stop condition: SATISFIED — execution stopped at the fourth supported-product defect and created this narrowly scoped child rather than extending the parent repair authorization.
- Child scope: COMPLIANT for Assess — only child KBD artifacts are written. `scope.json` currently permits no product or OpenSpec files, so Plan must deliberately name the minimum write surface before Execute.
- Capability inversion: unaffected — the likely repair is in the trusted frontend transport and test boundary, not an agent kernel.
- Dependency discipline: no new dependency is indicated. The repository already has EventSource, realtime-manager, fake-timer, and Playwright test surfaces needed for a bounded repair.

GOAL PROGRESS
- Make embedded SSE change delivery use one server/client event contract: NOT MET — named events and payload fields disagree.
- Prove that the registered embedded adapter restores delivery after a deterministic disconnect without reload: NOT MET — the strengthened scenario times out and currently watches a separate probe.
- Prove reconnect does not duplicate a delivered entity change or leave a reconnect active after unsubscribe: NOT MET — no adapter-level negative controls exist.
- Return truthful passing local-first evidence to `screen-by-screen-validation`: NOT MET — parent tasks 2.3 and 3.1 remain unchecked.

MINIMUM REPAIR BOUNDARY FOR PLAN
- Extract or export the embedded adapter from `frontend/src/entities/sync.ts` into a focused, testable frontend realtime module without changing the shared `/api/live` adapter.
- Define one embedded event contract matching `/api/uar/sync/stream`: listen for `entity.change`, map `table` and `record` into `EntityChange`, ignore connected/heartbeat events, surface status, and close all listeners/timers on unsubscribe.
- Choose and specify a deterministic reconnect trigger at the adapter boundary. The plan must not rely solely on a short browser offline toggle; its negative control must fail when reconnect handling is disabled.
- Add FakeEventSource unit tests for named-event mapping, malformed-event rejection, connection/status transitions, disconnect/reconnect, single delivery after reconnect, and unsubscribe cancellation.
- Replace the separate browser probe/synthetic replay assertion with a visible state change delivered through the registered embedded adapter, then force the documented disconnect path and prove another real change arrives without reload or duplication.
- Create one narrow OpenSpec change and widen child `scope.json` only to the selected frontend transport, focused tests, OpenSpec artifacts, KBD artifacts, and append-only history. Do not change backend Rust unless planning proves a deterministic test seam cannot be provided at the existing endpoint boundary.
- Return condition: focused unit negative/positive controls, TypeScript Tier 0, the single local-first browser scenario, strict OpenSpec validation, and independent artifact review pass; then resume `screen-by-screen-validation` without rerunning phase-level certification early.

SYCOPHANCY REVIEW
- Detect-only review score: 0.01785714365541935; correction not mandatory. The sole low-severity finding was document length. The uncomfortable finding retained is that the latest timeout is real but does not isolate native transport recovery, while the separate endpoint/adapter contract mismatch is independently confirmed and broader than the scenario originally claimed to test.

ASSESSMENT COMPLETE
