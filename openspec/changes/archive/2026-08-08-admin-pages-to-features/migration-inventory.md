# C-14a migration inventory and baseline

## Production page inventory

| Slice | Current page | Lines | Page-owned hook/store/API cluster |
|---|---|---:|---|
| agents | `admin/pages/agents-page.tsx` | 474 | `use-agents-admin`, `agents-admin-store`, `agents-api`, agent builder/editor |
| auth | `admin/pages/auth-page.tsx` | 136 | `use-auth-keys`, `auth-keys-store`, `auth-api` |
| compiler | `admin/pages/compiler-page.tsx` | 109 | `use-compiler`, `compiler-store`, `compiler-api` |
| cost | `admin/pages/cost-dashboard-page.tsx` | 281 | entity-graph projection only |
| credentials | `admin/pages/credentials-page.tsx` | 320 | `use-credentials`, `credentials-store`, `credentials-api` |
| knowledge | `admin/pages/knowledge-page.tsx` | 792 | knowledge entity/store/fetcher and `knowledge-api` |
| memory | `admin/pages/memory-page.tsx` | 411 | `use-memory-admin`, `memory-admin-store`, memory entity/fetcher and `memory-api` |
| models | `admin/pages/models-page.tsx` | 893 | `use-models-admin`, `model-catalog-view`, model/provider-model stores and `models-api` |
| providers | `admin/pages/providers-page.tsx` | 483 | provider hooks/stores, `providers-api`, admin welcome |
| runtime | `admin/pages/runtime-console-page.tsx` | 563 | runtime hook/store/feed, `runtime-console-api`, run-trace test |
| settings | `admin/pages/settings-page.tsx` | 3,336 | settings/onboarding/type-meta/JWT hooks and stores; settings/onboarding/user-settings APIs |
| skills | `admin/pages/skills-page.tsx` | 491 | skills hook/store/API, page utils/tests, import dialog |
| tools | `admin/pages/tools-page.tsx` | 124 | tools hook/store/API/entity fetcher, detail panel |

Production page source totals 8,413 lines (8,523 including the runtime trace test). The exact token inventory is 307 occurrences: models 105, memory 103, cost 39, skills 33, compiler 27.

## Shared and cross-consumer constraints

- `admin-states.tsx` is used by agents, auth, credentials, providers, tools, A2UI testing, and MCP health; it is the only multi-feature shared configuration-state projection.
- Agent builder/editor, skill import, and tool detail are domain-owned and move with those features.
- `agents-api` is also consumed by chat run-trace lookup; `models-api` by chat, onboarding, provider fetchers, and model selector; `providers-api` by chat session config, runtime console, and models; `settings-api` by chat streaming and entity settings; skills/tools/knowledge APIs by the agent editor. Those callers must use explicit owning-feature exports.
- The inventory scan found 140 current references across the page, hook, store, service, entity, test, and composition-root paths in scope.
- `A2uiTestingPage.tsx`, `McpHealthPage.tsx`, `admin-shell.tsx`, the terminal-theme wrapper, and dependency/store retirement remain C-14c-owned except for import rewrites needed to consume the shared configuration states.

## Baseline evidence

- `pnpm -C frontend typecheck`: pass.
- `pnpm -C frontend lint`: pass.
- `node scripts/check-frontend-boundaries.mjs`: pass, zero production violations.
- `node scripts/check-flat2-style.mjs`: pass, 385 tracked legacy violations and zero new.
- Twelve focused page/hook/store files: 57 tests passed.
- Protected paths at entry remain operator-owned: `.gitmodules`, `crates/prometheus-skill-system`, and `src/uar/*`; staged deletions remain `LICENSE-COMMERCIAL.md` and `sdks/rust/LICENSE-AGPL`.
