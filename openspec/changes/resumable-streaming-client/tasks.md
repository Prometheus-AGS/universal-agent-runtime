# Tasks — resumable-streaming-client

## 0. Bootstrap

- [ ] 0.1 Confirm server resume endpoint + `x-uar-run-id` header exist (routes.rs:63-94; HP1 header) — no backend change
- [ ] 0.2 `bun run typecheck` / `eslint` baseline on touched files

## 1. Capture SSE id + run id cursor

- [ ] 1.1 `parseSseBlock` also parses `id:` and returns it
- [ ] 1.2 Track `lastEventId` (monotonic max) per `startStream` invocation
- [ ] 1.3 Confirm `serverRunIdRef` (from `x-uar-run-id`, added in HP1) is set after the POST response

## 2. Refactor dispatch for reuse (pure)

- [ ] 2.1 Extract the per-SSE-block `switch (event)` into a `handleBlock(block) -> { stop }` closure in `startStream` scope (captures store/threadId/runId/pendingArgs/callbacks/emit flags)
- [ ] 2.2 Extract `readSseStream(reader, onBlock)` (chunk read + `\n\n` split + `parseSseBlock` + cursor update)
- [ ] 2.3 Route the initial POST stream through `readSseStream`/`handleBlock` — verify identical behavior (pure refactor)

## 3. Resume service (chat-stream-api.ts)

- [ ] 3.1 `resumeRunStream(runId: string, lastEventId: number, signal: AbortSignal): Promise<Response>` → `GET /api/uar/runs/{runId}/stream` with `Last-Event-ID` header (store-only caller, layering)

## 4. Reconnect on mid-stream drop (chat-stream-store.ts)

- [ ] 4.1 On transport error after first chunk WITH `serverRunIdRef` + `lastEventId`: enter a reconnect loop calling `resumeRunStream` → `readSseStream`/`handleBlock`
- [ ] 4.2 Bound reconnect by a small cap + the existing retry budget; finalize via existing terminal handling on exhaustion
- [ ] 4.3 Relax `shouldRetryStreamAttempt` so `stream_already_started` routes to resume (not terminate, not re-POST); pre-first-chunk path unchanged
- [ ] 4.4 Client cursor guard: ignore blocks with id ≤ `lastEventId` across the reconnect boundary (belt-and-suspenders vs server id filter)

## 5. Validation (gate)

- [ ] 5.1 `bun run typecheck` — touched files clean (pre-existing unrelated errors noted)
- [ ] 5.2 `eslint` clean on `chat-stream-store.ts` + `chat-stream-api.ts`
- [ ] 5.3 Manual: normal stream unaffected; simulated mid-stream drop resumes remaining tokens, no duplicate message/run (pending live env — document if not runnable here)
- [ ] 5.4 `openspec validate resumable-streaming-client --strict`; update `.kbd-orchestrator` progress

## Notes

- Frontend-only; server resume path already complete.
- Keep `fetch`/`ReadableStream` for POST (need headers); resume is a GET.
- Dedup is server-authoritative (id filter) + client cursor guard.
- Out of scope: `EventSource` switch; cross-page-reload resume (in-memory cursor); visual reconnect indicator (would trigger UI/UX routing).
- This change is store/service logic (data layer), not visual UI/UX — frontend layering rules apply; visual-design routing does not.
