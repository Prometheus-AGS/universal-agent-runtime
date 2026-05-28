## Why

`AgentEntity` in `frontend/src/entities/types.ts` declares a flat shape (`name`, `description`, `model`, `protocol`, …). `loadAgentsIntoGraph()` in `entities/fetchers/agents.ts` casts away the difference and upserts the raw `UarAgent` (nested: `metadata.title`, `policy.provider.default.model`, …). Every consumer that reads agent data reads through the nested shape — the flat declaration has never been correct.

Aligning the type to reality unblocks the rest of the agent migration by giving `useAgents()` an accurate return type.

## What Changes

- Replace the `AgentEntity` interface in `entities/types.ts` with `export type AgentEntity = UarAgent;` (importing `UarAgent` from `@/types`).
- Remove the `as unknown as Record<string, unknown>` cast in `loadAgentsIntoGraph()` — once the types line up, an `upsertEntity("Agent", a.id, a)` direct call works.
- Audit any other consumer that imported `AgentEntity` expecting the flat shape; either update them or document the change.

## Acceptance

- `pnpm --filter ./frontend tsc --noEmit` passes.
- `pnpm --filter ./frontend build` clean.
- No runtime behaviour change — entries already match `UarAgent` at runtime.
