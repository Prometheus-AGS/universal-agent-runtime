## Context

Server resume is complete: `GET /api/uar/runs/{id}/stream?last_event_id=` reads the `Last-Event-ID` header, replays from the 512-event ring buffer, and merges live events filtered by id (`routes.rs:63-94`, `sse.rs` sets `.id(event.id)` on each SSE event). Client gaps (all in `frontend/src/stores/chat-stream-store.ts`):
- `parseSseBlock` (line 220) only reads `event:`/`data:`, dropping `id:`.
- The read loop has no `lastEventId` cursor.
- `shouldRetryStreamAttempt` (line 469) returns `stream_already_started` (no retry) once `sawFirstStreamChunk` is set — a mid-stream drop is terminal.
- HP1 already added `serverRunIdRef` (from the `x-uar-run-id` header) and `cancelRun`.

The event-dispatch logic is a large `switch (event)` (~lines 656-860) over `agui.*` events, currently inline in the POST read loop.

## Goals / Non-Goals

**Goals:** capture the SSE id; resume a dropped stream via the server endpoint without duplicating the run or re-applying events; keep clean/pre-first-chunk paths unchanged; reuse one dispatch path for POST and resume.

**Non-Goals:** switching to `EventSource` (we keep `fetch` for POST + headers); any server change; a visual redesign (an optional tiny "reconnecting" status is allowed, not required); resuming across a full page reload (in-memory cursor only — persistence is a follow-up).

## Decisions

### D1 — `parseSseBlock` returns the id; the loop tracks `lastEventId`
Add `id?: string` (or number) to the parse result; in the read loop, after handling a block, set `lastEventId = max(lastEventId, parsedId)`. The cursor is per-`startStream` invocation.
- **Why max, not last:** events arrive in id order, but `max` is robust to any reordering and to replay overlap.

### D2 — Extract the per-block dispatch into one closure
Refactor the inline `switch (event)` into a closure `handleBlock(block) -> { stop: boolean }` defined in `startStream`'s scope (capturing `store`, `threadId`, `runId`, `pendingArgs`, callbacks, the emit flags). Both the initial POST read loop and the resume read loop call it. This avoids duplicating ~150 lines and guarantees identical handling on resume.
- **Why a closure over a free function:** it already needs ~8 captured locals; a closure keeps the call sites tiny without threading a big context struct.

### D3 — A `readSseStream(reader)` helper that drives `handleBlock` and updates the cursor
Extract the chunk-read/`buffer.split("\n\n")`/`parseSseBlock` loop into `readSseStream(reader, onBlock)` returning when the stream ends or `onBlock` signals stop. Used by both POST and resume.

### D4 — Resume loop on mid-stream drop
Restructure the outer flow:
1. Initial POST attempt(s) — existing retry logic for failures **before** the first chunk (unchanged).
2. Once `sawFirstStreamChunk` and `serverRunIdRef` are set, a transport error breaks into a **reconnect loop**: `resumeRunStream(runId, lastEventId, signal)` (a GET service) → `readSseStream` → on terminal, done; on transport error, retry up to a reconnect cap within the retry budget; on exhaustion, finalize via existing terminal handling.
- **Why gate resume on `serverRunIdRef` + `lastEventId`:** without both we can't target the resume endpoint or set the cursor, so fall back to the existing behavior.

### D5 — `resumeRunStream` service (layering)
Add `resumeRunStream(runId, lastEventId, signal) -> Promise<Response>` to `chat-stream-api.ts` doing `GET /api/uar/runs/{runId}/stream` with header `Last-Event-ID: <id>` and the abort signal. Only the store calls it (layering).

### D6 — Dedup is server-authoritative; client cursor prevents re-apply
The server already filters live events by id and replays from `last_event_id`. The client advances `lastEventId` monotonically and ignores blocks with id ≤ cursor as a belt-and-suspenders guard, so terminal/text events aren't double-applied across the reconnect boundary.

## Risks / Trade-offs

- **[Large refactor of a hot path]** extracting the dispatch closure touches the core streaming loop → Mitigation: pure extraction (same logic, same order); rely on type-check + lint; keep the diff reviewable; manual smoke before merge.
- **[Gap on long drops]** if the cursor predates the 512-event window, some events are unrecoverable → Mitigation: server replays from oldest available; documented; acceptable for typical short drops.
- **[Double-apply across boundary]** replay overlap could re-render text → Mitigation: D6 client cursor guard + server id filter.
- **[Reconnect storm]** a flapping connection could loop → Mitigation: D4 reconnect cap + existing retry budget; clean finalize on exhaustion.
- **[run_id missing]** older server or proxy stripping the header → Mitigation: gate resume on `serverRunIdRef`; fall back to current terminate-on-drop behavior.

## Migration Plan

1. `parseSseBlock` returns id; add `lastEventId` cursor (no behavior change yet).
2. Extract `handleBlock` closure + `readSseStream` helper from the existing loop; route the POST stream through them (pure refactor; verify unchanged behavior).
3. Add `resumeRunStream` service.
4. Add the reconnect loop on mid-stream drop; relax the `sawFirstStreamChunk` gate to allow resume (not re-POST).
5. Type-check + lint touched files; manual smoke (normal stream, simulated mid-stream drop).
- **Rollback:** frontend-only; revert restores terminate-on-drop. No data migration.

## Open Questions

- **Reconnect cap value:** reuse `maxAttempts`/retry budget, or a separate `maxReconnects`? (Lean: reuse the existing budget + a small fixed cap, e.g. 5.)
- **Reconnect UX:** surface a subtle "reconnecting…" indicator, or stay silent? (Lean: silent for HP5; a status chip is a small follow-up — would trigger the UI/UX routing.)
- **Cross-reload resume:** persist `{runId, lastEventId}` to survive a page reload? (Defer.)
