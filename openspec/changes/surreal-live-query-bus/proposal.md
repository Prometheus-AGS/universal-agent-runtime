## Why

The Universal Agent Runtime must guarantee that no view in the SPA shows stale data. Providers, LLM models, agents, skills, KBs, and settings are referenced across many UI surfaces (toolbars, dropdowns, header chips, list pages); changing one in any view must propagate to all subscribers immediately. We need a database-level realtime change stream — SurrealDB's `.select().live()` API returns `Action::{Create,Update,Delete}` notifications on a stream — wrapped into a process-internal bus and re-exposed to the SPA over SSE.

## What Changes

- New module `src/uar/realtime/` with:
  - `LiveQueryBus` trait: `subscribe(topic: EntityTopic) -> broadcast::Receiver<LiveEvent>`.
  - `SurrealLiveBus` impl that opens one `.select().live()` stream per topic at startup and forwards into a `tokio::sync::broadcast` channel keyed by topic.
  - `LiveEvent { action: Create|Update|Delete, id: RecordId, data: serde_json::Value }`.
- Topics enrolled at startup: `knowledge_bases`, `knowledge_documents`, `agents`, `providers`, `models`, `skills`, `settings`.
- Supervised reconnect with exponential backoff on stream drops; expose `live_bus_streams_up{topic}` gauge.
- Public SSE endpoint `GET /api/live/{topic}` (JWT-auth-gated) — each connected client gets a fan-out subscription; events emitted as `event: <action>\ndata: {json}\n\n`.
- Optional `POST /api/live/_replay/{topic}?since=<ts>` is **out of scope** for this change (covered by reflection backlog).
- Postgres-backend equivalent left as a TODO with a clear `pg_notify`/`LISTEN` plan; not implemented here because the postgres backend is currently feature-gated off in the production build.

## Acceptance

- Writing a row directly into Surreal triggers the corresponding SSE event in ≤200ms p95.
- Killing the upstream Surreal connection and bringing it back results in the bus reconnecting and resuming streams without restarting UAR.
- Smoke: `curl -N http://127.0.0.1:8088/api/live/knowledge_documents` shows live events while an unrelated client uploads a document.
