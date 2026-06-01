## 1. AgentSelector migration

- [ ] 1.1 Remove local `useState<AgentWithType[]>` + `fetchAgentsList()` effect.
- [ ] 1.2 `import { useAgents } from "@/entities/hooks/use-agents";`
- [ ] 1.3 `import { loadAgentsIntoGraph } from "@/entities/fetchers/agents";`
- [ ] 1.4 Hydrate on mount: `useEffect(() => { void loadAgentsIntoGraph(); }, []);`
- [ ] 1.5 Render-derived `AgentConfig` via `useMemo` + `useEffect` → `onAgentConfigChange`.
- [ ] 1.6 Audit `_type: "runtime" | "federated"` usage; derive locally if any consumer reads it.

## 2. Delete files

- [ ] 2.1 `rm frontend/src/hooks/use-agents-admin.ts`.
- [ ] 2.2 `rm frontend/src/stores/agents-admin-store.ts`.

## 3. Sweep

- [ ] 3.1 `git grep -nE "useAgentsAdmin|agents-admin-store" frontend/src` → empty.

## 4. Audit doc

- [ ] 4.1 Flip `Agent` row in `docs/migration-stale-data-audit.md` from `bridged` → `direct`.

## 5. Verification

- [ ] 5.1 `pnpm --filter ./frontend build` clean.
- [ ] 5.2 Two-tab smoke: admin edit propagates to selector — pending.
- [ ] 5.3 Chat smoke: switching agent re-tunes the chat hot path — pending.
- [ ] 5.4 `git diff --stat` shows net LOC reduction.
