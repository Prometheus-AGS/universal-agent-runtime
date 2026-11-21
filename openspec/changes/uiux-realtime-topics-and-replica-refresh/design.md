## Context

Phase `uar-uiux-architecture-2026-11`; build items 14, 17, 24; unresolved review finding
critical 1; scenario area RTM. Claims carry a file:line read in this session unless marked
UNVERIFIED. Nothing was built or run.

Version note for the lead: the task text mentions SurrealDB 3.2.4 pins. `versions.toml:35-38`
and `Cargo.toml:624` pin SurrealDB **4.0.0** (SDK, types, server and image), and the SDK is
path-patched (`Cargo.toml:88-91`, `vendor/patches/surrealdb-lifecycle/`). This design follows
the files. Live-query behavior across several connections to one remote server is therefore
UNVERIFIED for 4.0.0 until the gate runs; no claim below depends on a 3.x document.

Source facts:

- `EntityTopic` has 20 variants; `ALL` enrolls all of them, `table()` maps one table per topic,
  `is_public()` hides ten host-only topics (`src/uar/realtime/mod.rs:88-198`).
- The Surreal bus opens one `select(table).live()` per topic except memory, after an idempotent
  `DEFINE TABLE IF NOT EXISTS .. SCHEMALESS` (`surreal_bus.rs:61-66`, `:164-170`). Events are
  decoded from the notification and broadcast on that topic's channel, except
  `governance.enabled` events, which are dropped (`:184-201`).
- The Postgres bus uses one `LISTEN uar_entity_change` connection and decodes
  `{topic, id, action, data?}` (`postgres_bus.rs:14-30`, `:102-164`).
- Both buses are lifecycle-owned: a host-issued reconnect advances a generation; a transition
  broadcasts a `Reset` event with the generation and readiness (`lifecycle.rs:95-105`).
- `/api/live` merges every public topic's receiver; each event becomes a `reset` frame with
  `topic`, `requires_snapshot: true`, generation and readiness and nothing else
  (`src/uar/api/live.rs:62-119`). `/api/uar/sync/stream` does the same for embedded stores and
  returns 404 for a remote Surreal URL (`src/server.rs:4824-4860`). The snapshot route calls
  `LifecycleSnapshotService`, which replays the owner's read routes with the caller's
  extensions (`src/server.rs:2992-3006`, `src/server/lifecycle_snapshot.rs:50-90`).
- Postgres: the generic trigger function reads `rec.id::text` (`migrations/20260220000002_notify_entity_change.sql:23-34`),
  which the two policy tables cannot supply because their key is `(owner_id,
  conversation_id)` (`migrations/20260904000001_principal_conversation_policies.sql:2-8`). The
  later lifecycle function emits `id: '*'` and does not swallow failure
  (`migrations/20261020000000_lifecycle_work_notifications.sql:1-35`); it is the right shape.
- `ProviderRegistry` has `register_custom_provider` (upsert), `remove`, `set_default` and an
  `administration_guard` taken by the create, update, delete and model-test handlers
  (`src/llm/registry.rs:280`, `src/uar/api/providers.rs:203,276,304,428`).
  `register_custom_provider` preserves an existing `default_model` and `api_key` when the
  incoming config has none (`registry.rs:444-480`), which is wrong for an authoritative
  refresh of `default_model`.
- `SettingsManager.get_value` reads the cache first and the store only for a missing key
  (`manager.rs:468-489`). `set_value` is the only post-boot writer of the cache and of the
  "effective" map (`:627-664`).
- Conversation policy reads fall through from persistence to an in-process `agent_sessions`
  map when persistence has no record (`src/uar/api/discovery.rs:715-735`); saves and deletes
  update it (`:955-958`, `:1255-1262`). It is per replica.

## Goals / Non-Goals

Goals: every public topic has a working source on embedded Surreal, remote Surreal and
Postgres; a provider, default-provider or settings change made through any replica is the
state every replica serves for the next snapshot and the next turn; a client told "changed" is
never answered with the old state; failure to be current is reported, not hidden.

Non-goals: row payloads on the wire; replaying missed events; cross-replica governance posture;
credential-revision sync; team run state; frontend ownership of Provider/Model records.

## Decisions

### R1. Topic table

| Wire topic | Source on Surreal | Source on Postgres | Snapshot |
|---|---|---|---|
| `providers` | `settings` rows with key `provider.*`, and key `llm.default_provider` | trigger topic `providers` (existing, `provider.%`) plus key-carrying payload for `llm.default_provider` | required; records carry `is_default` |
| `models` | derived from `providers` and from `model.*` keys | derived the same way | required |
| `settings` | all other `settings` rows, and `llm.default_provider` | trigger topic `settings` | required |
| `conversation_policies` (new) | tables `conversation_policies` and `principal_conversation_policies` | new triggers on both tables, existing id-less function, topic `conversation_policies` | not required |
| `team_catalog` (new) | table `uar_collaboration_state` | new trigger on that table, topic `team_catalog` | not required |

Rationale for deriving `models`: `/api/models` is built from the provider registry
(`lifecycle_snapshot.rs:165-193` reads `/api/models`), so a provider change is a models
change; inventing a `models` table writer would duplicate state. Consequence: the `models`
snapshot, which replaces the graph's Model set, starts firing. It is gated on
`uiux-model-identity-graph-ownership` (design R9).

`llm.default_provider` is routed to both `settings` and `providers`: it stays a settings row
for existing consumers and it is what the provider list's default depends on.

### R2. One routing function for both stores

`realtime/mod.rs` gains `route(event: &LiveEvent) -> &'static [EntityTopic]`:

- topic `Settings` (Surreal) or any settings-originated event: classify by `data.key` when
  present, else by the record id's key part: `provider.` / `provider_` → `[Providers, Models]`;
  `model.` / `model_` → `[Models]`; exactly `llm.default_provider` / `llm_default_provider` →
  `[Settings, Providers]`; otherwise `[Settings]`;
- topic `Providers` (Postgres trigger) → `[Providers, Models]`;
- topic `Models` → `[Models]`;
- everything else → the event's own topic.

The Surreal decoder and the Postgres decoder both call it, then broadcast on each returned
topic. The record id string format that Surreal returns for the `settings` table is UNVERIFIED
(`surreal_bus.rs:191-193` takes `json["id"]` as a string); the key is read from
`data.key` first, and the id fallback is only for a delete whose payload lacks it. Task 2.3
reads the actual shape from the patched SDK before relying on the fallback.

### R3. Multi-table topics and derived readiness (Surreal)

`EntityTopic::table()` becomes `tables() -> &'static [&'static str]`:
`ConversationPolicies` → both policy tables; `TeamCatalog` → `uar_collaboration_state`;
`Providers`, `Models` → `settings`. The supervisor opens one live stream per table, merged
into one receiver loop. A topic is `ready` only when all of its streams are registered. Because
`Providers` and `Models` share the `settings` source, their readiness follows that stream and
cannot read "ready" while the stream that feeds them is down (today the empty `providers` and
`models` tables would report ready regardless).

