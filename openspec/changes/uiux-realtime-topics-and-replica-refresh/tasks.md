## 1. Preconditions and ownership

- [ ] 1.1 Obtain a single-writer assignment from the lead for the files outside the `uar-state` list: `src/server.rs`, `src/server/lifecycle_snapshot.rs`, `src/uar/api/live.rs`, `src/uar/settings/manager.rs`, `src/uar/settings/provider_sync.rs`, `src/llm/registry.rs`, `src/uar/api/discovery.rs`, `src/uar/api/administration_capabilities.rs`, and the frontend files named in section 4. `src/uar/realtime/**`, `src/uar/persistence/**` and the new migration are `uar-state`.
- [ ] 1.2 Agree with `uiux-config-boot-safety-and-live-reload` the `ProviderRefresh` trait (`refresh_now() -> Result<()>`), which lives in `src/uar/settings/` and is implemented by `ReplicaState`.
- [ ] 1.3 Agree with `uiux-system-model-live-default` who carries the default provider id to the client. This change adds `is_default` to provider snapshot records (task 3.6); if that change chooses another carrier, drop 3.6 and keep the invalidation.
- [ ] 1.4 Confirm with `uiux-model-identity-graph-ownership` the point at which the `models` fan-out (task 2.2) may be enabled.
- [ ] 1.5 Depend on `uiux-settings-truthful-status-and-removals` rejecting settings-API writes to `llm.default_provider` (status code and error body pointing to `POST /api/uar/providers/{id}/default` are named there). Replica refresh applies the stored `llm.default_provider` row on every replica (design R6 step 2); that is safe only while the live default route and provider deletion are the row's sole writers. Ship the two changes together or leave the default step of refresh disabled.
- [ ] 1.6 Treat `fixtures/*.json` in this change as frozen oracle inputs: the boundary runner reads them, the implementer does not edit them, and a needed change is a revision of this change.

## 2. Topics and change sources (build item 14)

- [ ] 2.1 In `src/uar/realtime/mod.rs` add `EntityTopic::ConversationPolicies` (wire `conversation_policies`) and `EntityTopic::TeamCatalog` (wire `team_catalog`). Update `ALL`, `as_str`, `from_str`, `is_public` (both public) and the host-only list. Replace `table()` with `tables()` per design R3 and add `requires_snapshot()` (false for the two new topics). Keep memory handled by its own bus.
- [ ] 2.2 Add `route(&LiveEvent) -> &'static [EntityTopic]` per design R2 and call it in both decoders. A provider change reaches `models` as well; keep the `models` fan-out behind a single constant so task 4.6 can enable it with the identity change.
- [ ] 2.3 Read the patched SDK (`vendor/patches/surrealdb-lifecycle/surrealdb-4.0.0`) to establish the exact shape of the record id and data in a `settings` live notification, including delete. Use `data.key` first and the id key part only as a fallback; record the finding in the design's UNVERIFIED list.
- [ ] 2.4 Surreal bus (`surreal_bus.rs`): open one live stream per table of a topic, merge them in the topic supervisor, mark a topic ready only when all its streams are registered, and close all of them on exit. `Providers` and `Models` use the `settings` table, so their readiness follows it. Keep the idempotent `DEFINE TABLE IF NOT EXISTS` per table and keep dropping `governance.enabled` events (`surreal_bus.rs:201`).
- [ ] 2.5 Postgres bus (`postgres_bus.rs`): apply `route` to decoded notifications and broadcast on every returned topic; keep the `governance.enabled` drop.
- [ ] 2.6 Make `live.rs` (`sse_event`, `gap_event`) and `sync_stream_handler` read `requires_snapshot()` instead of the constant. Add snapshot arms in `lifecycle_snapshot.rs` for the two new topics answering `200 {complete:false, reason:"owner_scoped_refetch"}`. Keep the 404 for remote URLs on `/api/uar/sync/stream`.
- [ ] 2.7 Author the additive Postgres migration (proposed `migrations/20261120000000_realtime_replica_topics.sql`): replace `uar_notify_settings_change()` with a version that adds `data.key` (key only); create triggers on `conversation_policies` and `principal_conversation_policies` with `uar_notify_lifecycle_work('conversation_policies')`, and on `uar_collaboration_state` with `uar_notify_lifecycle_work('team_catalog')` under a new trigger name; guard each with `to_regclass`. Do not execute it; the product applies it at startup on the scratch database at the boundary.

