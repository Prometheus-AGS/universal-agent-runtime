## 1. Bus core

- [x] 1.1 Create `src/uar/realtime/mod.rs` with `LiveEvent`, `EntityTopic` enum, `LiveAction`.
- [x] 1.2 Create `src/uar/realtime/surreal_bus.rs` with `SurrealLiveBus`; constructor accepts an existing `Surreal<Any>` client.
- [x] 1.3 For each topic, spawn a tokio task that calls `db.select(table).live().await?` and forwards into a broadcast channel.
- [x] 1.4 Supervised reconnect with exponential backoff (start 250 ms, cap 30 s); "table does not exist" recognized and parked at max backoff.

## 2. SSE endpoint

- [x] 2.1 `GET /api/live/{topic}` in `src/uar/api/live.rs` using axum `Sse`.
- [x] 2.2 Topic string parser; 404 unknown.
- [x] 2.3 JWT-auth-gated via existing middleware on `/api/*`.
- [x] 2.4 Emits `event: create|update|delete\ndata: {json}\n\n`; 15 s keep-alive.

## 3. Observability

- [x] 3.1 tracing info on connect; warn on reconnect (except for benign "table does not exist").
- [ ] 3.2 Prometheus gauge/counter — deferred to `integration-tests-and-docs`.

## 4. Tests

- [x] 4.1 Manual: live curl + upload smoke test passed — `event: create` delivered immediately on document upload.
- [ ] 4.2 Drop/reconnect test — deferred to integration-tests change.

## 5. Docs

- [ ] 5.1 `docs/realtime.md` — deferred to integration-tests-and-docs change.
