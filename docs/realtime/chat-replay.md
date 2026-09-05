# Primary chat stream replay

Both `/api/chat/completion` and `/v1/chat/completions` accept an authenticated
reconnect to the original run. Retain the initial response's `x-uar-run-id` and
the last fully processed SSE `id`. Send both back as `x-uar-run-id` and
`Last-Event-ID`, with `stream: true` and the same `stream_mode` in the JSON body.
No new message is required for replay. The supplied model, message, attachments
and run-policy fields do not create a new turn on this path.

New IDs are opaque cursors shaped as `event:frame:format`. The format number is
0 for OpenAI, 1 for AG-UI, 2 for dual, and 3 for AG-UI spec. Store the complete
ID; do not parse it as an integer. A legacy numeric ID acknowledges every frame
of that runtime event. Frame cursors can resume between the several frames that
one runtime event produces. Completion IDs are derived from the original run.

Replay checks the original verified subject and tenant, not merely session
identity. Anonymous replay is rejected. An explicitly supplied session must
match. A foreign or unknown run returns 404; malformed, future or format-mismatched
cursors return 400. Missing retained projection history returns 410. None of
these responses starts another model call.

Replay does not repeat input processing, memory recall/capture or post-response
quality model calls. It seeds text, reasoning and tool projection state from
retained events, suppresses acknowledged frames, and subscribes before taking
the history snapshot so replay and live events do not overlap.

## Limits

The uncomfortable limit is in-process retention: the current event buffer holds
512 runtime events and has no durable projection checkpoint. Once the prefix is
evicted, primary-chat replay returns 410 even if the cursor itself is still in
the buffer. Server restart also loses this replay state. Existing last-viewer
disconnect cancellation and its 250 ms grace period remain in force; reconnect
never restarts a cancelled run. Local HTTP regression coverage passes for all
four formats, owner/tenant isolation, strict cursor ordering, legacy and terminal
cursors, no second primary execution, and retained-cursor prefix eviction. Replay
with enabled memory/quality side effects and cancelled-run HTTP replay remain
unverified by that fixture; the full phase suite is a separate completion gate.
