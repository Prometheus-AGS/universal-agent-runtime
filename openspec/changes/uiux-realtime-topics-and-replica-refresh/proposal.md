## Why

Operator decisions 3 and 10 (2026-11-19, `.prometheus/decisions.md`) require that live
updates to open clients work with remote Surreal and Postgres, and that provider and model
changes are correct when several UAR replicas share one remote store. The repo deploys 2
replicas (`k8s/base/uar-deployment.yaml:11`) and 3 in production
(`k8s/overlays/production/patches/uar-deployment-patch.yaml:7`).

Source inspection changed the shape of the problem (analysis delta 1). Push already exists
for remote stores: the frontend uses one SSE connection to `/api/live` plus
`/api/live/{topic}/snapshot` for Postgres and remote Surreal
(`frontend/src/entities/sync.ts:342-344`), the backend serves both routes on every backend
(`src/server.rs:2437-2438`, `:2992-3006`), and the Surreal and Postgres buses feed them
(`src/server.rs:887-892`, `:938-941`). `/api/live` sends invalidations only; every frame is a
`reset` with `requires_snapshot: true` (`src/uar/api/live.rs:98-110`). What is missing is
per topic, and it is backend work:

1. **Conversation policy publishes nothing.** The save and delete handlers make no bus call
   (`src/uar/api/discovery.rs:871-958`, `:1107-1160`), there is no topic, and no Postgres
   trigger exists for `conversation_policies` or `principal_conversation_policies`.
2. **Providers arrive on the wrong topic on Surreal.** Provider rows are `provider.{id}` rows
   of the `settings` table (`src/uar/settings/manager.rs:965-1000`), so Surreal emits them on
   `settings`. Postgres routes `provider.%` keys to `providers`
   (`migrations/20260220000002_notify_entity_change.sql:66-72`). The frontend subscribes to
   `providers` (`frontend/src/lib/realtime/topics.ts:15`). Runtime behavior is UNVERIFIED.
3. **The default-provider change has no topic of its own.** It is the `llm.default_provider`
   settings row, so it invalidates `settings` on both stores, and the providers snapshot
   drops the default (`src/server/lifecycle_snapshot.rs:130-142` keeps `providers` and
   discards `default_id`, which `list_providers` returns, `src/uar/api/providers.rs:131-139`).
4. **The `models` topic never fires.** No writer exists, yet its snapshot replaces the whole
   Model set in the graph (`src/server/lifecycle_snapshot.rs:165-193`).
5. **No public team or binding topic.** `CollaborationState` is host-only
   (`src/uar/realtime/mod.rs:193-198`).
6. **Every replica is stale after another replica's change.** Each replica loads the
   provider registry once (`src/server.rs:1173`, `src/uar/settings/provider_sync.rs:15-29`),
   reads settings from an in-process cache first (`manager.rs:468-489`, written only by that
   process, `:640-664`), and resolves the turn-time default model from the registry
   (`src/uar/runtime/manager.rs:5946`, `:6664`, `:11317`). No process subscribes to a change
   topic. The only consumers of the bus besides the SSE routes are the committed-work wakeups
   (`src/server.rs:2286-2320`).

Without item 6, fixing items 1-5 makes a client on replica B refetch a snapshot that replica
B still computes from stale memory.

## What Changes

Build items 14, 17 and 24 of the analysis; unresolved review finding critical 1.

- **Topics.** Add public topics `conversation_policies` and `team_catalog`. Route provider,
  default-provider and model-key changes identically on both stores through one shared
  function; a provider change also invalidates `models`. A topic declares whether a client
  needs a snapshot.
- **Postgres.** One additive migration: the settings trigger also carries the row key
  (never the value) so the default-provider key can be recognised, and triggers for the new
  topics reuse the existing id-less lifecycle function.
- **Replica refresh.** A `ReplicaState` on every replica subscribes server-side to the
  existing change feed and, on a `providers` or `settings` change, a reset, a lag or startup,
  reloads the settings cache from the store and replaces the provider registry and default
  from the stored rows.
- **Refresh before snapshot.** The bus stamps a per-topic change epoch before it broadcasts.
  The snapshot routes for `providers`, `models` and `settings` wait until the replica has
  applied that epoch, so a client that is told "changed" never reads the old state. A replica
  that cannot refresh answers `complete: false` rather than a stale complete snapshot.
- **Gate.** The final gate builds with `postgres-backend`, runs at least two replicas
  against one remote Surreal store and against one Postgres store, and exercises each change
  through one replica while observing another.

