---
name: entity-realtime-setup
description: >
  Greenfield RealtimeManager wiring: getRealtimeManager options (flushInterval, onStatusChange, onChangeReceived),
  instantiate the chosen adapter factory, register ChannelConfig entries, store UnsubscribeFn for cleanup,
  and document SSR/client-only boundaries.
---

# `/entity-realtime-setup` — Manager and first adapter

## When to use

- First introduction of push updates into an app already using @prometheus-ags/prometheus-entity-management hooks
- You need a **single place** that owns connection lifecycle (login → connect, logout → disconnect)

## Steps

1. **Choose adapter** using **`agents/adapter-selector.md`** and **`references/adapter-catalog.md`**.
2. **Manager options**:
   - Default **`flushInterval: 16`**; use **`0`** only with written justification
   - **`onStatusChange(adapter, status)`** for banners or dev indicators
   - **`onChangeReceived`** for debugging pre-coalesce volume
3. **Bootstrap module** (e.g. `src/realtime/bootstrap.ts` or `hooks/useRealtimeRegistry.ts`):
   - `const manager = getRealtimeManager({ ... })` — options apply on **first** call only
   - `const unregister = manager.register(adapter, channelConfigs, normalizeFn?)`
   - Return **`unregister`** for `useEffect` cleanup
4. **Never** register the same adapter twice without **`unregister`** (HMR, strict mode double-mount).
5. **`resetRealtimeManager()`** — tests or full sign-out reset only; understand it clears **all** adapters.

## SSR / Next.js

- WebSocket and most subscriptions are **client-only**: dynamic import or `useEffect` guard with `typeof window !== "undefined"`.
- Do not instantiate adapters in Server Components.

## Verification

- Simulate burst events; confirm **one** flush per frame (default) via logging or React Profiler
- **`delete`** removes entity and list id entries

## Parent skill

**`prometheus-entity-skills/entity-graph-realtime/SKILL.md`**, **`../CLAUDE.md`**
