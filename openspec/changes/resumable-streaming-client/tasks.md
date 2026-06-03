# Tasks — resumable-streaming-client

## 0. Bootstrap

- [x] 0.1 Confirmed server resume endpoint + `x-uar-run-id` header exist (routes.rs:63-94; HP1 header) — no backend change
- [x] 0.2 `bun run typecheck` / `eslint` baseline on touched files

## 1. Capture SSE id + run id cursor

- [x] 1.1 `parseSseBlock` parses `id:` and returns it
- [x] 1.2 `lastEventId` tracked (monotonic) inside the stream generator
- [x] 1.3 `serverRunIdRef` (from `x-uar-run-id`, HP1) confirmed set after the POST response and passed into the generator

## 2. Refactor dispatch for reuse (pure) — done via async generator (see Notes)

- [x] 2.1 Instead of extracting a `handleBlock` closure, the per-block dispatch stays inline and is driven by `for await (const { event, data } of streamSseBlocks(...))` — the giant `switch` is untouched
- [x] 2.2 `streamSseBlocks(initialReader, opts)` async generator owns chunk-read + `\n\n` split + `parseSseBlock` + cursor + reconnect (subsumes `readSseStream`)
- [x] 2.3 The initial POST stream is consumed through the generator — dispatch behavior unchanged

## 3. Resume service (chat-stream-api.ts)

- [x] 3.1 `resumeRunStream(runId, lastEventId, signal): Promise<Response>` → `GET /api/uar/runs/{runId}/stream?last_event_id=` + `Last-Event-ID` header (store-only caller)

## 4. Reconnect on mid-stream drop (chat-stream-store.ts)

- [x] 4.1 On a read error after ≥1 yielded block WITH `runId`: the generator reconnects via `resumeRunStream` and continues yielding (seamless to the dispatch loop)
- [x] 4.2 Bounded by `STREAM_MAX_RECONNECTS` (5); on exhaustion the error propagates and the existing terminal handling finalizes
- [x] 4.3 No re-POST after first chunk: reconnect happens in the generator; the existing `stream_already_started` gate still correctly prevents a duplicate POST after reconnects exhaust
- [x] 4.4 Cursor guard: generator skips blocks with id ≤ highest seen (dedup across the reconnect boundary; server also filters)

## 5. Validation (gate)

- [x] 5.1 `bun run typecheck` — no errors in touched files (28 pre-existing errors in unrelated files unchanged)
- [x] 5.2 `eslint --fix` clean (exit 0) on `chat-stream-store.ts` + `chat-stream-api.ts`
- [ ] 5.3 Manual: normal stream unaffected; simulated mid-stream drop resumes remaining tokens, no duplicate message/run — PENDING live env (not runnable headlessly)
- [x] 5.4 `openspec validate resumable-streaming-client --strict`; `.kbd-orchestrator` progress updated in wrap-up

## Notes

- **Design refinement (D2/D3):** implemented as an **async generator** `streamSseBlocks` that yields `{event,data}` and owns read+parse+reconnect, rather than extracting a `handleBlock` closure + `readSseStream`. This leaves the ~320-line dispatch `switch` completely untouched (lowest-risk) and isolates all reconnect complexity in one place. Satisfies every `resumable-streaming` spec requirement.
- Abort (user stop) propagates immediately from the generator; never reconnects; `cancelRun` still fires server-side (HP1).
- Frontend-only; server resume path already complete. Kept `fetch`/`ReadableStream` for POST (headers); resume is a GET.
- Out of scope: `EventSource` switch; cross-page-reload resume (in-memory cursor); visual "reconnecting" indicator (would trigger UI/UX routing).
- This change is store/service logic (data layer), not visual UI/UX — frontend layering rules applied (new GET is a service, store orchestrates); visual-design routing did not apply.
