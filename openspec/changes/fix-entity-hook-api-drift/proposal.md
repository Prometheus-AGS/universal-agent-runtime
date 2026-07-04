# fix-entity-hook-api-drift

## Why

Disclosed in the CH-07/CH-10 proposals: the `prometheus-entity-management`
submodule was uninitialized (empty) for this entire phase until it was built
this session — which is why nobody had caught that 6 call sites
(`use-models.ts`, `use-compiler-sessions.ts`, `use-mcp-status.ts`,
`use-memory.ts`, `use-settings-entity.ts`, `tools-page.tsx`) were calling the
library's `useEntityList`/`useEntity` with a pre-2.0, bare-`EntityType`-string
API that no longer exists. The library's current `useEntityList`/`useEntity`
take a full `ListQueryOptions`/`EntityQueryOptions` object (and are themselves
now deprecated in favor of a transport-registry-driven `useEntities`/
`useEntityQuery` pair, per the library's own `@deprecated` JSDoc and
`CLAUDE.md`). The mismatched calls didn't throw — they silently produced
empty results, breaking the models catalog, compiler sessions, MCP status,
memory, and settings-namespace pages.

## What changed

- New shared `useGraphEntities<T>(type)` / `useGraphEntity<T>(type, id)`
  (`frontend/src/entities/hooks/use-graph-entities.ts`): thin
  `useGraphStore` selectors matching the pattern this app already uses
  successfully elsewhere (`runtime-console-page.tsx`'s local helper,
  this phase's new `cost-dashboard-page.tsx`). This app hydrates entities
  explicitly (a fetcher module REST-fetches then `upsertEntity`s; SSE keeps
  the graph fresh thereafter) rather than through the library's
  transport-registry hooks, which require a `registerEntityTransport` call
  this app never makes (and which is Supabase/PostgREST-shaped —
  `makeRestTransport` isn't a fit for UAR's plain-`fetch` Rust REST API
  without writing a custom `EntityTransport` per entity type, a larger
  migration than this fix warrants).
- All 6 broken call sites now use `useGraphEntities`/`useGraphEntity`
  instead of the library's deprecated/incompatible hooks.
- Fixed an adjacent bug in the same function: `useSettingsEntity` read
  `r.value`, but `SettingWithMeta` has no `value` field (it's `data`) — the
  `values` map it built was always empty regardless of the hook API issue.
- `CompareDialog`'s (CH-10) width override was `max-w-3xl`, but the base
  `DialogContent` sets `sm:max-w-md` — `tailwind-merge` doesn't treat an
  unprefixed override as conflicting with a `sm:`-prefixed base class, so
  the override silently lost and the dialog rendered at 448px instead of
  768px, forcing an undiscoverable horizontal scroll for a 2-column compare.
  Fixed by matching the modifier (`sm:max-w-3xl`). Found while verifying
  this fix end-to-end against a live server (the models catalog now
  actually populates, which is what surfaced it).

## Verification

- `bun run typecheck`: all "Expected 2 type arguments" / hook-shape errors
  gone (17 remaining errors are pre-existing, unrelated: Base UI `Select`
  `string | null` nullability, `react-resizable-panels` API drift,
  `recharts` type-export drift — none touch entity hooks).
- `bun run build` green.
- Live server smoke test (embedded SurrealDB, `UAR_SECURITY__JWT_REQUIRED=false`):
  models catalog now shows **5331 of 5331 in catalog, 150 providers**
  (was 0 before this fix). Exercised CH-10's compare feature end-to-end
  with real data: selected 2 real models via the catalog checkboxes,
  opened the compare dialog, confirmed both columns render correctly
  (context window, cost, capabilities) with zero console errors.
- `cargo test --lib`: 330/330 (unaffected, backend-only change was none here).