A live query on `uar_collaboration_state` and on the two policy tables must be allowed by
whatever table permissions the persistence user has; the bus already issues the same kind of
statement for the other topics, so no new privilege class is introduced (permissions on a
remote deployment are UNVERIFIED).

### R4. Per-topic snapshot declaration and frame content

`EntityTopic::requires_snapshot()` is false for `ConversationPolicies` and `TeamCatalog` and
true for the rest. `live.rs` (`sse_event`, `gap_event`) and `sync_stream_handler` read it
instead of the constant `true`. The frame still carries only `version`, `topic`,
`requires_snapshot`, `reason`, `generation`, `ready` and no id, owner, key or value.
Frames are broadcast to every connected client, as today; the new topics reveal only that
some policy or catalog changed. That is the same disclosure level as `agents`.

Why no snapshot for these two: both are owner-private and have no list route
(`PersistenceLayer` has load/save/delete for conversation policies only,
`src/uar/persistence/mod.rs:734-759`). A snapshot would need a new list method on three
providers and owner-scoping rules for legacy versus principal keys (`.prometheus/gotchas.md`,
2026-09-04). Instead the client refetches what it holds through the read routes that already
apply owner scope (R9). The snapshot route for these topics answers `200` with
`complete: false, reason: "owner_scoped_refetch"` so a stray call is harmless.

### R5. Postgres migration (additive)

One new migration, named with a timestamp after `20261031010000` (proposed
`20261120000000_realtime_replica_topics.sql`):

1. `CREATE OR REPLACE FUNCTION uar_notify_settings_change()` with the same topic mapping and
   an added `'data', json_build_object('key', row_key)` in the payload. The key is a setting
   name, never a value; the payload stays far below the 8000-byte NOTIFY limit. The decoder
   already accepts `data` (`postgres_bus.rs:25-26`).
2. Triggers on `conversation_policies` and `principal_conversation_policies` using
   `uar_notify_lifecycle_work('conversation_policies')`, and on `uar_collaboration_state` using
   `uar_notify_lifecycle_work('team_catalog')` under a distinct trigger name (the existing
   `trg_lifecycle_uar_collaboration_state` keeps its host-only topic). Each guarded by
   `to_regclass(..) IS NOT NULL` like the existing migration (`:15-34`).

`EntityTopic::from_str` gains the two wire names; `ALL`, `table`/`tables`, `as_str` and the
host-only list are updated together. The migration is authored here and applied by the
product's own startup on a scratch database at the gate; nothing in this change runs a
migration. `LISTEN` requires a direct connection: a transaction-mode pooler in front of
Postgres breaks it (limitation; UNVERIFIED for the production topology).

### R6. Replica state

New `src/uar/realtime/replica_refresh.rs` (uar-state). `ReplicaState` holds the settings
manager, the provider registry and the bus. It is built after settings hydration and started
under the supervisor (`src/server.rs`, next to the committed-work subscriptions at
`:2286-2320`).

*Epoch.* `BusLifecycle` gains a per-topic `AtomicU64` change epoch and `stamp(topic)`, called
in each bus's decode step immediately before `sender.send`. `RealtimeBus::observed_epoch(topic)`
(default `0`) exposes it. Stamping before broadcasting is the happens-before edge: no client
frame can exist for a change whose epoch is not yet visible.

*Triggers.* Subscribe, then read (the order the committed-work code already uses, comment at
`src/server.rs:2281-2283`). Refresh on: startup (after subscribing); any event on `Providers`,
`Settings` or `Models`; any `Reset` with `ready: true`; any `Lagged`. A `Reset` with
`ready: false` marks the feed unavailable until the next ready reset. Nothing else: a periodic
timer is rejected because no failure scenario was observed that the reset, lag and event paths
leave open.

*Refresh (single flight, coalescing).* One task. An arriving trigger sets `dirty`; the task
loops while `dirty`. Per pass: read `epoch_before = observed_epoch(topic)` for each of the
three topics; take `registry.administration_guard()` (so a local create/update/delete is never
interleaved); then

1. `SettingsManager::reload_cache_from_store()`: list all settings rows; build the new cache
   and effective maps off to the side; for each row, if its data is unchanged keep its cache
   metadata, else store it with source `Api` when `updated_at` is set and `is_drift: false`
   (mirroring `set_value`); remove rows that no longer exist; update the effective map only
   for keys the settings change's class table (`uiux-settings-truthful-status-and-removals`
   design D1/D3: `live`, `next_turn`) marks as read per request, never for `restart` or
   `inert` keys, so the refresher cannot report a boot-only key as in effect (a class table
   that does not exist yet falls back to the current three-way `setting_apply_mode`);
   skip `governance.*` entirely (its
   posture is published by the runtime authority, `.prometheus/gotchas.md` 2026-08-28) and
   leave `internal.model-verification.*` as read; swap both maps under the write lock.
2. `ProviderRegistry::apply_persisted(configs, default_id)`: add or replace each persisted
   provider with enrichment and pricing validation (as `register_custom_provider`) but taking
   `default_model` from the row as authoritative, keeping an in-memory `api_key` only when the
   row has none; remove providers absent from the store; set the default from
   `llm.default_provider` when that provider exists, else keep the current default and record
   a warning. A skipped undecodable row (from the boot safety change) is kept as is.

   *Why applying the default row is safe.* Every write comes back through the feed, including
   the writer's own, so applying the row means any write to it becomes live on every replica.
   That is correct only if the row has one writer. The row is written by the live default
   route (`POST /api/uar/providers/{id}/default`, `src/uar/api/providers.rs:367-399`, via
   `set_default_provider_id`, `src/uar/settings/manager.rs:1248-1250`) and by provider
   deletion, which writes `""` when the default is removed (`providers.rs:334-336`). The
   settings API SHALL reject writes to `llm.default_provider`; that rejection, its status code
   and its error body (pointing to the live route) are owned by
   `uiux-settings-truthful-status-and-removals` and this change depends on it (task 1.5). With
   that rejection in place a settings-API write cannot reach the row, and refresh keeps
   applying the row on every replica. Without it, refresh would create a second live system
   model path, which operator decision 2 rules out; the two changes ship together or the
   default step of refresh is not enabled.
3. Set `applied[topic] = epoch_before` and notify waiters.

An own-write echo triggers the same pass and is idempotent. Pass failure keeps the previous
state, records `Stale{since, class}`, and retries with capped backoff (cap 30 s, the cap the
existing reconnect paths use; `live` adapter spec "Reconnection with Backoff"); the stale
state clears on the next success. Logs: `realtime.replica.refresh_failed` once per
transition, never per retry.

