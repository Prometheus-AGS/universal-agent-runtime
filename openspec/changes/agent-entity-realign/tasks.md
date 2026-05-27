## 1. Type alias

- [ ] 1.1 Replace `AgentEntity` interface in `frontend/src/entities/types.ts` with `export type AgentEntity = UarAgent;`.
- [ ] 1.2 Add import `import type { UarAgent } from "@/types";` to that file.

## 2. Fetcher cleanup

- [ ] 2.1 Drop the `as unknown as Record<string, unknown>` cast in `entities/fetchers/agents.ts`.

## 3. Consumer audit

- [ ] 3.1 `git grep AgentEntity frontend/src` — confirm every consumer reads nested fields (none rely on the flat shape).

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend tsc --noEmit` passes.
- [ ] 4.2 `pnpm --filter ./frontend build` clean.
