## Why

Thirteen production configuration pages still live under the legacy `frontend/src/admin/pages/` tree while their hooks, stores, and REST clients are split across global technical-layer directories. This prevents the feature-owned architecture required by the active migration and blocks safe retirement of `src/admin/` in C-14c.

## What Changes

- Re-home the agents, auth, compiler, cost, credentials, knowledge, memory, models, providers, runtime-console, settings, skills, and tools page surfaces into explicit `frontend/src/features/<domain>/` slices without changing their user-visible behavior or backend contracts.
- Move each surface's owned REST client from `frontend/src/services/*-api.ts` into its feature `api/` boundary and relocate directly owned hooks, model helpers, stores, tests, and admin-only UI helpers when needed to keep the slice coherent.
- Replace the 307 legacy `hsl(var(--…))` usages owned by the five affected admin pages with the current Tailwind 4 design-token form while preserving semantic colors, status cues, and both themes.
- Update the admin composition root and tests to import each page through its feature's public entry point; preserve the existing route/section inventory, CRUD flows, loading/empty/error states, and runtime-console live state.
- Leave settings-page decomposition to C-14b and legacy-tree/dependency deletion plus final boundary-zone enforcement to C-14c.

## Capabilities

### New Capabilities

- `frontend-configuration-surfaces`: Defines behavior-preserving, feature-owned configuration surfaces and their UI/model/API ownership boundaries.

### Modified Capabilities

- None.

## Impact

- Frontend: `frontend/src/admin/pages/`, related admin-only components, feature-owned hooks/stores/model helpers, `frontend/src/services/*-api.ts`, `frontend/src/pages/admin-page.tsx`, tests, and import aliases.
- Runtime UX: the same configuration destinations, controls, validation, error handling, and reactive updates remain available; this change relocates ownership rather than redesigning behavior.
- Provider compatibility: provider/model/auth/credential payloads and endpoints remain unchanged, and all LLM access continues through existing runtime APIs and liter-llm routing.
- Realtime state: runtime-console, run-trace, agent, skill, memory, and related store/subscription semantics remain unchanged while their frontend ownership becomes explicit.
- Dependencies and backend: no new frontend dependency, Rust change, API route change, database migration, or protocol change is introduced.
- Workflow: canonical KBD C-14a is tracked independently; C-14b, C-14c, and C-14d remain strictly sequential follow-on changes.
