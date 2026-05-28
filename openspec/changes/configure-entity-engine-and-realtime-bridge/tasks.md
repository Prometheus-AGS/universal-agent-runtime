## 1. Engine bootstrap

- [x] 1.1 `frontend/src/lib/entity-engine.ts` calls `configureEngine` with the locked defaults.
- [ ] 1.2 Side-effect import wired in `main.tsx` — **deferred** to the first migration that actually consumes a hook (change 10).

## 2. Realtime adapter

- [x] 2.1 `frontend/src/lib/realtime/uar-sse-adapter.ts` implements `RealtimeAdapter` over `EventSource` against `/api/live/{topic}`.
- [x] 2.2 SSE `create|update|delete` event names map to `EntityChange.op` (`insert | update | delete`).
- [x] 2.3 `frontend/src/lib/realtime/topics.ts` enumerates the 7 topics + `createAllUarAdapters()` helper.
- [x] 2.4 Exponential-backoff reconnect with jitter cap at 30 s.

## 3. Fetchers

- [ ] 3.1 Per-entity `services/entities/*.ts` modules — deferred to migration changes.

## 4. Topic registration

- [ ] 4.1 RealtimeManager wiring on the host application — deferred to migration changes.

## 5. Tests

- [ ] 5.1 Mock EventSource → upsertEntity smoke — deferred.

## 6. Docs

- [ ] 6.1 `docs/frontend-realtime.md` — deferred to integration-tests-and-docs change.

## 7. Build verification

- [x] 7.1 `pnpm --filter ./frontend build` passes with the new modules present (tree-shaken because nothing imports them yet).