## 3. Replica refresh (build items 17 and 24)

- [ ] 3.1 Add the per-topic change epoch to `BusLifecycle` (`stamp(topic)`), call it in both buses immediately before every `sender.send` of a decoded event, and expose `RealtimeBus::observed_epoch(topic)` with a default of `0` for unmanaged buses.
- [ ] 3.2 Add `SettingsManager::reload_cache_from_store()` per design R6 step 1: list all rows, rebuild cache and effective maps off to the side, preserve metadata for unchanged rows, mark changed rows `Api` with `is_drift: false` when `updated_at` is set, remove vanished rows, skip `governance.*`, update the effective map only for keys the settings change's class table marks `live` or `next_turn` (never `restart` or `inert`; until that table exists, keep the current three-way `setting_apply_mode`), swap under the write lock.
- [ ] 3.3 Add `ProviderRegistry::apply_persisted(configs, default_id)` per design R6 step 2: enrichment and pricing validation as in `register_custom_provider`, `default_model` taken from the row, in-memory `api_key` kept only when the row has none, removal of providers absent from the store, default from `llm.default_provider` when that provider exists. Take it under `administration_guard`. Do not change `register_custom_provider` behavior for its existing callers.
- [ ] 3.4 Implement `ReplicaState` in `src/uar/realtime/replica_refresh.rs`: subscribe before reading; triggers on startup, `Providers`/`Settings`/`Models` events, `Reset` with `ready: true`, `Lagged`; `Reset` with `ready: false` marks the feed unavailable; single-flight loop with a `dirty` flag; epoch captured before the read and recorded as applied after it; failure keeps the previous state, records `Stale`, retries with capped backoff, logs `realtime.replica.refresh_failed` once per transition. Expose `refresh_now()`.
- [ ] 3.5 Implement `await_current(topics, deadline)` and call it from the snapshot route for `providers`, `models` and `settings` with the deadline `config.resilience.admission_timeout_ms`. Outcomes: current, `replica_refresh_pending`, `replica_refresh_failed`, `subscription_unavailable` (design R6). Other topics are unchanged.
- [ ] 3.6 In `LifecycleSnapshotService::providers` keep `providers` records and add `is_default` from `default_id` (subject to task 1.3).
- [ ] 3.7 Wire `ReplicaState` in `start_server` after settings hydration, under the supervisor, next to the committed-work subscriptions; clear `AppState.agent_sessions` on `ConversationPolicies` events, resets and lag (`src/lib.rs:154`); implement the `ProviderRefresh` trait (task 1.2).

## 4. Frontend consumers (owner `uar-console`; no frontend code is written in this change's spec stage)

- [ ] 4.1 Add `conversation_policies` and `team_catalog` to `frontend/src/lib/realtime/topics.ts` with a snapshotless flag and an `onReset` hook per topic.
- [ ] 4.2 On a `providers` reset also refresh `ConfiguredProvider` and `ConfiguredModel` (`frontend/src/platform/entities/session-configuration/registration.ts`), which use their own transports.
- [ ] 4.3 On a `conversation_policies` reset refetch every loaded `AgentSession` through its existing `get` transport, from the platform domain only (no component, feature hook or store imports the graph or transport).
- [ ] 4.4 On a `team_catalog` reset refetch loaded team packages and bindings through the teams domain loaders in `frontend/src/platform/entities/teams/`.
- [ ] 4.5 In `frontend/src/entities/sync.ts` treat a reset with `requires_snapshot: false` as an invalidation, not an error (today `:262`), and invoke the same topic reset hook as the multiplexed adapter.
- [ ] 4.6 Enable the `models` fan-out only together with or after `uiux-model-identity-graph-ownership`; confirm the `Model` entity accepts the snapshot record shape.

## 5. Documentation and handoffs