*Barrier.* `ReplicaState::await_current(topics, deadline)` reads `observed_epoch` for each
topic at call time and waits until `applied[topic] >= observed` or the deadline. The deadline
is `config.resilience.admission_timeout_ms` (read at `src/server.rs:1092`; its default value is
UNVERIFIED). The snapshot route calls it for `providers`, `models` and `settings` before
replaying the read route. Outcomes: current → normal snapshot; deadline →
`complete:false, reason:"replica_refresh_pending"`; `Stale` →
`complete:false, reason:"replica_refresh_failed"`; topic not ready on this bus →
`complete:false, reason:"subscription_unavailable"`. The client already treats
`complete:false` as "mark lists stale" (`frontend/src/lib/realtime/uar-sse-adapter.ts:360-368`, the `!snapshot.complete` branch of `recover`).

*Turns.* The next turn reads `registry.default_model()` (`registry.rs:551-563`) after the pass,
so the turn-time default is current without a barrier. The bound is feed delivery plus one
pass; the product limit is UNKNOWN and the gate uses a deadline of its own (below).

*Conversation policy cache.* On any `ConversationPolicies` event, reset or lag the replica
clears `AppState.agent_sessions` (`src/lib.rs:154`). With persistence configured it is a
read-through copy (`discovery.rs:715-735`), so clearing is safe and removes a deleted-on-A,
served-on-B policy. Only `discovery.rs:728,893,955,1260` touch it (searched `src/`).

*Provider refresh interface for the config change.* `ReplicaState::refresh_now()` runs one
pass and returns its result; `uiux-config-boot-safety-and-live-reload` calls it through a
trait so the live registry matches rows it just wrote.

### R7. Snapshot content

`LifecycleSnapshotService::providers` keeps `payload.providers` and adds
`"is_default": id == payload.default_id` to each record (additive; the frontend `Provider`
entity must accept the field). If `uiux-system-model-live-default` specifies a different
carrier for the default id, that change wins and this one only guarantees the invalidation.

### R8. Embedded stream

`sync_stream_handler` serves the same public topic set from the same bus and the same
`requires_snapshot()`; its 404 for a remote Surreal URL stays (remote stores use `/api/live`).
Open question, not built: the embedded frontend adapter waits for `entity.snapshot` frames
after every reset (`frontend/src/entities/sync.ts:256` sets `awaitingSnapshot`, `:267` handles `entity.snapshot`), and no `entity.snapshot`
or `entity.change` frame is emitted anywhere in `src/` (searched). Whether embedded clients
therefore only get list invalidation is UNVERIFIED; the lead should decide whether to move
embedded to the `/api/live` adapter.

### R9. Frontend consumers (owner `uar-console`)

- `topics.ts`: add `conversation_policies` and `team_catalog` to the topic map with their
  snapshotless flag; `createUarSseAdapter` already exposes `onReset` for this
  (`uar-sse-adapter.ts:61` option, `:349` call).
- On a `conversation_policies` reset, every `AgentSession` entity loaded in the graph is
  refetched through its existing `get` transport, which applies owner scope
  (`session-configuration/registration.ts:agentSessionTransport`). On a `team_catalog` reset,
  loaded team packages and bindings are refetched through the teams domain loaders.
  Both go through `@/platform/entities`; components and feature hooks do not import the graph
  or transports (project rule). The exact PEM refetch call is chosen at implementation.
- A `providers` reset must also refresh `ConfiguredProvider` and `ConfiguredModel`, which are
  fetched by their own transports and are not the `Provider` / `Model` types the topic maps
  to (`session-configuration/registration.ts:30-82`).
- `sync.ts` embedded adapter: a reset with `requires_snapshot: false` is an invalidation, not
  an error (today `:262` sets status error for it).
- Enable the `models` fan-out only together with or after `uiux-model-identity-graph-ownership`
  (Model ids already `provider/model` in the snapshot, `lifecycle_snapshot.rs:178-185`).

### R10. Postgres in the gate

The postgres code is behind the non-default feature (`Cargo.toml:318-321, 399`). The gate
builds one binary with `--features server-full,postgres-backend` and runs both store modes
from it. A Postgres-only defect cannot hide behind a default build.

## Risks / Trade-offs

- **Refresh cost.** Every provider or settings change triggers a full settings list on every
  replica, including the echo on the writer. The settings table is small (hundreds of rows;
  UNVERIFIED count) and the pass coalesces. A change flood coalesces to at most one pass in
  flight plus one queued.
- **Barrier deadline vs. slow store.** A slow store yields `replica_refresh_pending` frames
  and client retries instead of stale data. Acceptable; it is the honest answer.
- **`ready: false` window.** While a replica's feed is down it answers incomplete snapshots for
  three topics; clients show stale-marked lists. Intended.
- **Governance and restart-class keys are not applied live by the refresher.** A governance
  change on replica A is not adopted by replica B until B restarts (existing behavior, now
  stated).
- **Provider credential rotation** across replicas is not covered (credential revisions are a
  separate store).
- **Registry removal race.** A provider created on A and removed from B's registry by a pass
  that read before A's write committed is corrected by the echo pass; the window is one pass.
- **Fan-out to `models` activates a fourth Model writer.** Gated as above.
- **Disclosure.** Frames reveal the occurrence of a policy or catalog change to every
  connected principal.

## Verification surface (authored at the boundary, not now)

Shared fixture contract (every RTM scenario):

- scratch root `$SCN_ROOT/rtm-<NNN>/` with `store/`, `replica-a/{home,data}`,
  `replica-b/{home,data}`, `stub/`, `logs/`;
- one scratch remote store per run: a SurrealDB server at the `versions.toml` pin
  (`surrealdb_server 4.0.0`, image digest in `versions.toml:38`) with a scratch data path, or a
  scratch Postgres with the `pgvector` extension, reached directly (no pooler). Store
  authentication is the fixture's own scratch configuration between the replicas and the
  store and is not a UAR API credential; whether the Surreal connector accepts an
  unauthenticated scratch server is UNVERIFIED;
- two production-binary replicas, same YAML, same `persistence.database_url`, distinct
  ports from the gate's reserved scratch range (never `1906` or `28000`), distinct `HOME` and
  data directories, `security.jwt_required: false` behind the existing external identity
  proxy so the owner is supplied by infrastructure. How that proxy is launched twice is
  UNVERIFIED; the runner reuses the existing launcher. Requests carry no JWT, key or login;
- **No paid model call (R4).** The only upstream provider is the repo's `stub-llm` binary
  (`src/bin/stub-llm.rs`, `[[bin]] stub-llm`, `required-features = ["server"]`,
  `Cargo.toml:127-130`). Invocation, read from source: `stub-llm <fixtures.json>` or
  `STUB_LLM_FIXTURES=<file>`, port from `STUB_LLM_PORT` (default 4600), listening on
  `http://127.0.0.1:<port>/v1`. A fixture entry is
  `{model, last_user_message, has_tools, has_tool_result, response}` with `response.kind` one
  of `content`, `grounded_content`, `tool_call{name, arguments}`; matching is by exact
  fingerprint. The fixtures are frozen in `fixtures/replica-scenario.json` and
  `fixtures/capability-inversion.json`. Whether the request reaches the stub with the bare
  model id (`m-one`) or a provider-qualified one is UNVERIFIED (a known provider hint is
  stripped before serialization, `.prometheus/gotchas.md` 2026-10-31); the runner records the
  fingerprint it observed;
