# resumable-streaming-client

## Why

The server already supports resumable streaming end-to-end — a 512-event ring buffer with monotonic `u64` ids and a working `GET /api/uar/runs/{id}/stream?last_event_id=` endpoint that honors the `Last-Event-ID` header and merges replay+live (`routes.rs:63-94`). **The client never uses it.** The chat store streams via `POST /api/chat/completion`, `parseSseBlock` discards the SSE `id:` line (`chat-stream-store.ts:225`), and `shouldRetryStreamAttempt` **gives up entirely once the first chunk arrives** (`chat-stream-store.ts:476` — `if (sawFirstStreamChunk) return { retry: false, reason: "stream_already_started" }`). So a mid-stream network drop loses the rest of the response with no recovery. This is HP5 of `uar-harness-parity` — a frontend-only change to wire the client to the already-complete server resume path. (HP1 already added the `x-uar-run-id` response header and captures it as `serverRunIdRef`.)

## What Changes

- **Capture the SSE event id:** `parseSseBlock` returns the `id:` line; the read loop tracks the highest `lastEventId` seen.
- **Reconnect instead of giving up:** when the stream drops mid-response (transport error after the first chunk) AND the server `run_id` (`serverRunIdRef`) and a `lastEventId` are known, the client reconnects via `GET /api/uar/runs/{run_id}/stream` with the `Last-Event-ID` header set to `lastEventId`, and continues consuming the SAME event-dispatch logic from where it left off — rather than the current "stream_already_started → stop" behavior or a duplicate re-POST.
- **Bounded reconnect:** reconnection respects a reconnect attempt cap + the existing retry budget; on exhaustion the stream ends cleanly (current terminal behavior). The initial-POST retry path (before any chunk) is unchanged.
- **No duplicate runs:** reconnect uses the resume GET (read-only on the existing run), never a second POST, so a run is never started twice.
- **Refactor for reuse:** extract the per-SSE-block dispatch into a single closure used by both the initial POST stream and the resume GET stream (no duplicated event handling).
- A `cancelRun` / explicit stop continues to work (HP1) and cancels the underlying run.

Out of scope: switching the primary transport to `EventSource` (we keep `fetch`/`ReadableStream` for POST + headers); server-side changes (already complete); UI redesign (a small, optional "reconnecting" status is allowed but not a visual redesign).

## Capabilities

### New Capabilities
- **`resumable-streaming`** — `specs/resumable-streaming/spec.md`. Client tracks the last received event id; on a mid-stream drop it resumes via the server's `Last-Event-ID` endpoint without duplicating the run or replaying already-handled events; reconnection is bounded; clean responses are unaffected.

## Impact

- **Affected code (frontend only):** `frontend/src/stores/chat-stream-store.ts` (capture id, track `lastEventId`, extract block-dispatch closure, reconnect loop, fix retry gating), `frontend/src/services/chat-stream-api.ts` (add a `resumeRunStream(runId, lastEventId, signal)` GET service). No backend change (the resume endpoint + `x-uar-run-id` header already exist).
- **Layering:** respects component→hook→store→service — the new GET is a service; the store orchestrates; no component/hook changes required (the existing Stop button keeps working).
- **APIs:** consumes the existing `GET /api/uar/runs/{id}/stream?last_event_id=` (and/or `Last-Event-ID` header). No new endpoints.
- **Runtime/UX:** a dropped connection mid-response now recovers the remaining tokens instead of truncating; no duplicate assistant messages; no duplicate runs/cost.
- **Dedup correctness:** the server merges replay+live filtering by event id; the client must not re-apply events with id ≤ `lastEventId` (the server already filters, but the client tracks `lastEventId` as the resume cursor).
- **Edge case:** if the resume id predates the 512-event window, the server replays from the oldest available; the client continues from there (a small gap is possible on very long drops — acceptable, documented).
- **Security:** no new data exposure; same-origin GET with the run id (already used for cancel).
- **KBD workflow state:** YES — HP5 of `uar-harness-parity`.
