# Design

Read existing history_since(runId,None) exactly once after the same get_run_for_context check as stream. Project through existing public to_agui_event; no internal credentials/admission objects become response data. Wrapper fields are version=1, runId, after(default0), cursor(latest retained ID or0), retention=process-local-bounded, firstAvailableEventId(nullable), gapReason(nullable retention-gap or cursor-ahead) and events(eventId,eventName,data).

All retained IDs are examined in order before public projection; missing IDs after the requested cursor produce retention-gap. An after above the captured latest ID produces cursor-ahead. Projection omission does not itself signal lost history. cursor advances over retained internal IDs too. Existing history includes a separately retained latest presentation; any gap between that record and the ring still marks incomplete retention.

Only process-local current history is exposed, presently a bounded 512-event ring plus existing retained presentation. It is not durable replay or restart recovery. Missing/foreign/unavailable runs return404 without disclosing existence. Snapshot cannot cancel, subscribe, approve, delegate or mutate. Existing SSE remains byte/behavior-compatible.

The observed roster call's raw arguments remain unavailable; this repair adds inspection, not a guessed cursor fix. Completion evidence requires root's actual build and normal packaged repeat. No worker gates.

## Source boundary

Source implements the owner predicate and a single history snapshot without touching runtime, subscriptions or authority. Metadata labels runs.events as Owner/Read and OpenAPI references the same versioned envelope schema. Scoped rustfmt applied only to the new API module. No tests, compilation, build or review ran. Signed plan/production commits suppress repository hooks invocation-locally under the explicit parent delivery policy; native build and actual trace capture remain parent-owned.