- a plain SSE reader process attached to a replica's `/api/live`;
- a runner-owned TCP forwarder in front of each replica's store connection (the fault hook
  for SCN-RTM-003, -007 and -008; implementation task 5.5, mandatory);
- convergence deadline 10 s (gate parameter, `fixtures/replica-scenario.json`);
- build: `cargo build --locked --no-default-features --features server-full,postgres-backend`,
  once, serialized (the lead sets `--target-dir` and the clean-build rule);
- cleanup owner: the runner stops every process it started and removes the scratch root.

Prerequisites declared for the lead's build step (R5): the main binary above, the `stub-llm`
binary from the same build, a SurrealDB server binary or image at the pinned version, a
Postgres with `pgvector`, the forwarder fixture (task 5.5). Three mutant builds (`fixtures/mutants.json`: M1, M2, M3) are required by SCN-RTM-008 and
SCN-RTM-009, each in its own worktree under `~/.claude/worktrees/` made by
`scripts/worktree-new.sh`, built with its own `--target-dir` and a receipt. The lead's
runner-integrity gate also builds the baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52`;
SCN-RTM-001 to 007 must FAIL on it, SCN-RTM-008 and 009 must PASS on it (regression guards).

## Frozen oracle fixtures

Written now, read by the boundary runner, never edited by the implementer (R1). A change to
any file requires a revision of this change.

| File | Contents |
|---|---|
| `fixtures/topic-routing.json` | the 12 public topics with their snapshot requirement; five setting keys with the exact topics each must and must not emit; the six allowed frame keys and forbidden substrings; the four incomplete-snapshot reasons; the Surreal tables of the two new topics |
| `fixtures/replica-scenario.json` | providers `p-one` and `p-two`, default `p-one`, markers `RTM-MARK-ONE`/`RTM-MARK-TWO`, the stub fixtures, the 10 s deadline, forbidden ports, sample counts (20 ordering edits, 5 cross-replica changes, 3 feed-gap cycles), the next-turn key `prompt_caching.enabled` and the restart-class key `memory.max_context_tokens` (2000 to 1500, expected `restart_required`), the invalid policy route, body and its 400, the invalid-binding rule, the baseline commit with the `mustFail` list (SCN-RTM-001 to 007) and the `regressionOnly` list (SCN-RTM-008, 009), and the persist-ordering expectations |
| `fixtures/capability-inversion.json` | the 22 `/mcp/uar` tool function names allowed, the forbidden tool-name pattern, the four host boundaries with their source citations, the scripted `web_fetch` POST attempt and its stub fixtures, runs R0-R3 with their expected outcomes and verdicts, the unchanged-after assertions |
| `fixtures/mutants.json` | the three deliberately wrong builds (M1 settings-mutating MCP tool, M2 SSRF loopback rule removed, M3 registry swap before persist): file, anchor, exact edit, how each is produced, and the expected verdict it must cause |

Truth sources named here are operator decisions, source files read at write time, and
fabricated test data. No expected value comes from a route table, classifier, audit file or
register that the implementer writes.

## Acceptance

### Behavior: a default provider change through one replica takes effect on another

| Field | Declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-realtime-topics-and-replica-refresh` / tasks 1.5, 2.2, 3.1-3.7 |
| Production entry point | `POST /api/uar/providers/p-two/default` on replica A; then `GET /api/uar/providers` and a one-message `POST /api/chat/completion` (message `RTM-PING`, no model) on replica B |
| Real collaborators | two production-binary replicas; one scratch remote store (Surreal in one run, Postgres in the other); `stub-llm`; the external identity proxy; the real change feed (live query / LISTEN) |
| Boundary exercised | HTTP on both replicas; the store's change feed; provider registry on B; the provider call to the stub |
| Observable result | on B, `default_id` becomes `p-two` within 10 s, and the completion reply is `RTM-MARK-TWO`; before the change it was `RTM-MARK-ONE`; repeated for 5 alternating changes |
| Negative control | the default route called on A with `p-does-not-exist` returns `404`; B's `default_id` stays as it was and the reply marker is unchanged |
| Isolation | shared fixture contract |
| Prerequisites | build with `postgres-backend`; scratch store; `stub-llm` fixtures from `fixtures/replica-scenario.json` |
| Final local gate | build once, then the scenario runner `scripts/behaviors/realtime-replica-consistency.cucumber.mjs` (new, path proposed, following `scripts/behaviors/*.cucumber.mjs`) once per store mode |
| Evidence | pending |
| Limitations | upstream model replaced by `stub-llm` (declared); an agent's explicit model is not exercised (owned by `uiux-system-model-live-default`) |

### Behavior: a client told "changed" reads the new state from the replica it is connected to

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.1, 3.4, 3.5, 5.5 |
| Production entry point | edit then delete a provider through replica A while an SSE reader holds `/api/live` on replica B and immediately calls `GET /api/live/providers/snapshot` on B after each `reset` frame |
| Real collaborators | as above plus the runner's forwarder |
| Boundary exercised | `/api/live`, `/api/live/{topic}/snapshot`, bus epoch, replica refresh barrier |
| Observable result | for each of 20 consecutive edits the first snapshot after the `providers` frame reflects that edit, and a `models` frame follows; a final delete is reflected as absence |
| Negative control | with B's store forwarder cut, `GET /api/live/providers/snapshot` on B returns `complete:false` with a reason from the frozen list and never a complete record set |
| Isolation | shared fixture contract |
| Prerequisites | forwarder fixture (task 5.5) |
| Final local gate | same runner, per store mode |
| Evidence | pending |
| Limitations | 20 samples cannot prove the absence of a race; they make a systematic ordering defect visible |

### Behavior: a settings change reaches the other replica

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.1, 3.2, 3.4 |
| Production entry point | `PUT /api/uar/settings/prompt_caching.enabled` (true to false) on replica A; `GET` of the same key on B and a `settings` frame on B's `/api/live` |
| Real collaborators | as above |
| Boundary exercised | settings cache on B; settings topic |
| Observable result | B returns `false` within 10 s and its reader sees a `settings` frame; repeated 5 times alternating |
| Negative control | `PUT /api/uar/settings/memory.max_context_tokens` 2000 to 1500 on A (a genuinely restart-class key, class source frozen in the fixture): B reports the stored value with `application_status: restart_required` and has not applied 1500 to its running configuration; the same status as A reports |
| Isolation | shared fixture contract |
| Prerequisites | as above |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | `governance.*` is not adopted across replicas (stated limitation); `server.port` is deliberately not used because the settings change classifies `server.*` as inert (`not_applied`) |

