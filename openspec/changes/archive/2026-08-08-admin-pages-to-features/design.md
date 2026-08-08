## Context

The production admin composition root currently imports thirteen page files from `frontend/src/admin/pages/`. Those pages depend on domain hooks in `src/hooks/`, Zustand stores in `src/stores/`, entity helpers in `src/entities/`, REST clients in `src/services/`, and six admin-only helper components. The result is a technical-layer graph rather than the target `app → features → shared → platform` graph.

C-14a is an ownership migration, not a redesign. C-14b still owns decomposition of the 3,336-line settings page; C-14c owns deletion of the remaining legacy tree, retired stores/dependencies, and installation of the final boundary zones. Existing provider/model/backend/protocol contracts, entity-graph synchronization, PGlite behavior, and route destinations are binding compatibility surfaces.

## Goals / Non-Goals

**Goals:**

- Establish thirteen independently reviewable feature slices: `agents`, `auth`, `compiler`, `cost`, `credentials`, `knowledge`, `memory`, `models`, `providers`, `runtime`, `settings`, `skills`, and `tools`.
- Give each slice explicit `ui/`, `model/`, and, where applicable, `api/` ownership with deliberate public entry contracts: the root `index.ts` for page composition plus narrow `api/index.ts`, `model/index.ts`, or named root entries for observed lightweight consumers.
- Move page-owned hooks, stores, helpers, tests, and REST clients with the page so the resulting feature is coherent rather than a path-only shell.
- Preserve all page exports, routes, operations, loading/empty/error states, reactive entity updates, and user-visible behavior.
- Replace the C-14a-owned legacy color expressions with existing semantic Tailwind 4 tokens without changing their meaning.

**Non-Goals:**

- Redesigning configuration workflows or introducing new actions.
- Splitting the settings page by domain; C-14b owns that work.
- Deleting the legacy admin shell, terminal-theme wrapper, TanStack Query, Radix declarations, highlight.js, or retired stores; C-14c owns those removals.
- Moving the retired A2UI testing page or MCP health page unless required to keep the existing composition root compiling; their final disposition remains C-14c-owned.
- Changing Rust, REST payloads, provider routing, AG-UI/A2UI contracts, entity schemas, or persistence.

## Decisions

### 1. Migrate one vertical ownership cluster at a time

Each page moves with the files that directly implement its UI → hook → store → API path:

| Slice | Page and owned cluster |
|---|---|
| `agents` | agents page, AI builder/editor, agents admin hook/store/tests, agents API and agent-specific model helpers |
| `auth` | API-key page, auth-key hook/store/tests, auth API |
| `compiler` | compiler page, compiler hook/store/tests, compiler API |
| `cost` | cost dashboard and its entity projection helpers |
| `credentials` | credentials page, hook/store/tests, credentials API |
| `knowledge` | knowledge page, directly owned model/store/fetcher code, knowledge API |
| `memory` | memory page, admin hook/store/tests and memory API ownership |
| `models` | models page, catalog view model, model/provider-model hooks and stores, models API |
| `providers` | providers page and welcome UI, provider hooks/stores/tests, providers API |
| `runtime` | runtime-console page/tests, runtime hook/store/feed code, runtime-console API |
| `settings` | settings page moved intact, settings hooks/stores/tests and settings/user-settings API clients |
| `skills` | skills page/helpers/tests, import dialog, skills hook/store/tests, skills API |
| `tools` | tools page/detail UI, tools hook/store/tests, tools API |

Shared admin loading, empty, and error projections move to `shared/ui/configuration/`. A page must never import another feature's implementation path. Existing cross-domain consumers of a moved API or model helper use a deliberate narrow public entry (`api/index.ts`, `model/index.ts`, or a named root entry) until C-14c installs the final import zones. Broad root barrels are not used when they would pull an admin page into the initial application graph.

Alternative considered: move only the thirteen `.tsx` files and REST clients. Rejected because the hooks would still reach global stores which reach feature APIs, leaving an inverted and partially migrated call graph.

### 2. Preserve exported component contracts at the composition root

`pages/admin-page.tsx` continues to own section-to-page composition during C-14a, but imports page components from feature public entries. Exported names such as `ProvidersPage`, `SettingsPage`, `RuntimeRunsPage`, and the other runtime-console projections remain stable. The `AdminSection` inventory, query-string behavior, navigation, and onboarding banner placement remain unchanged.

