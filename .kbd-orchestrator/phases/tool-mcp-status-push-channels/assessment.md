# Assessment — `tool-mcp-status-push-channels`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `settings-store-retirement` (100%, reflect_complete)

---

## 1. Phase goal

Wire two push channels that don't fit the existing SurrealDB-live-query pattern, then migrate the last bridged entity (`Tool`) and a non-realtime polling consumer (`McpStatus`) to the direct pattern. Finishing this phase deletes `use-graph-bridge.ts` entirely.

The two channels differ from the existing 10 topics because they're not naturally backed by SurrealDB tables:

- **Tools** — discovered dynamically from MCP servers via the `McpRegistry`. The list changes when servers connect/disconnect or tools are added/removed. It's an in-memory derivation, not a database table.
- **McpStatus** — derived from periodic health probes against MCP endpoints. Today the frontend polls every 30 s. The state is server-process-local.

Solution: extend the realtime bus to support **manual-publish topics** alongside the existing **Surreal-live-driven topics**. Backend code paths that mutate Tool or McpStatus state call `bus.publish(topic, payload)` directly; the SSE endpoint and frontend adapter handle both kinds identically.

---

## 2. Current state inventory

### 2.1 Backend

| Component | Path | State |
|---|---|---|
| `EntityTopic` enum | `src/uar/realtime/mod.rs` | 10 topics; Tools/McpStatus absent |
| `LiveQueryBus` | `src/uar/realtime/surreal_bus.rs` | hardcoded to start one `.live()` stream per topic from a Surreal `db` handle |
| `LiveEvent` payload | `src/uar/realtime/mod.rs` | `{ table, action, row }` shape |
| MCP registry | (TBC at execute time) | source of Tool state |
| MCP health probes | (TBC at execute time) | source of McpStatus state |
| SSE endpoint | `src/uar/api/live.rs` | works per topic; ready to expose new topics |

### 2.2 Frontend

| Component | Path | State |
|---|---|---|
| `UAR_TOPICS` | `frontend/src/lib/realtime/topics.ts` | 10 topics enumerated |
| Tool migration | `frontend/src/admin/pages/tools-page.tsx` + `hooks/use-tools-discovery.ts` + `stores/tools-discovery-store.ts` | bridged |
| McpStatus | `frontend/src/admin/McpHealthPage.tsx` + `hooks/use-mcp-health.ts` + `stores/mcp-health-store.ts` | polling (30 s) |
| `useGraphBridge` | `frontend/src/lib/realtime/use-graph-bridge.ts` | one consumer (`use-tools-discovery`); marked `@deprecated` |

### 2.3 Bridge consumers

`git grep useGraphBridge frontend/` shows exactly one consumer: `hooks/use-tools-discovery.ts`. After this phase, that consumer migrates direct, the bridge file is deleted, and the audit's "Historical: bridge pattern" appendix gets a "permanently retired" note.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|---|---|
| D1 | `EntityTopic` enum extends with `Tools` + `McpStatus` | grep + cargo build |
| D2 | `LiveQueryBus` supports a `publish(topic, event)` API for manual-emit topics | unit-style test or doctest |
| D3 | MCP registry calls `bus.publish(Tools, …)` on tool add/remove | code search; manual smoke |
| D4 | MCP health prober calls `bus.publish(McpStatus, …)` on state change | code search |
| D5 | SSE endpoint exposes `/api/live/tools` and `/api/live/mcp_status` | curl smoke |
| D6 | `UAR_TOPICS` frontend list adds the two topics | grep |
| D7 | Tool migration: `useTools()` reads from graph; mutations direct; `tools-discovery-store` retired | grep |
| D8 | McpStatus migration: `useMcpHealth()` reads from graph; polling loop replaced by SSE; `mcp-health-store` retired | grep |
| D9 | `useGraphBridge` deleted; `git grep useGraphBridge frontend/` empty | grep |
| D10 | `pnpm --filter ./frontend test ≥ 40/40` after every change | output |
| D11 | `cargo build --features metal,memory-palace,wasm-runtime` clean | output |
| D12 | Audit doc: Tool + McpStatus rows flipped to `direct`; bridge appendix marked "Permanently retired" | file diff |

---

## 4. Gap analysis

### 4.1 Bus API extension

`LiveQueryBus` today only supports Surreal-live-driven topics. The new design:

