## Why

`agents-page.tsx` has two inline optimistic patches: a local `patchAgentOptimistic` helper and the snapshot/remove block in `handleDelete`. With the shared module shipped, delete the local helper and inline the canonical helpers at each call site.

## What Changes

- Delete the local `patchAgentOptimistic` function (lines 79–105). Update `AgentMemorySection.save` to call `optimisticUpsert("Agent", agent.id, body, () => patchAgentApi(agent.id, body))` directly with a try/catch for the local `setError`.
- Rewrite `handleDelete` to use `optimisticRemove("Agent", id, async () => { const res = await deleteAgent(id); if (!res.ok) throw new Error((await res.text()) || res.status); })`. The `Response.ok` check moves inside the `serverCall` closure so the helper sees the throw and rolls back automatically.
- Drop the `useGraphStore` import if nothing else references it.

## Acceptance

- Page renders identically.
- Memory toggle still flips optimistically; rollback still works on rejection.
- Delete flow still works; forced rejection re-upserts the snapshot.
- `pnpm --filter ./frontend test` → 36/36 green.
- `pnpm --filter ./frontend build` clean.
- `git grep -nE "useGraphStore.getState" frontend/src/admin/pages/agents-page.tsx` → empty.