- [ ] 5.1 Update `docs/realtime.md` with the topic table (R1), the frame contract (R4), the replica refresh and barrier behavior (R6), and the limitation that `LISTEN` needs a direct connection.
- [ ] 5.2 Add a deployment note to `docs/DEPLOYMENT.md` that replicas share one store and rely on the change feed; no manifest change (`k8s/base/uar-deployment.yaml` already declares 2 replicas, production 3).
- [ ] 5.3 Record the interface notes for the console owner (`is_default`, snapshotless topics, new `complete:false` reasons) as a cross-domain handoff.
- [ ] 5.7 Provide the mutant build support for SCN-RTM-008 and SCN-RTM-009: a script (test support under `scripts/behaviors/support/`, authored now, executed only at the boundary) that applies each edit in `fixtures/mutants.json` (M1, M2, M3) to a fresh worktree created with `scripts/worktree-new.sh` under `~/.claude/worktrees/`, builds it with its own `--target-dir` and the candidate's feature set, writes a build receipt, and removes the worktree afterwards. The edits are frozen; the implementer does not alter them.
- [ ] 5.4 Record for the owner of `uiux-team-model-adoption-at-boundary` that `team_catalog` covers packages and bindings only, not run state.
- [ ] 5.5 Provide the TCP forwarder fixture the gate needs as its fault hook: a disposable, runner-owned forwarder in front of each replica's store connection that can be cut and restored on command (test support under `scripts/behaviors/support/`, authored now, executed only at the boundary). SCN-RTM-003, -007 and -008 are mandatory and are BLOCKED without it.
- [ ] 5.6 Provide the fixture support for SCN-RTM-009: the scratch YAML with `native_tools.web_fetch_enabled: true` and `security.settings_mutation_auth_required: true` on replica A with its admin key only in A's own environment, an agent that exposes `web_fetch` to the run (read the agent configuration shape first and record it), and the `stub-llm` fixture file generated from `fixtures/capability-inversion.json` with `${REPLICA_A_PORT}` substituted. No request made by the runner, the stub or the run carries the admin key.

## 6. Boundary acceptance (runs only at the phase boundary)

These tasks describe what the final integration gate exercises. They must not be executed
before the whole phase's implementation is complete. Build once, serialized:
`cargo build --locked --no-default-features --features server-full,postgres-backend`; the three mutant builds and the baseline build are separate builds with their own `--target-dir` and receipts (task 5.7). Each
scenario uses two production-binary replicas, one scratch remote store, `stub-llm`, the
existing external identity proxy and no credential of any kind in a request. Replica count
is at least two in every cross-replica scenario. Ports never include `1906` or `28000`.

- [ ] 6.1 SCN-RTM-001: default provider change through replica A observed on replica B (registry default and turn-time marker), remote Surreal; run the unknown-provider control.
- [ ] 6.2 SCN-RTM-002: the same on Postgres, including readiness of the `LISTEN` connection and the key-carrying payload observed by a read-only probe session.
- [ ] 6.3 SCN-RTM-003: 20 consecutive provider edits plus a delete, frame-then-snapshot on replica B, both store modes; run the cut-forwarder control.
- [ ] 6.4 SCN-RTM-004: settings change crossing replicas, both store modes; the restart-class control uses `memory.max_context_tokens` and expects `restart_required` on replica B.
- [ ] 6.5 SCN-RTM-005: conversation policy topic on remote Surreal, Postgres and one embedded process, including delete visibility on replica B; negative control is the invalid agent-config body (`400`, no frame).
- [ ] 6.6 SCN-RTM-006: team catalog topic on both remote stores; negative control is the binding that references an absent package (4xx, no frame). Missing binding fixture is BLOCKED.
- [ ] 6.7 SCN-RTM-007: feed gap and convergence, 3 cut/restore cycles per store mode. A missing forwarder fixture is BLOCKED, never passed.
- [ ] 6.8 SCN-RTM-008 (regression guard, mandatory): default route on replica A with its store forwarder cut: non-2xx, no frame on replica B, default unchanged on both replicas and in the stored row; positive-flow control (restore, repeat, A flips to `p-two`, a `settings` frame arrives on B); run mutant M3 once per store mode and require the scenario to FAIL on it (A `default_id == p-two`, stored row `p-one`). A surviving mutant fails the guard.
- [ ] 6.9 SCN-RTM-009 (regression guard, mandatory): runs R0 and R1 pass (R0 candidate refuses at the SSRF guard; R1 on mutant M2 with admin auth required is refused with 401 and `Provider administration requires valid admin authority`), runs R2 (M2 with admin auth off) and R3 (M1 with a settings-mutating MCP tool) must FAIL; `/mcp/uar` tool list within the frozen 22 names; no request carries the admin key. The scenario must PASS on the baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52`.
- [ ] 6.10 Record, per scenario, exact argv, replica ports, store identity and version, build identity (`postgres-backend` present), timelines and bodies in the phase evidence store; confirm every process started was stopped and the scratch root removed.
