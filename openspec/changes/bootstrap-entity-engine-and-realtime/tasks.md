## 1. Edit main.tsx

- [x] 1.1 Already wired in prior phase — `bootstrapEntityGraph()` invoked at module init in `frontend/src/main.tsx`.
- [x] 1.2 `configureEngine(...)` called inside `frontend/src/entities/bootstrap.ts`.
- [x] 1.3 Realtime transport selection runs via `initSyncTransport`.

## 2. Realtime transport — surreal-remote branch

- [x] 2.1 Replaced the previous direct-WebSocket-to-Surreal path (would have bypassed JWT auth) with `createAllUarAdapters()` from `@/lib/realtime/topics`.
- [x] 2.2 Registered every adapter against the shared `RealtimeManager`.
- [x] 2.3 Removed unused `createWebSocketAdapter` import.

## 3. Verification

- [x] 3.1 `pnpm --filter ./frontend build` succeeds — new bundle hash `index-DZ9fJ5jd.js` + dedicated `topics-KtNwMZYU.js` chunk.
- [x] 3.2 UAR restarted, healthy.
- [ ] 3.3 Open SPA in browser, confirm 7+ EventSource connections — manual step.
