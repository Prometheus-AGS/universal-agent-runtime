---
name: entity-graph-realtime
description: >
  Add realtime synchronization to an existing @prometheus-ags/prometheus-entity-management setup: detect the push source
  (WebSocket, Supabase Realtime, Convex, GraphQL subscriptions, ElectricSQL+PGlite), configure RealtimeManager,
  normalize inbound traffic to ChangeSet/EntityChange, tune the 16ms coalescing window or flushInterval 0,
  and wire channel subscriptions with correct cleanup and list invalidation.
---

# Entity Graph Realtime

## Purpose

Extend graph-backed apps with **live updates** that land in the same Zustand entity graph as REST/GraphQL fetches. **Adapters** speak `RealtimeAdapter`; **`RealtimeManager`** batches writes, coalesces per `(type, id)`, and applies `upsertEntity` / `removeEntity` / `invalidateLists` on flush—never duplicate that logic in UI code.

## When to use

- Live dashboards, collaborative admin grids, or notification-driven list refresh
- Replacing polling with push while keeping **one normalized graph**
- **Local-first** with ElectricSQL shapes + PGlite + graph hydration
- Diagnosing render storms from chatty sources (tune coalescing before disabling it)

## Sub-skills (slash-style routes)

| Route | Focus |
|-------|--------|
| **`/entity-realtime-setup`** | `getRealtimeManager`, `ManagerOptions`, register adapter(s), app lifecycle, cleanup |
| **`/entity-realtime-channel`** | New or updated `ChannelConfig`, `normalize` on `register`, `SubscriptionConfig`, replay semantics |
| **`/entity-realtime-local-first`** | `createElectricAdapter`, `ElectricTableConfig`, `useLocalFirst`, `usePGliteQuery`, sync boundaries |

Each route maps to `prometheus-entity-skills/entity-graph-realtime/skills/<name>/SKILL.md`.

## Workflow

1. **`prompts/specify.md`** — Source, auth, payload samples, ordering, latency, `affectedListKeys`, lifecycle.
2. **`prompts/plan.md`** — Adapter choice, channel topology, flush policy, SSR/client split, failure modes.
3. **`prompts/execute.md`** — Adapter module + bootstrap hook/provider; no direct graph writes from adapters.
4. **`prompts/reflect.md`** — Verify flush behavior, unsubscribe on logout/HMR, typecheck.
5. **`prompts/persist.md`** — State file + runbook for env vars and testing.

## Orchestrators

```bash
bash prometheus-entity-skills/entity-graph-realtime/scripts/detect-orchestrators.sh
```

## Core APIs

| Area | Exports |
|------|---------|
| Manager | `RealtimeManager`, `getRealtimeManager`, `resetRealtimeManager`, `ManagerOptions`, `forceFlush` |
| Types | `RealtimeAdapter`, `SyncAdapter`, `ChangeSet`, `EntityChange`, `ChangeOperation`, `ChannelConfig`, `SubscriptionConfig`, `AdapterStatus` |
| Network adapters | `createWebSocketAdapter`, `createSupabaseRealtimeAdapter`, `createConvexAdapter`, `createGraphQLSubscriptionAdapter` |
| Local-first | `createElectricAdapter`, `useLocalFirst`, `usePGliteQuery`, `ElectricAdapterOptions`, `ElectricTableConfig` |

## References

- **`references/adapter-catalog.md`** — Factory options and registration pattern
- **`references/coalescing-guide.md`** — 16ms window, merge rules, `flushInterval: 0`, `forceFlush`
- **`CLAUDE.md`**, **`AGENTS.md`**

## Architectural note

**Components** still read through **`useEntity`**, **`useEntityList`**, **`useEntityView`**, **`useEntityCRUD`**—not the store. Realtime only **feeds** the graph via the manager; it does not change the Components → Hooks → Stores rule for app code.
