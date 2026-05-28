## 1. Imports

- [ ] 1.1 `import { useAgents } from "@/entities/hooks/use-agents";`
- [ ] 1.2 `loadAgentsIntoGraph` import already present.

## 2. Read swap

- [ ] 2.1 Replace the destructure on line 201 with `useAgents()` consumption.
- [ ] 2.2 Add local `error` state + `setError`.
- [ ] 2.3 `load = () => loadAgentsIntoGraph();`

## 3. Mutations stay on legacy

- [ ] 3.1 `AgentMemorySection.patchAgent` still reads from `useAgentsAdminStore`.
- [ ] 3.2 `handleDelete` still calls `deleteAgent` service (already direct).

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend build` clean.
- [ ] 4.2 Manual: page renders identically — pending browser smoke.