### Behavior: conversation policy and team catalog changes publish a topic on every store

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 2.1-2.7, 3.7, 4.1, 4.3-4.5 |
| Production entry point | `PUT /api/uar/conversations/{id}/policy` then `DELETE` on replica A; a team deployment binding install on replica A; readers on replica B's `/api/live`, on a Postgres store, and on an embedded single process `/api/uar/sync/stream` |
| Real collaborators | as above; embedded run uses one process and an embedded SurrealKV store |
| Boundary exercised | the new topics' sources, frames, `GET /api/uar/conversations/{id}/policy` on B |
| Observable result | B's reader receives a `conversation_policies` frame with `requires_snapshot: false` whose keys are a subset of the frozen allowed set; B's `GET .../policy` returns the saved policy, then `204` after the delete; a `team_catalog` frame arrives after the binding install and B lists the binding |
| Negative control | `POST /api/sessions/{id}/agent-config` with `model: "no-slash-model"` returns `400` (the model-shape check at `src/uar/api/discovery.rs:872-880`, in `save_agent_session_config`, which persists through the same policy store) and no `conversation_policies` frame arrives within 10 s; an install of a binding that references an absent package is rejected with a 4xx and no `team_catalog` frame arrives |
| Isolation | shared fixture contract |
| Prerequisites | a conversation created through the real route; a minimal installable team binding fixture (UNVERIFIED size; absence is BLOCKED) |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | other principals' frames are not isolated by design (disclosure level of every existing topic) |

### Behavior: a replica that missed changes converges when its feed returns

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.4, 3.5, 5.5 |
| Production entry point | cut replica B's store forwarder, change a provider and a setting on A, restore the forwarder; 3 cycles |
| Real collaborators | as above plus the runner's forwarder |
| Boundary exercised | host auto-reconnect, `Reset` handling, full resync |
| Observable result | B serves the new provider and setting values within 10 s of the feed being ready again; B's reader saw a reset with `ready: false` then `ready: true` |
| Negative control | while the forwarder is cut, B's `providers` snapshot is `complete:false` with a reason, not the old complete set |
| Isolation | shared fixture contract |
| Prerequisites | forwarder fixture (task 5.5, mandatory); auto-reconnect active for server runs (`lifecycle_auto_reconnect::start`, `src/server.rs:1019`) |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | exercised on remote Surreal and on Postgres; not on embedded |

### Behavior: a failed persist publishes nothing and changes no replica (regression guard)

Classification: **regression guard**. The ordering already holds at the pre-phase baseline
`f2c9e36efe088e06a17c5d880ada877e25f2ad52`: `set_default` persists, then swaps the registry
(`src/uar/api/providers.rs:379-398`), and the feed can only carry a committed row. This phase
adds a refresher that applies stored rows, which makes the ordering matter on every replica,
but changes no ordering code. The scenario is mandatory and is expected to PASS on the
baseline; its sensitivity is proved by a mutant, not by a baseline failure.

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.4, 3.7, 5.5, 5.7 |
| Production entry point | `POST /api/uar/providers/p-two/default` on replica A while A's store forwarder is cut so that the write fails |
| Real collaborators | two replicas, the scratch store, the forwarder, `stub-llm` |
| Boundary exercised | durable write before live swap and before publication |
| Observable result | the route answers non-2xx within `resilience.request_timeout_ms` (scratch YAML sets 5000); after the forwarder is restored `default_id` is `p-one` on A and on B, the stored `llm.default_provider` is `p-one`, and B's reader received no `providers` or `settings` frame between the attempt and 10 s after restore |
| Negative control | two distinct inputs that a wrong implementation accepts. (1) Mutant **M3-swap-before-persist** (frozen in `fixtures/mutants.json`): the same fault input against a build that swaps the registry before persisting. The scenario must FAIL on it, observed as replica A `default_id == p-two` while the stored row is `p-one`. (2) Positive-flow control: with the forwarder restored the identical request returns `200`, replica A's `default_id` becomes `p-two`, and B's reader receives a `settings` frame within 10 s (the default is the `llm.default_provider` settings row, so a `settings` frame flows on every backend including the baseline; this proves frames flow, so "no frame" in the fault run is meaningful, and keeps the scenario a regression guard because cross-replica convergence and the `providers` frame are asserted by SCN-RTM-001 and SCN-RTM-003). Expected outcomes frozen in `fixtures/replica-scenario.json` `persistOrdering` |
| Isolation | shared fixture contract; the M3 build lives in a worktree under `~/.claude/worktrees/` with its own target directory |
| Prerequisites | forwarder fixture (task 5.5); the M3 mutant build (task 5.7). Whether the persist call is bounded by `request_timeout_ms` is UNVERIFIED and is part of what the scenario establishes |
| Final local gate | same runner, per store mode |
| Evidence | pending |
| Limitations | default route only; there is no explicit publish for this path, so a "publish before persist" mutant has nothing to mutate. The swap-before-persist mutant is the faithful wrong implementation |

### Behavior: an agent kernel cannot mutate settings or the default provider (regression guard)

