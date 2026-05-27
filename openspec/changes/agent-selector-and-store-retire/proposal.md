## Why

This is the keystone change of the phase. Two problems get fixed in one PR:

1. **Latent staleness bug in `AgentSelector`.** The chat sidebar selector runs its own `fetchAgentsList()` on mount and stashes the result in a local `useState`. There is **no** realtime feed into that cache today — the dropdown silently goes stale until the user reloads. Even when the admin store was bridged, the selector was blind. Migrating to `useAgents()` gives the selector cross-tab freshness for free.

2. **Store retirement.** With `agents-page.tsx` (changes 2–3) and `AgentSelector` (this change) both reading from the graph and mutating via direct service calls, `useAgentsAdmin` and `useAgentsAdminStore` are orphaned. Delete them and update the audit doc.

## What Changes

### `features/chat/agent-selector.tsx`

- Remove local `useState<AgentWithType[]>` agents cache.
- Remove the `fetchAgentsList()` `useEffect`.
- `const agentsView = useAgents();`
- Hydrate the graph on mount (cheap idempotent): `useEffect(() => { void loadAgentsIntoGraph(); }, []);`
- Preserve the `runtime` vs `federated` `_type` tag — verify whether any consumer reads it; if so, derive locally rather than persist on the graph.
- **Render-derived `AgentConfig`**: `const currentAgent = agents.find(a => a.id === selectedId); const config = useMemo(() => currentAgent ? extractAgentConfig(currentAgent) : null, [currentAgent]); useEffect(() => onAgentConfigChange?.(config), [config, onAgentConfigChange]);`
- This means context auto-updates when the underlying agent mutates via SSE.

### Retire admin hook + store

- `rm frontend/src/hooks/use-agents-admin.ts`.
- `rm frontend/src/stores/agents-admin-store.ts`.
- `git grep -nE "useAgentsAdmin|agents-admin-store" frontend/src` must return empty.

### Docs

- Flip the `Agent` row in `docs/migration-stale-data-audit.md` from `bridged` → `direct`.

## Acceptance

- AgentSelector dropdown reflects admin-page edits in another tab ≤200 ms (no reload).
- AgentSelector dropdown removes a deleted agent in another tab without reload.
- `useAgentConfig` (consumed by `enhanced-thread.tsx` and the chat hot path) still receives correct values; switching agents still re-tunes the chat runtime.
- `git grep` clean.
- Net frontend LOC negative.
