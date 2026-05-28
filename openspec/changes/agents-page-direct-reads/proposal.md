## Why

`agents-page.tsx` reads its list of agents from `useAgentsAdmin()` → `useAgentsAdminStore`. With the entity graph already hydrated by `loadAgentsIntoGraph()` on page mount and kept fresh by the SSE realtime adapter, the store is now an indirection we can retire by switching reads to `useAgents()`.

This change swaps **reads only**; mutations stay on the legacy hook so the page is fully functional after this step and the mutation surgery (next change) can proceed in isolation.

## What Changes

- Replace `const { agents, loading, error, load } = useAgentsAdmin();` with:
  - `const agentsView = useAgents();`
  - `const agents = agentsView.items;`
  - `const loading = agents.length === 0;`
  - `const [error, setError] = useState<string | null>(null);`
  - `const load = () => loadAgentsIntoGraph();`
- `AgentMemorySection` continues to grab `patchAgent` from the store (untouched in this PR).

## Acceptance

- Page renders pixel-equivalent to today.
- Agent list reflects SSE-delivered updates without manual refresh.
- `pnpm --filter ./frontend build` clean.