Classification: **regression guard**. This phase adds no kernel boundary. The property holds
at the baseline through four existing host boundaries (`fixtures/capability-inversion.json`
`hostBoundaries`): the agent tool catalog (no registered tool changes settings or the default
provider; `/mcp/uar` exposes 22 tools, none of them settings or provider mutations,
`src/uar/mcp_server.rs`); the admin-authority boundary on mutation routes
(`provider_admin_boundary`, `src/uar/api/providers.rs:59-80`, and `require_admin_key`,
`src/uar/api/settings.rs:446-457`, enforced when `security.settings_mutation_auth_required` is
true, the default, `src/server.rs:2237`); the descriptor approval class (`web_fetch` is
`ExternalMutation`, therefore approval `Required`, `src/uar/runtime/native_skill.rs:76-86`; not
exercised here); and the SSRF guard (`src/uar/tools/fetch_guard.rs:139-162`). The scenario is
mandatory and is expected to PASS on the baseline. It matters in this phase because replica
refresh makes any write to the default row live on every replica, so a kernel path to that row
would now have a wider effect. The SSRF guard alone proves nothing this phase adds, so the
scenario asserts a refusal by the admin-authority boundary on a path the SSRF guard does not
cover, and proves its own sensitivity with mutants.

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 1.5, 5.6, 5.7 |
| Production entry point | four runs from `fixtures/capability-inversion.json` `runs`, each a real run through `POST /api/chat/completion` on replica A whose stub-scripted model emits a `web_fetch` POST (no headers) to A's own default route, plus `tools/list` on `/mcp/uar` and `GET /api/tools`. Replica A has `settings_mutation_auth_required: true` and its admin key in its own environment; no request made by the runner, the stub or the run carries that key |
| Real collaborators | replicas A and B, scratch store, `stub-llm`, the real native tool registry, the SSRF guard, the provider admin boundary, the MCP tool surface |
| Boundary exercised | tool catalog; admin authority on the mutation route; SSRF guard |
| Observable result | **R0 (candidate build):** the tool result is an error naming a blocked address, state unchanged (`unchangedAfterKernelAttempt`), catalog within the frozen allowlist with no name matching `(setting\|provider\|default_model\|default_provider)`. **R1 (build M2, SSRF loopback rule removed, auth required):** the POST reaches the route and is refused with `401` and the body text `Provider administration requires valid admin authority`; state unchanged. This asserts the admin-authority boundary on a path the SSRF guard no longer covers |
| Negative control | three different inputs a wrong implementation accepts, each frozen with its expected verdict. **R2 (build M2, `settings_mutation_auth_required: false`):** the same POST succeeds, `default_id` becomes `p-two` on A and B, and the scenario must FAIL (killed): a deployment that does not require host authority is detected. **R3 (build M1, a settings-mutating tool registered in the MCP surface):** the catalog check must FAIL because `uar_set_default_provider` is outside the frozen allowlist. The mutants are produced as `fixtures/mutants.json` specifies: edit applied to a fresh worktree under `~/.claude/worktrees/` made by `scripts/worktree-new.sh`, built with its own `--target-dir` and a receipt |
| Isolation | shared fixture contract; `native_tools.web_fetch_enabled: true` in the scratch YAML |
| Prerequisites | agent that exposes `web_fetch` (shape UNVERIFIED, recorded at implementation); the two mutant builds (task 5.7) |
| Final local gate | same runner; the scenario verdict is PASS only if R0 and R1 pass and R2 and R3 are killed. A mutant that survives is a FAIL of the guard |
| Evidence | pending |
| Limitations | proves the tool catalog and the admin-authority route boundary, not shell tools, whose governance is approval and sandbox policy. A kernel cannot be shown to lack the admin key except by its absence from the request; the key is never given to any component the scenario drives |

## Scenario matrix rows

Common to all rows: restoration = stop every process started (replicas, store, stub, proxy,
forwarder, reader) and remove the scratch root; production build identity
`server-full,postgres-backend`; fixtures per "Frozen oracle fixtures" and the shared fixture
contract; negative controls are real invalid or faulted inputs with a named observable
rejection, and a mandatory scenario whose control did not run is BLOCKED.

**SCN-RTM-001**
- name: default provider change crosses replicas
- actor: operator using one replica of a load-balanced deployment
- useful outcome: the next turn on any replica uses the new system default
- source requirement: `openspec:realtime-replica-consistency#Each replica refreshes its in-memory provider and settings state from the change feed`
- reason now: item 24, critical 1; the operator named replicas explicitly
- likelihood: UNKNOWN
- mandatory: true
- production entry: `POST /api/uar/providers/p-two/default` on A; `RTM-PING` completion on B; run once on remote Surreal, once on Postgres
- real collaborators: two replicas, scratch remote store, `stub-llm`, external identity proxy
- oracle: observation = B's `default_id` and the completion reply text; expected truth source = `fixtures/replica-scenario.json` (`markers`, `initialDefaultProvider`); comparator = exact equality within 10 s, 5 alternating changes
- negative control: input = unknown provider id `p-does-not-exist` on A; expected rejection = `404`, B's `default_id` and reply unchanged
- nondeterministic: true
- stochastic: samples 5 changes per store mode; grader = both `default_id` and reply marker equal the fixture marker within 10 s; stoppingRule = fixed N=5, all must pass
- fixtures: `fixtures/replica-scenario.json`
- evidence required: both replicas' logs, request/response bodies, marker comparison, timestamps, the stub fingerprint observed
- substituted boundaries: upstream LLM replaced by `stub-llm`; the evidence does not establish real-provider routing
- limitations: agent-explicit model precedence not exercised
- invalidation keys: `src/uar/realtime/**`, `src/llm/registry.rs`, `src/uar/settings/manager.rs`, `src/uar/api/providers.rs`, `src/server.rs`, `fixtures/replica-scenario.json`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#2.2`, `#3.1`-`#3.7`, `#6.1`

**SCN-RTM-002**
- name: default provider change crosses replicas on Postgres
- actor, useful outcome, source requirement, reason now, likelihood, mandatory: as SCN-RTM-001 (mandatory: true)
- production entry: as SCN-RTM-001 against a scratch Postgres with `pgvector`, migrations applied by the product at startup
- real collaborators: two replicas built with `postgres-backend`, scratch Postgres, `stub-llm`, proxy, a read-only `LISTEN uar_entity_change` probe session
- oracle: as SCN-RTM-001; plus observation = both replicas report their notification listener ready in `/readyz`; the probe receives, for the default change, a payload whose topic is `settings` and whose `data` has only a `key` member equal to `llm.default_provider`; expected truth source = `fixtures/topic-routing.json` (`routing`)
- negative control: input = unknown provider id `p-does-not-exist`; expected rejection = `404` and the probe receives no payload for that attempt
- nondeterministic: true
- stochastic: samples 5 changes; grader = as SCN-RTM-001 plus probe payload equality; stoppingRule = fixed N=5, all must pass
- fixtures: `fixtures/replica-scenario.json`, `fixtures/topic-routing.json`
- evidence required: as SCN-RTM-001 plus probe payloads (key names only)
- substituted boundaries: as SCN-RTM-001
- limitations: direct connection only; no pooler
- invalidation keys: as SCN-RTM-001 plus the new migration, `src/uar/realtime/postgres_bus.rs`, `src/uar/persistence/providers/postgres.rs`, `fixtures/topic-routing.json`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#2.1`-`#2.7`, `#3.1`-`#3.7`, `#6.2`

**SCN-RTM-003**
- name: invalidation precedes a current snapshot on the receiving replica
- actor: a console user connected to replica B
- useful outcome: the Providers and Models screens never show the pre-change state after being told of the change
- source requirement: `openspec:realtime-replica-consistency#A snapshot is served only after the replica applied the change it was told about`
- reason now: without the barrier the client and the registry race (design R6)
- likelihood: UNKNOWN
- mandatory: true
- production entry: provider edits and a delete on A; reader and snapshot calls on B, both store modes
- real collaborators: as SCN-RTM-001 plus the forwarder
- oracle: observation = first snapshot after each frame; expected truth source = the edit the runner just made (values drawn from `fixtures/replica-scenario.json`); comparator = field equality
- negative control: input = B's store forwarder cut; expected rejection = `GET /api/live/providers/snapshot` returns `complete:false` with a reason in `fixtures/topic-routing.json` `snapshotIncompleteReasons`
- nondeterministic: true
- stochastic: samples 20 consecutive edits per store mode; grader = the first snapshot after the `providers` frame equals the edit, and a `models` frame follows; stoppingRule = fixed N=20, all must pass, stop at the first stale snapshot (FAIL)
- fixtures: `fixtures/replica-scenario.json`, `fixtures/topic-routing.json`
- evidence required: frame/snapshot pairs with timestamps for every sample
- substituted boundaries: none
- limitations: 20 samples make a systematic ordering defect visible; they do not prove absence of a race
- invalidation keys: `src/server.rs` snapshot route, `src/server/lifecycle_snapshot.rs`, `src/uar/realtime/**`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#3.1`, `#3.4`, `#3.5`, `#5.5`, `#6.3`