```rust
pub enum TopicSource {
    SurrealLive,   // existing behavior
    ManualPublish, // new: publisher-driven only
}

impl EntityTopic {
    pub fn source(self) -> TopicSource {
        match self {
            EntityTopic::Tools | EntityTopic::McpStatus => TopicSource::ManualPublish,
            _ => TopicSource::SurrealLive,
        }
    }
}

impl LiveQueryBus {
    pub fn publish(&self, topic: EntityTopic, event: LiveEvent) -> Result<()> {
        if let Some(tx) = self.senders.get(&topic) {
            let _ = tx.send(event); // ignore "no receivers"
        }
        Ok(())
    }
    // start() skips Surreal subscription for ManualPublish topics.
}
```

The constructor logic in `start()` becomes a small match on `topic.source()`.

### 4.2 Where to publish from

- **Tools** — wherever `McpRegistry` adds/removes a tool. The list is currently rebuilt on each `/api/tools` request; we need an explicit mutation point. Likely `src/uar/mcp/registry.rs` (TBC at execute).
- **McpStatus** — wherever the health probe runs. Today probably a tokio interval task; we add a `bus.publish` on state-transition.

### 4.3 Idempotency

`Tools` and `McpStatus` events are derived, not row-level. Frontend consumers must treat them as snapshot replacements, not deltas. The `LiveEvent.action` field should always be `update` (or a new `replace` variant) for manual-publish topics. **Recommendation:** add `LiveAction::Snapshot` so consumers know to wipe + re-upsert the slice.

### 4.4 Frontend SSE adapter

`uar-sse-adapter.ts` currently maps `create|update|delete` → entity graph ops. For snapshot events it needs to either:
- map `snapshot` → bulk-replace the topic's slice in the graph, OR
- map snapshots into N individual upserts (lossier; orphans persist)

**Recommendation:** add explicit snapshot handling. Add `clearType(type)` to the graph store call (or use a generation counter).

### 4.5 McpStatus polling-to-push transition

Today the 30 s poll fires from the frontend. After push, the backend pushes whenever state changes; the frontend can drop the polling shim but should retain a fallback 60 s poll for liveness in case the SSE connection drops.

### 4.6 Risk areas

- **Backend testability.** Adding a unit test for `bus.publish` is straightforward but the actual integration (registry → bus → SSE) is harder to test autonomously without a live server.
- **MCP registry refactor scope.** If the registry doesn't have a clear mutation API today, adding one is its own mini-phase.
- **Bridge deletion is final.** Once `use-graph-bridge.ts` is deleted, any future bridged consumer would have to reinvent it. Capture this in the audit's historical appendix.

---

## 5. Sequencing recommendation

7 changes ordered:

1. **`extend-entity-topic-enum`** — add `Tools` + `McpStatus` to `EntityTopic`. Backend only. Cargo build.
2. **`add-manual-publish-topic-source`** — `TopicSource` enum + `bus.publish` API + constructor branching. Backend + small docstring.
3. **`add-snapshot-live-action`** — `LiveAction::Snapshot` variant; SSE serialiser; frontend adapter `snapshot` handler that bulk-replaces the type slice.
4. **`wire-mcp-registry-to-bus`** — find the registry mutation points; emit `bus.publish(Tools, snapshot)`.
5. **`wire-mcp-health-to-bus`** — find the health probe; emit `bus.publish(McpStatus, snapshot)` on transitions.
6. **`migrate-tool-page-direct`** — frontend: `useTools` reads from graph, mutations direct, retire `tools-discovery-store` + admin hook.
7. **`migrate-mcp-health-page-direct-and-delete-bridge`** — frontend: `useMcpHealth` reads from graph + 60 s fallback poll; retire `mcp-health-store`; **delete `use-graph-bridge.ts`**; flip audit; mark bridge appendix "Permanently retired".

Each change runs tests + build gate.

---

## 6. Open questions

1. **Snapshot semantics.** Add `LiveAction::Snapshot` variant (Recommended) or shoehorn into `update` with a magic id like `__snapshot__`? Snapshot variant is cleaner.
2. **McpStatus fallback poll.** Keep a 60 s frontend fallback poll for connection-drop resilience? Recommended yes.
3. **Backend integration test.** Skip (Recommended — pnpm tests cover frontend; cargo build covers backend compile; manual smoke owed) or wire a basic `cargo test` for `LiveQueryBus::publish`? Skip is cheaper.
4. **Registry mutation API.** If the registry doesn't have a clear add/remove API, defer Tool push wiring to a separate phase and complete only McpStatus + bridge deletion this phase? Inspect at execute time.

---

## 7. Progress signal

Assessment complete. Recommended defaults are reasonable. Next: `/kbd-plan tool-mcp-status-push-channels`.

**Note on context budget:** this phase has substantial Rust backend work (~5 Rust files touched) plus the bridge-finalization frontend work. Consider pausing for review after the backend changes (#1–#5) before doing the frontend migration. Or break into two phases: `add-push-channels-backend` + `migrate-tools-and-mcp-frontend`.
