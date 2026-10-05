# Owner-scoped run event snapshots

`GET /api/uar/runs/{id}/events?after=0` returns a read-only snapshot of the same public event projections available from the ordinary run SSE stream. `after` is an exclusive unsigned event cursor and defaults to zero. The existing authenticated subject/tenant/run-owner predicate applies; unavailable and foreign runs/history return 404. Reading never subscribes, approves, dispatches, cancels or creates a disconnect guard. SSE behavior is unchanged.

The camelCase version 1 envelope contains:

- `version: 1`, `runId` and the requested `after`.
- `cursor`: the latest retained source event ID, or zero for empty history.
- `retention: "process-local-bounded"`.
- `firstAvailableEventId`: the first retained source ID, or null.
- `gapReason`: null, `"retention-gap"`, or `"cursor-ahead"`.
- `events`: ordered `{eventId, eventName, data}` records using the existing public AG-UI SSE name and payload semantics.

`agui.tool_call.complete` contains `name`, `id`, `arguments_json`, `call_index`, and `request_id`; its `agui.tool_result` carries the same call `id`, `name`, `content`, and `success`. Consumers can retain those owner-scoped records before application shutdown and correlate exact requested arguments with the native result. Tool arguments/output remain owner-visible data; credentials, admin keys and internal admission authority objects are not added to the snapshot.

The source history is an existing process-local 512-event ring with the existing separately retained latest presentation. Restart/shutdown loses this history. This endpoint adds no durable trace store and makes no durable replay guarantee. Missing retained IDs after the requested cursor produce `retention-gap`, including any hole between a retained presentation and the ring. `cursor-ahead` means the supplied cursor exceeds this captured history's last source ID. Consumers must preserve the gap indication and cannot claim a complete trace from such a response. Public projection omission alone is not a retention gap: `cursor` covers every retained source ID, including events without a public projection. An empty `events` array therefore need not mean an unchanged cursor.

The observed C14 attempt b7ad66b9-98a3-4df4-9617-b488f1864b7a reported a roster revision conflict, but its exact roster arguments were unavailable after shutdown. No empty cursor is assumed and no roster or write compare-and-swap condition is changed. The later actual packaged repeat must capture the input before choosing any roster correction. Source implementation and metadata do not prove that operation.