**SCN-RTM-004**
- name: settings change crosses replicas, restart-class key stays restart-required
- actor: operator
- useful outcome: a settings edit through one replica is what every replica serves, and a boot-only key is never reported as in effect
- source requirement: `openspec:realtime-replica-consistency#Each replica refreshes its in-memory provider and settings state from the change feed` and `#Replica refresh never applies governance or restart-class keys`
- reason now: the settings cache is per replica and written only locally
- likelihood: UNKNOWN
- mandatory: true
- production entry: `PUT /api/uar/settings/prompt_caching.enabled` on A, read on B; `PUT /api/uar/settings/memory.max_context_tokens` on A, read on B; both store modes
- real collaborators: as SCN-RTM-001
- oracle: observation = B's `GET` of each key (`data`, `application_status`) and B's `settings` frame; expected truth source = `fixtures/replica-scenario.json` `settingsChecks`; comparator = equality within 10 s; `restart_required` for the restart-class key
- negative control: input = the restart-class key edit; expected rejection of "in effect" = B's `application_status` is `restart_required`, not `effective`
- nondeterministic: true
- stochastic: samples 5 alternating edits per key per store mode; grader = value equality and expected status; stoppingRule = fixed N=5, all must pass
- fixtures: `fixtures/replica-scenario.json`
- evidence required: bodies, frames
- substituted boundaries: none
- limitations: governance posture not adopted across replicas; `server.port` not used (inert, `not_applied` in the settings change)
- invalidation keys: `src/uar/settings/manager.rs`, `src/uar/api/settings.rs`, `src/uar/realtime/**`, the settings change's class table
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#3.1`, `#3.2`, `#3.4`, `#6.4`

**SCN-RTM-005**
- name: conversation policy topic works on every store
- actor: a user with the same conversation open in two places
- useful outcome: a model or policy change made in one place appears in the other without reload
- source requirement: `openspec:realtime-replica-consistency#Every public topic has a change source on every supported store`
- reason now: item 14; conversation policy publishes nothing today
- likelihood: UNKNOWN
- mandatory: true
- production entry: `PUT` then `DELETE /api/uar/conversations/{id}/policy` on A; reader on B; remote Surreal, Postgres, and one embedded process
- real collaborators: as SCN-RTM-001; embedded run uses one process and embedded SurrealKV
- oracle: observation = frame on B (`topic`, `requires_snapshot`, body keys) and B's `GET .../policy`; expected truth source = `fixtures/topic-routing.json` (`publicTopics`, `frameAllowedKeys`, `frameForbiddenSubstrings`) and the policy the runner saved; comparator = frame keys subset of allowed, no forbidden substring, `GET` equals saved policy then `204`
- negative control: input = `POST /api/sessions/{id}/agent-config` with `model: "no-slash-model"`; expected rejection = `400` (`discovery.rs:872-880`) and no `conversation_policies` frame within 10 s
- nondeterministic: true
- stochastic: samples 5 save/delete cycles per store mode; grader = frame received within 10 s and B's reads match; stoppingRule = fixed N=5, all must pass
- fixtures: `fixtures/topic-routing.json`, `fixtures/replica-scenario.json`; conversation created by the runner through the real route
- evidence required: frame bodies, GET bodies
- substituted boundaries: none
- limitations: frames are global to connected principals
- invalidation keys: `src/uar/api/discovery.rs`, `src/uar/realtime/**`, the new migration, `src/uar/persistence/providers/*`, `fixtures/topic-routing.json`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#2.1`-`#2.7`, `#3.7`, `#4.1`, `#4.3`, `#4.5`, `#6.5`

**SCN-RTM-006**
- name: team catalog topic works on every store
- actor: console user viewing teams
- useful outcome: a binding installed elsewhere appears without reload
- source requirement: `openspec:realtime-replica-consistency#Every public topic has a change source on every supported store`
- reason now: item 14, required by the team adoption decision
- likelihood: UNKNOWN
- mandatory: true
- production entry: install a deployment binding via `POST /api/v1/collaboration/deployment-bindings` on A; reader on B; both remote stores
- real collaborators: as SCN-RTM-001
- oracle: observation = `team_catalog` frame on B and B's binding list; expected truth source = the installed binding id the runner chose and `fixtures/topic-routing.json`; comparator = present on B within 10 s, frame keys within the allowed set
- negative control: input = a binding referencing an absent package; expected rejection = a 4xx from the install route and no `team_catalog` frame within 10 s
- nondeterministic: true
- stochastic: samples 3 installs per store mode; grader = frame and list within 10 s; stoppingRule = fixed N=3, all must pass
- fixtures: minimal package and binding fixture (UNVERIFIED size; absence is BLOCKED), `fixtures/topic-routing.json`
- evidence required: frame, list bodies, rejection status
- substituted boundaries: none
- limitations: team run state and per-task model provenance are not published by this topic
- invalidation keys: `src/uar/api/collaboration.rs`, `src/uar/compiler/collaboration/storage.rs`, `src/uar/realtime/**`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#2.1`-`#2.7`, `#4.4`, `#5.4`, `#6.6`

**SCN-RTM-007**
- name: a replica that missed changes converges when its feed returns
- actor: operator during a transient store connection fault
- useful outcome: no replica stays stale after reconnecting
- source requirement: `openspec:realtime-replica-consistency#A replica resynchronizes after a feed gap and says so while stale`
- reason now: resync-on-reset is the only recovery path chosen
- likelihood: UNKNOWN
- mandatory: true (the forwarder is now an implementation task, 5.5; if it does not exist at the gate the scenario is BLOCKED, not skipped)
- production entry: cut B's forwarder, change a provider and a setting on A, restore
- real collaborators: as SCN-RTM-001 plus the forwarder
- oracle: observation = B's values and frames after restore; expected truth source = the values the runner wrote on A (from `fixtures/replica-scenario.json`); comparator = equality within 10 s; the reader saw `ready:false` then `ready:true`
- negative control: input = B's forwarder cut; expected rejection = B's `providers` snapshot is `complete:false` with a frozen reason while cut
- nondeterministic: true
- stochastic: samples 3 cut/restore cycles per store mode; grader = convergence within 10 s plus the frame order; stoppingRule = fixed N=3, all must pass
- fixtures: forwarder; `fixtures/replica-scenario.json`
- evidence required: timeline of frames, snapshot bodies
- substituted boundaries: network fault injected by the forwarder
- limitations: not exercised on embedded
- invalidation keys: `src/uar/realtime/**`, `src/server/lifecycle_auto_reconnect.rs`
- restoration: as common
- owner tasks: `uiux-realtime-topics-and-replica-refresh#3.4`, `#3.5`, `#5.5`, `#6.7`

