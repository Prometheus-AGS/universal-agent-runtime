# Lifecycle console transport handoff

## Changed console paths

- `frontend/src/lib/realtime/sse-stream.ts` owns one fetch-based SSE attempt.
  It has no native `EventSource` retry behavior and no application timer. An
  `online`, visible-document, or explicit `uar:realtime-reinitialize` event
  may request one new connection attempt.
- `frontend/src/lib/realtime/uar-sse-adapter.ts` multiplexes `/api/live`,
  subscribes before every snapshot request, and treats every live payload as
  an invalidation. During recovery it discards row payloads and coalesces them
  into one newer authoritative snapshot. Complete snapshots become one graph
  changeset; incomplete, duplicate-identity, or generation-mismatched
  snapshots only invalidate graph lists.
- `frontend/src/entities/sync.ts` applies the same contract to the embedded
  stream. Recovery state is per topic, so a snapshot for one table cannot
  release another topic's reset. A legacy snapshot without `complete: true`
  may upsert rows but never deletes rows.
- `src/server/lifecycle_snapshot.rs` owns the reusable server-side snapshot
  bridge. The server composition owner must capture a clone of the complete
  state-bound router before adding `/api/live/{topic}/snapshot`, then call
  `LifecycleSnapshotService::response(topic, headers, extensions, generation)`.
  It replays only established caller-authorized GET routes through `tower`
  `ServiceExt::oneshot`.

## Applied frontend contract

This work changes transports and normalized entity ingestion, not rendered UI.
The Vercel React listener guidance led to one shared multiplexed listener and
explicit cleanup when the final subscriber leaves. The composition guidance
keeps connection mechanics behind the transport boundary. The Prometheus entity
realtime guidance keeps every durable update in a typed adapter `ChangeSet`;
components do not receive stream rows or mutate graph state directly. The
Impeccable, frontend-design, and UI/UX Pro Max guidance had no visual surface
to apply.

## Wire contract

`GET /api/live/{topic}/snapshot` must return this object for every public
topic, including failures represented as an incomplete response:

```json
{
  "version": 1,
  "topic": "knowledge_bases",
  "generation": 42,
  "complete": true,
  "records": [{ "id": "kb-1" }]
}
```

An unavailable, denied, collision-prone, or otherwise non-authoritative
snapshot must set `complete` to `false`, return no records, and use a stable
non-secret `reason`. Clients must not delete cached entities for that response.

SSE resets use `event: reset`. The accepted payload forms are the direct gap
form `{version, topic, reason, requires_snapshot: true}` and the lifecycle
envelope `{topic, id: "", data: {version, generation, ready, reason}}`.
The console treats either as a snapshot requirement; a lifecycle envelope need
not duplicate `requires_snapshot`.

The embedded stream must publish `entity.snapshot` with `{version: 1,
topic|table, generation, complete: true, records}` and `entity.change` with
`{topic|table, generation, action, id, record?}`. A reset without a topic
applies to every public embedded topic. The stream must send a new complete
snapshot after a reset or a client reinitialization. Partial snapshots are
allowed only as non-destructive upserts.

## Snapshot source mapping

| Topic | Existing internally replayed GET route | Normalization |
| --- | --- | --- |
| knowledge_bases | `/api/knowledge` | Array rows |
| knowledge_documents | bases then `/api/knowledge/{id}/documents` | Flattened caller-visible documents |
| agents | `/api/agents` | `runtime_agents` and `federated_agents`, retaining `_type`; duplicate IDs are incomplete |
| providers | `/api/uar/providers` | `providers` rows |
| models | `/api/models` | Flatten catalog using the existing `<provider_id>/<model_id>` graph ID, plus raw `model_id` and `provider_id` |
| skills | `/api/skills` | Array rows with `skill_id` normalized to graph `id` |
| settings | `/api/uar/settings` | Existing admin-masked list; denied remains incomplete |
| memory | `/api/admin/memories` | Existing trusted-host `items`; denied remains incomplete |
| compiler_sessions | `/api/compiler/sessions` | Array rows |
| threads | `/api/uar/sessions` | Owner-scoped Session state becomes `{id, title: "New conversation", created_at, updated_at: last_activity}`; titles are not durably stored by the session source |

Model snapshots reuse the established `features/models/model/models-graph.ts`
identity `<provider_id>/<model_id>`, so model names shared by providers remain
distinct. Agent snapshots cannot do the same. The current agent graph loader
(`features/agents/model/agents-graph.ts`), API parser
(`features/agents/api/agents-api.ts`), and backend mutation routes all use the
raw agent ID. A source-qualified agent identity would require a coordinated
migration across those typed transports and mutation paths. Until that work is
accepted, a runtime/federated collision returns `complete: false` and never
merges or deletes agent state.

The `/api/uar/sessions` source route must preserve the authenticated
`UserContext` owner filter supplied by `PersistenceLayer::list_sessions`; the
snapshot bridge intentionally omits `owner_id`, messages, and system prompt
from the public Thread records.

## Lifecycle and cleanup

The entity graph's existing `teardownEntityGraph` unregisters its adapters.
Unregistering the last multiplexed listener aborts its fetch reader and removes
the browser recovery listeners. Embedded adapters perform the same cleanup, so
logout and HMR do not retain stream readers.

No test, lint, format, typecheck, build, compiler probe, or review gate ran.
The active phase verification lock is closed. The future acceptance work needs
to exercise reset-before-snapshot, change-during-snapshot, incomplete snapshot,
generation mismatch, listener cleanup, and every source mapping through the
fully wired server route.
