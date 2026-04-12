# AGENTS.md — entity-graph-realtime

Instructions for AI agents using the **entity-graph-realtime** skill package (agentskills.io-style: prompts, agents, references, hooks, nested skills).

## Mission

Integrate **push-based** updates into the **@prometheus-ags/prometheus-entity-management** graph using **`RealtimeManager`** and the official adapter factories under **`src/adapters/`**. UI layers continue to consume data through **hooks**, not raw store subscriptions.

## Read order

1. Consuming repo **`AGENTS.md`** / **`CLAUDE.md`** (pnpm, Components → Hooks → Stores).
2. Skill **`CLAUDE.md`** (adapter boundary, coalescing, no graph writes from adapters).
3. **`references/adapter-catalog.md`** and **`references/coalescing-guide.md`**.

## Operating principles

- Pick the **smallest** adapter that matches the backend; avoid double-wrapping WebSockets.
- Normalize to **`EntityChange`** with stable string **`EntityId`** values and correct **`EntityType`**.
- Every bootstrap path returns **cleanup** (`UnsubscribeFn` from `manager.register`, plus any vendor unsubscribe).
- **Do not** call **`getRealtimeManager`** from arbitrary components; expose a **`useRealtimeBootstrap()`** hook or **`RealtimeProvider`** in the hook/provider layer.
- **`flushInterval: 0`** is opt-in with explicit performance tradeoff documentation.

## Specialist playbooks

| File | When |
|------|------|
| **`agents/adapter-selector.md`** | Choosing WS vs Supabase vs Convex vs GraphQL-WS vs Electric |
| **`agents/channel-configurator.md`** | `ChannelConfig`, filters, `normalize` on `register`, `affectedListKeys` |

## Sub-skill routing (slash commands)

| Command | Use when |
|---------|-----------|
| **`/entity-realtime-setup`** | Greenfield: manager singleton, first adapter, dev logging, teardown |
| **`/entity-realtime-channel`** | Additional entity table, topic, or subscription slice |
| **`/entity-realtime-local-first`** | PGlite + Electric shapes + `SyncAdapter` + optional `useLocalFirst` |

## Quality gate

- Burst traffic produces **bounded** Zustand writes (default **16ms** coalesce) unless `flushInterval: 0` is intentional.
- **`delete`** removes the entity and strips IDs from lists via manager flush.
- Adapters **never** import **`useGraphStore`**; only the manager flush path mutates graph state.
- Typecheck green; manual smoke with **`onChangeReceived`** logging in dev.

## Deliverables

- Adapter factory usage module + bootstrap hook/provider
- **Runbook**: env vars, reconnect/backoff, how to test Electric offline, when to call **`resetRealtimeManager`**

## Lifecycle hooks

**`hooks/hooks.json`**: `on_activate` → **`scripts/state-init.sh`**; orchestrator JSON → **`scripts/detect-orchestrators.sh`**.