**SCN-RTM-008**
- name: a failed persist publishes nothing and changes no replica
- classification: regression guard (holds at baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52`; listed in `fixtures/replica-scenario.json` `baseline.regressionOnly`; sensitivity proved by mutant M3)
- actor: operator whose store connection fails during a default change
- useful outcome: the system default is never half-changed; no client is told about a change that did not happen
- source requirement: `openspec:realtime-replica-consistency#A failed persist publishes nothing and changes no replica`
- reason now: non-waivable check SAFE-PERSIST-BEFORE-PUBLISH had no scenario that injects a persist failure; replica refresh raises the cost of getting the order wrong
- likelihood: UNKNOWN
- mandatory: true
- production entry: `POST /api/uar/providers/p-two/default` on A with A's forwarder cut; then restore and repeat
- real collaborators: two replicas, scratch store, forwarder, `stub-llm`
- oracle: observation = the route status, `default_id` on A and B, the stored `llm.default_provider`, B's reader frames; expected truth source = `fixtures/replica-scenario.json` (`initialDefaultProvider`, `persistOrdering.faultExpected`); comparator = non-2xx, `default_id` and stored row equal `p-one` on both replicas, zero frames in the window
- negative control: (1) input = the same fault against mutant M3 (`fixtures/mutants.json`, swap before persist); expected rejection = the scenario verdict is FAIL with A `default_id == p-two` and the stored row `p-one`; (2) positive-flow input = restore the forwarder and repeat the request; expected = `200`, replica A flips to `p-two`, a `settings` frame arrives on B within 10 s
- nondeterministic: true
- stochastic: samples 3 fault cycles per store mode, and the M3 run once per store mode; grader = as the oracle; stoppingRule = fixed N=3, all must pass; M3 must fail on its single run
- fixtures: forwarder; scratch YAML with `resilience.request_timeout_ms: 5000`; `fixtures/replica-scenario.json`, `fixtures/mutants.json`
- evidence required: statuses, bodies, frame log with timestamps, the M3 build receipt and its verdict
- substituted boundaries: store connection fault by forwarder
- limitations: default route only
- invalidation keys: `src/uar/api/providers.rs`, `src/uar/settings/manager.rs`, `src/uar/realtime/**`, `fixtures/mutants.json`
- restoration: as common; remove the M3 worktree
- owner tasks: `uiux-realtime-topics-and-replica-refresh#3.4`, `#3.7`, `#5.5`, `#5.7`, `#6.8`

**SCN-RTM-009**
- name: an agent kernel cannot mutate settings or the default provider
- classification: regression guard (the property holds at baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52`, this phase adds no kernel boundary; listed in `baseline.regressionOnly`; sensitivity proved by mutants M1 and M2)
- actor: a model-driven agent run
- useful outcome: only the trusted host changes the system default; a kernel's attempt is refused at a host boundary and changes nothing
- source requirement: `openspec:realtime-replica-consistency#Agent-kernel paths cannot mutate settings or the default provider`
- reason now: non-waivable check SAFE-CAPABILITY-INVERSION had only operator-initiated writes mapped to it, and refresh makes any write to the default row live on every replica
- likelihood: UNKNOWN
- mandatory: true
- production entry: runs R0-R3 of `fixtures/capability-inversion.json` through `POST /api/chat/completion` with the scripted `web_fetch` POST, plus `tools/list` on `/mcp/uar` and `GET /api/tools`
- real collaborators: replicas A and B, scratch store, `stub-llm`, native tool registry, the SSRF guard, the provider admin boundary, the MCP tool surface
- oracle: observation = tool result, HTTP status and body of the refused POST, `default_id` on A and B, stored row, B's frames, tool catalog names; expected truth source = `fixtures/capability-inversion.json` (`runs`, `mcpUarAllowedToolFunctions`, `forbiddenToolNamePattern`, `unchangedAfterKernelAttempt`); comparator = R0 error names a blocked address and state unchanged; R1 status `401` with the frozen body text and state unchanged; catalog within the allowlist and free of the forbidden pattern
- negative control: inputs a wrong implementation accepts, each with frozen expected verdict: R2 (mutant M2, `settings_mutation_auth_required: false`) must make the scenario FAIL because the POST succeeds and the default flips; R3 (mutant M1, a settings-mutating tool registered) must make the catalog check FAIL. Scenario verdict is PASS only if R0 and R1 pass and R2 and R3 are killed
- nondeterministic: true
- stochastic: samples 3 repetitions of R0 and R1, one run each of R2 and R3; grader = as the oracle; stoppingRule = fixed N=3 for R0/R1, all must pass; R2 and R3 must be killed
- fixtures: `fixtures/capability-inversion.json`, `fixtures/mutants.json`; `native_tools.web_fetch_enabled: true`; replica A's admin key only in its own environment (no request carries it)
- evidence required: run trace with the tool call and result, the 401 status and body, state reads, frame log, MCP and `/api/tools` name lists, mutant build receipts and verdicts
- substituted boundaries: model replaced by `stub-llm`; R1-R3 run on mutant builds by design
- limitations: tool catalog and admin-authority route boundary only; shell tools are not exercised; the approval class boundary is cited but not exercised
- invalidation keys: `src/uar/tools/fetch_guard.rs`, `src/uar/tools/web_fetch.rs`, `src/uar/mcp_server.rs`, `src/uar/api/providers.rs`, `src/uar/api/settings.rs`, `src/uar/runtime/native_skill.rs`, `fixtures/capability-inversion.json`, `fixtures/mutants.json`
- restoration: as common; remove the mutant worktrees
- owner tasks: `uiux-realtime-topics-and-replica-refresh#1.5`, `#5.6`, `#5.7`, `#6.9`

## Open decisions and UNVERIFIED items

- The embedded adapter's wait for `entity.snapshot` frames (R8): lead decision.
- Surreal record id string format for the `settings` table (R2): read from the patched SDK in task 2.3.
- Permissions of the persistence user for live queries on the new tables (R3).
- `admission_timeout_ms` default (R6).
- PEM refetch API for the console consumers (R9).
- Whether two proxy-fronted replicas can share one scratch store at the gate.
- Whether the persist call on the default route is bounded by `request_timeout_ms` (SCN-RTM-008).
- Agent configuration that exposes `web_fetch` to a run (SCN-RTM-009).
- Whether the stub receives a bare or provider-qualified model id.
- Dependency: the settings API rejection of `llm.default_provider` (task 1.5), owned by `uiux-settings-truthful-status-and-removals`.
- Single-writer assignment for every non-`uar-state` file listed in the proposal.