Out of scope: team run state (task attempts and per-task model provenance, owned by
`uiux-team-model-adoption-at-boundary`, which keeps its existing run watch); cross-replica
governance posture (the feed deliberately suppresses `governance.enabled`,
`src/uar/realtime/surreal_bus.rs:201`); credential store revisions across replicas;
frontend state ownership of `Provider` and `Model` records (`uiux-model-identity-graph-ownership`);
the system-default picker (`uiux-system-model-live-default`); fixing the embedded
adapter's wait for `entity.snapshot` frames that the server never sends (reported as an
open question in design.md, not built here); any change to dependency pins.

## Capabilities

### New Capabilities

- `realtime-replica-consistency`: topic coverage per store, routing, invalidation-only frames,
  per-topic snapshot declaration, replica refresh, refresh-before-snapshot, stale reporting,
  Postgres and multi-replica verification.

### Modified Capabilities

- `embedded-sse-sync`: adds a requirement that the embedded invalidation stream serves every
  public topic and honors the per-topic snapshot declaration.

`entity-surreal-live-adapter` is **not** modified: it specifies a browser-side
`LIVE SELECT` adapter inside the entity-management package, which UAR does not use. UAR
delivers invalidations from the server over SSE.

## Impact

Rust (single-writer assignment needed for files outside the `uar-state` list):
`src/uar/realtime/{mod.rs,surreal_bus.rs,postgres_bus.rs,lifecycle.rs}` and a new
`src/uar/realtime/replica_refresh.rs` (uar-state); `src/server.rs`,
`src/server/lifecycle_snapshot.rs`, `src/uar/api/live.rs`, `src/uar/settings/manager.rs`,
`src/uar/settings/provider_sync.rs`, `src/llm/registry.rs`, `src/uar/api/discovery.rs`,
`src/uar/api/administration_capabilities.rs` (other owners). New migration under
`migrations/` (Postgres). Surreal needs no migration: the bus already defines each topic
table idempotently (`src/uar/realtime/surreal_bus.rs:164-166`).

Frontend (owner `uar-console`): `frontend/src/lib/realtime/topics.ts`,
`frontend/src/entities/sync.ts`, `frontend/src/lib/realtime/uar-sse-adapter.ts`, and the
session-configuration and teams platform domains that react to the new resets.

Interface effects: two new public wire topics (`conversation_policies`, `team_catalog`);
`requires_snapshot: false` on those frames (additive field value); an `is_default` marker on
provider snapshot records (additive); `complete: false` snapshot reasons
`replica_refresh_pending`, `replica_refresh_failed`, `subscription_unavailable`
(new values of an existing field).

Frozen oracle fixtures (written now, not editable by the implementer): `fixtures/topic-routing.json`,
`fixtures/replica-scenario.json`, `fixtures/capability-inversion.json`.

Added after review round 1: replica refresh keeps applying the stored `llm.default_provider`
row on every replica, which is safe only because the settings API rejects writes to that key
(owned by `uiux-settings-truthful-status-and-removals`; task 1.5 makes the dependency
explicit). SCN-RTM-008 (a failed persist publishes nothing) and SCN-RTM-009 (an agent kernel
cannot mutate settings or the default provider) cover the persist-before-publish and
capability-inversion safety checks. Both are regression guards: the properties already hold at
baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52` and this phase adds no kernel boundary, so
they are mandatory, expected to pass on the baseline, and proved sensitive by three mutant
builds frozen in `fixtures/mutants.json`.

Cross-change dependencies: `uiux-model-identity-graph-ownership` must land with or before
the `models` fan-out is enabled (the `models` snapshot becomes a live writer of Model
records); `uiux-config-boot-safety-and-live-reload` calls `ReplicaState::refresh_now`
through a trait defined in that change.

## Related open changes

- `backend-extend-realtime-topics` — **continue, then archive.** It added topics 8-10 of the
  enum; its open tasks 3.1 and 3.2 are deferred non-database channels that this change
  does not build. Record the deferral when archiving.
- `surreal-live-query-bus` — **archive.** The bus exists and has been rebuilt twice since;
  its open tasks (3.2 gauge, 4.2 reconnect test, 5.1 docs) are superseded by this change's
  gate and by the existing lifecycle-owned bus.
- `configure-entity-engine-and-realtime-bridge` — **continue.** Five open frontend tasks
  about the SPA bridge; this change only adds two topic entries to the map it owns.
- `embedded-conversation-policy-surface` — **archive** (all tasks done); it introduced the
  conversation policy routes this change now publishes.
- `postgres18-flint-forge-support` — **continue.** Three open tasks about the Postgres
  target; this change's Postgres gate must use the Postgres version that change settles.
- `uiux-system-model-live-default`, `uiux-team-model-adoption-at-boundary`,
  `uiux-model-identity-graph-ownership`, `uiux-config-boot-safety-and-live-reload` —
  **continue, coordinate** as named under Impact.
