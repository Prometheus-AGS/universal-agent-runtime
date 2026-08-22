PLAN: fix-embedded-sse-offline-reconnect
Project: universal-agent-runtime
Date: 2026-08-20
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-embedded-sse-offline-reconnect: Align embedded SSE delivery and prove deterministic no-reload recovery.
   - Scope: frontend transport | focused unit test | one BDD acceptance path | OpenSpec/evidence
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Keep the existing Rust endpoint and shared `/api/live` transport unchanged. Correct the embedded adapter's named-event/payload mapping, give that adapter one explicit error-to-reconnect state machine with bounded backoff and complete cleanup, and bind tests to the EventSource actually registered by `initSyncTransport`.

EXECUTION ROUND ORDER
Round 1 (serial): `fix-embedded-sse-offline-reconnect`

IMPLEMENTATION ORDER INSIDE THE CHANGE
1. Specify the `embedded-sse-sync` contract before code:
   - `entity.change` is the only data event consumed; `connected` and
     `heartbeat` do not mutate the graph.
   - `{table, action, id, record}` maps through canonical table-to-entity names
     into one `EntityChange`.
   - A detected error closes the old EventSource before scheduling one bounded
     retry, successful open resets backoff, and unsubscribe cancels the source
     and any pending timer.
   - Delivery after reconnect is exactly once per received event; this adapter
     does not add replay/checkpoint semantics the server does not provide.
2. Implement the minimum product change in `frontend/src/entities/sync.ts`:
   - Export the embedded adapter factory for direct testing instead of moving
     unrelated transport initialization.
   - Listen with `addEventListener("entity.change", ...)`, validate the payload,
     map canonical entity names, and surface connection status through the
     existing `RealtimeAdapter` contract.
   - Own reconnection explicitly after `onerror`; close before retry so native
     and application retries cannot create parallel streams.
3. Add `frontend/src/entities/sync.test.ts` with a hand-rolled FakeEventSource:
   - Named-event and payload mapping positive case.
   - Unnamed/wrong-name and malformed-payload rejection controls.
   - Open → error → timed replacement → open status sequence.
   - One post-reconnect event produces one change, not two.
   - Unsubscribe closes the source and prevents a pending reconnect.
4. Correct only the embedded scenario in
   `tests/bdd/features/local-first-resilience.feature` and
   `tests/bdd/steps/local-first-resilience.steps.ts`:
   - Install an EventSource observer before application bootstrap and capture
     the application's embedded source; do not create a separate probe source.
   - Deliver one named entity event through that source and assert a visible
     graph-backed Knowledge state.
   - Force the adapter's error path, observe a second real
     `/api/uar/sync/stream` request/open, deliver an update for the same entity,
     and assert the visible state updates once without reconnect-time reload or
     `replayRuntime`.
5. Repair the observed projection defect in the source package rather than
   patching the UAR Knowledge screen:
   - Work in an isolated worktree of
     `/Users/gqadonis/Projects/prometheus/prometheus-entity-management`.
   - Make `useEntityView` and its replacement `useEntityQuery` subscribe to
     entity snapshots as well as list IDs, with focused React regression tests
     proving that an existing entity update changes rendered items without an
     ID change.
   - Run the upstream package's focused test, TypeScript, lint, build, and full
     package test gates. Only after they pass, increment the publishable React
     package version and re-run its publish-facing gates.
   - Commit, push, and open the upstream source/compatibility PR, and open the
     canonical Changesets rc.2 PR after its generated output passes the same
     gates. Advance UAR's submodule pin to the tested source/compatibility
     commit; do not pretend the two unmerged PR heads are one commit. Do not
     patch UAR's Knowledge model or UI around a source-package reactivity
     defect.
6. Make BDD preparation build the source package through its dependency graph:
   - Replace the direct React-package `tsup` invocation with the upstream Turbo
     build filtered to the React package and its dependencies.
   - This is required by the observed clean-build declaration failure for
     `getGraphSyncStatus`; do not rely on a stale prebuilt core `dist`.
   - Keep `pnpm-lock.yaml` and `frontend/package.json` excluded.
7. Verify only after implementation:
   - Preserve the existing two-attempt Playwright timeout as the pre-fix
     negative observation; do not rerun broad tests before code exists.
   - After each edit run Tier 0: `pnpm typecheck` and `pnpm lint`.
   - When the unit is complete run focused Tier 1:
     `pnpm -C frontend test src/entities/sync.test.ts`.
   - At child phase completion run Tier 2: `pnpm build` and `pnpm test`.
   - Run only the required live browser acceptance scenario, with fresh
     processes: generate its BDD spec through the existing preparation path,
     then `CI=1 pnpm exec playwright test -c tests/bdd/playwright.config.ts tests/bdd/.features-gen/features/local-first-resilience.feature.spec.js --grep 'embedded SSE connection'`.
   - Run `openspec validate fix-embedded-sse-offline-reconnect --strict`, the
     artifact-refiner validation gate, JSON/schema checks, and `git diff --check`.
8. Record and hand off:
   - Write child `verification.md` with actual commands/output and separate
     positive/negative rows; do not claim checkpoint replay or lossless
     delivery across the disconnected interval.
   - Append the named-event/probe lesson to `.prometheus`; complete independent
     critic/judge review; commit only the child change; reflect and return to the
     parent.

EXPLICIT CUTS AND STOP CONDITIONS
- The root manifest may change only the BDD preparation build order proven
  necessary above. No UAR dependency declaration, frontend manifest, or
  lockfile change. The approved canonical source-package version PR and UAR
  submodule pointer advance are the only dependency-version surfaces.
- No `src/server.rs` or shared `/api/live` adapter change. Stop and revise the
  plan if the existing endpoint cannot support the specified proof.
- No generic realtime refactor, checkpoint replay, offline write queue, UI
  redesign, or broad screen-suite rerun.
- The observed visible-Knowledge failure is resolved in the upstream graph
  projection package. Stop if that repair requires a UAR UI/store workaround,
  store injection, manual replay, or unrelated upstream refactor.
- Stop if reconnect produces parallel requests or duplicate delivery rather
  than hiding it with a count reset.

TRADE-OFF
- Owning reconnect in the adapter adds a small state machine instead of relying
  entirely on browser-native EventSource recovery. That cost is accepted
  because it exposes status, cleanup, and a deterministic error boundary for
  verification. This child deliberately does not promise replay of changes
  emitted while disconnected; adding checkpoint recovery would require a
  different server contract and a revised phase.

COMMANDS TO RUN
/opsx:new fix-embedded-sse-offline-reconnect

PLAN COMPLETE
