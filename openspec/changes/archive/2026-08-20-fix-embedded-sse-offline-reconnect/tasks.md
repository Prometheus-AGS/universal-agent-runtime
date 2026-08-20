## 1. Workflow and Contract

- [x] 1.1 Register and start this change in the active KBD child, preserve the existing two-attempt Playwright timeout as the pre-fix negative observation, and verify the canonical waypoint selects this change before product edits
- [x] 1.2 Validate the completed proposal, design, and `embedded-sse-sync` delta with `openspec validate fix-embedded-sse-offline-reconnect --strict`

## 2. Embedded Adapter

- [x] 2.1 Export the embedded adapter factory, consume and validate named `entity.change` payloads, map canonical entity types, expose status, and verify `pnpm typecheck` plus `pnpm lint` pass
- [x] 2.2 Add capped error recovery that closes the failed source before one retry and cancels source/timer state on unsubscribe, then verify `pnpm typecheck` plus `pnpm lint` still pass

## 3. Focused Regression Coverage

- [x] 3.1 Add FakeEventSource unit controls for named-event mapping, transport/malformed rejection, status transitions, one replacement connection, exactly-once post-reconnect delivery, and unsubscribe cancellation; verify `pnpm -C frontend test src/entities/sync.test.ts` passes
- [x] 3.2 Replace the separate-probe/manual-replay browser scenario and steps with instrumentation of the registered embedded source and a visible Knowledge transition before and after a forced error; verify `pnpm typecheck` plus `pnpm lint` pass
- [x] 3.3 Make BDD preparation build the source React package and its workspace dependencies after observing the direct React-only declaration build fail on `getGraphSyncStatus`

## 4. Completion Evidence

- [x] 4.1 At child completion, run and record `pnpm build`, `pnpm test`, and the fresh-process embedded SSE browser scenario; verify the browser observes a second real stream request and one visible post-reconnect update without reload or manual replay
- [x] 4.2 Write `verification.md` with actual positive and negative command/output rows, append the transport-proof lesson to `.prometheus`, and verify strict OpenSpec, artifact-refiner, JSON/schema, scope, and `git diff --check` gates pass
- [x] 4.3 Obtain independent artifact critic and judge approval, correct every blocker, commit only the child implementation/evidence while excluding parent certification artifacts and unrelated user changes, and hand control to OpenSpec archive plus KBD reflection
