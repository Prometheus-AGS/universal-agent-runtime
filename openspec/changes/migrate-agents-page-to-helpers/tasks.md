## 1. Imports

- [ ] 1.1 `import { optimisticUpsert, optimisticRemove } from "@/lib/realtime/optimistic";`
- [ ] 1.2 Drop `useGraphStore` import if no remaining references.

## 2. Delete local helper

- [ ] 2.1 Remove `patchAgentOptimistic` function definition (lines 79–105).

## 3. AgentMemorySection.save migration

- [ ] 3.1 Replace `patchAgentOptimistic(agent.id, body)` call with inline `optimisticUpsert("Agent", agent.id, body, () => patchAgentApi(agent.id, body))`.

## 4. handleDelete migration

- [ ] 4.1 Wrap `deleteAgent(id)` + `Response.ok` check inside `optimisticRemove`'s `serverCall` closure.
- [ ] 4.2 Preserve `setDeleting`/`setDeleteError` lifecycle.

## 5. Verification

- [ ] 5.1 `pnpm --filter ./frontend test` → 36/36 green.
- [ ] 5.2 `pnpm --filter ./frontend build` clean.
- [ ] 5.3 `git grep -nE "useGraphStore.getState" frontend/src/admin/pages/agents-page.tsx` empty.
