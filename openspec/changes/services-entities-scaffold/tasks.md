## Discovery

Inspecting the codebase reveals an entity-mgmt scaffold already lives under `frontend/src/entities/fetchers/` and `frontend/src/entities/hooks/`. Five entities are already covered by prior work in this branch:

- `frontend/src/entities/fetchers/agents.ts` + `hooks/use-agents.ts`
- `frontend/src/entities/fetchers/knowledge.ts` + `hooks/use-knowledge.ts`
- `frontend/src/entities/fetchers/providers.ts` + `hooks/use-providers.ts`
- `frontend/src/entities/fetchers/skills.ts` + `hooks/use-skills.ts`
- `frontend/src/entities/fetchers/tools.ts` + `hooks/use-tools.ts`

This is more comprehensive than the per-entity `services/entities/*.ts` shape called for in the plan, but functionally equivalent — both translate REST responses into graph writes. **Rather than fork the architecture**, we treat `frontend/src/entities/` as the canonical location and only **add** the missing 8 entity scaffolds.

## 1. Existing scaffolds (already in tree)

- [x] 1.1 `agents`
- [x] 1.2 `knowledge_base` + `knowledge_document` (combined under `knowledge.ts`)
- [x] 1.3 `providers`
- [x] 1.4 `skills`
- [x] 1.5 `tools`

## 2. Missing scaffolds — TO ADD

- [ ] 2.1 `models` → `entities/fetchers/models.ts` + `entities/hooks/use-models.ts`
- [ ] 2.2 `setting` → `entities/fetchers/settings.ts` + `entities/hooks/use-settings.ts`
- [ ] 2.3 `memory` → `entities/fetchers/memory.ts` + `entities/hooks/use-memory.ts`
- [ ] 2.4 `api_key` → `entities/fetchers/api-keys.ts` + `entities/hooks/use-api-keys.ts`
- [ ] 2.5 `compiler_session` → `entities/fetchers/compiler-sessions.ts` + `entities/hooks/use-compiler-sessions.ts`
- [ ] 2.6 `mcp_status` → `entities/fetchers/mcp-status.ts` + `entities/hooks/use-mcp-status.ts`
- [ ] 2.7 `thread` → `entities/fetchers/threads.ts` + `entities/hooks/use-threads.ts`

## 3. Status

**PARTIAL.** The five most-trafficked entity types are already covered. The remaining 8 will be added when their consumer migration (changes 4–5) reaches them — there's no value in dead scaffolds.