Alternative considered: replace the admin composition root during the moves. Rejected because shell retirement belongs to C-14c and would mix behavior change with ownership migration.

### 3. Use mechanical moves first, then import rewrites

For each slice:

1. Capture its existing focused tests and import consumers.
2. Move the page and owned cluster without editing behavior.
3. Rewrite imports to the new relative/internal paths or public feature entry.
4. Update the composition root and focused tests.
5. Run typecheck, lint, boundary, Flat 2.0, and only that slice's focused tests before proceeding.

The plan requests one commit per page. Because this workspace already contains operator-owned changes and no staging/commit authorization was given, the implementation retains one page-sized diff/checkpoint at a time without staging; commits can be created later only on explicit request.

### 4. Restrict token migration to the five C-14a-owned files

The legacy `hsl(var())` expressions in models, memory, cost, skills, and compiler are converted to the semantic `var(--color-*)` form introduced by C-02. No new palette values are introduced, semantic warning/error/success meaning is preserved, and status does not become color-only. The other eight page moves remain visual no-ops.

Alternative considered: run a repository-wide codemod. Rejected because C-05 already owned non-admin occurrences and C-14a is explicitly scoped to the remaining admin-page set.

### 5. Keep API and realtime behavior byte-for-byte compatible where practical

Moved API modules retain their exported functions, request methods, paths, bodies, response decoding, and error behavior. Stores retain existing refresh/subscription order. Runtime-console and entity-backed pages continue using the current entity graph and run-trace paths; no new polling, cache, or optimistic behavior is added.

### 6. Treat transitional public entries as the only cross-slice seam

Every feature exposes only the symbols required by `admin-page.tsx`, legacy entity consumers, or another already-observed caller. Page composition uses the feature root; lightweight consumers use a narrow public `api/index.ts`, `model/index.ts`, or named root entry rather than an implementation file. The initial broad-barrel arrangement measured 303,220 gzip bytes against the 250,000-byte budget because lightweight chat consumers pulled admin pages into the initial graph. Narrow public entries restored the measured closure to 242,518 bytes. This makes C-14c's boundary-zone enforcement a contraction of the allowed surface rather than another migration while preserving the established bundle budget.

## Risks / Trade-offs

- **[Risk] A mechanical move drops an undocumented page behavior.** → Preserve component bodies first, update paths separately, and run page-specific tests after each slice.
- **[Risk] Shared service consumers create reverse dependencies during the transition.** → Move the owning implementation once and route observed external consumers through the feature's public entry; record any remaining legacy consumer for C-14c.
- **[Risk] Settings work overlaps C-14b.** → Move `settings-page.tsx` intact in C-14a and defer all decomposition or behavior edits.
- **[Risk] Shared admin helper placement creates a dumping ground.** → Only state projections reused by multiple independent slices move to `shared/ui/configuration`; domain-specific components move with their owner.
- **[Risk] The 307 token edits obscure behavior-preserving moves.** → Apply token conversion only after each affected file is in its destination and verify zero remaining C-14a-owned expressions with the published matcher.
- **[Risk] Existing operator changes are accidentally staged or overwritten.** → Do not stage or commit, preserve the two staged license deletions, and exclude `.gitmodules`, `crates/prometheus-skill-system`, and `src/uar` from every operation.

## Migration Plan

1. Record baseline inventory, focused tests, route exports, and the C-03 Flat 2.0 allowlist entries for all thirteen pages.
2. Establish shared configuration-state helpers and feature public-entry conventions.
3. Move slices in low-coupling-to-high-coupling order: auth, credentials, tools, compiler, cost, memory, skills, knowledge, agents, providers, models, runtime, settings.
4. For each slice, migrate its observed ownership cluster, update all callers, shrink matching allowlist paths, run cheap/focused gates, and retain a page-sized diff checkpoint.
5. Convert the five in-scope legacy token sets and prove no `hsl(var())` remains in migrated feature pages.
6. Run consolidated C-14a frontend verification, strict OpenSpec validation, scope-integrity checks, and isolated adversarial review.
7. Transition canonical KBD C-14a, append the waypoint, archive the OpenSpec change, and advance to C-14b.

Rollback is path-local: restore the affected page cluster and its composition-root imports before moving to the next slice. No data migration or backend rollback is required.

## Open Questions

None. The operator has fixed the page inventory, strict C-14 ordering, token scope, and behavior-preserving requirement; observed cross-consumers determine the minimal public feature exports during implementation.
