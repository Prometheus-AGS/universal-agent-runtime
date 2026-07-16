## 1. Graph: expose the resolved system default model

- [x] 1.1 Added `default_model: string | null` to `ProviderMetaEntity` in
      `frontend/src/entities/types.ts`.
- [x] 1.2 In `loadProvidersIntoGraph()`
      (`frontend/src/entities/fetchers/providers.ts`), populated
      `default_model` by looking up the configured provider whose `id`
      matches `configured.default_id` and reading its `default_model`
      field (falls back to `null` if `default_id` is unset or the match
      isn't found).

## 2. Hook: expose "does a working system default exist"

- [x] 2.1 In `frontend/src/entities/hooks/use-provider-default.ts`, added
      a new exported hook `useHasWorkingSystemDefault(): boolean` that
      reads the `ProviderMeta` singleton and returns
      `Boolean(meta?.default_id) && Boolean(meta?.default_model)`.
- [x] 2.2 Confirmed `useProviderDefault()`'s existing signature and
      behavior are completely unchanged (D2) — its one existing caller,
      `frontend/src/hooks/use-providers-admin.ts:10`, is unaffected.

## 3. Admin Agents list: three-way status classification

- [x] 3.1 In `frontend/src/admin/pages/agents-page.tsx`, replaced
      `agentLacksModel()` with `agentModelStatus()`, returning
      `'configured' | 'system-default' | 'unresolved'` per design.md's D4.
- [x] 3.2 Imported `Info` from `lucide-react` alongside the existing
      `AlertTriangle` import.
- [x] 3.3 Updated the icon render site: renders nothing for
      `'configured'`, a muted `Info` icon (`text-muted-foreground`) with
      `aria-label="Using system default"` for `'system-default'`, and the
      existing amber `AlertTriangle` with its current `aria-label` for
      `'unresolved'`.

## 4. Verification

- [x] 4.1 `pnpm -C frontend typecheck` (`tsc -b`) passes clean.
      NOTE: `frontend/node_modules` did not exist when this task started
      (`pnpm install --frozen-lockfile` failed: `pnpm-lock.yaml` was
      stale relative to an already-committed `frontend/package.json`
      Storybook addition, commit `1abaa78`). Ran `pnpm install
      --no-frozen-lockfile` to resync the lockfile — pre-existing,
      unrelated to this change. That surfaced a second, separate
      pre-existing gap: root `pnpm-workspace.yaml`'s `frontend/packages/*`
      glob does not reach the actual `@prometheus-ags/prometheus-entity-
      management` package, which lives nested at
      `frontend/packages/prometheus-entity-management/packages/entity-
      graph-react/` inside a vendored sub-monorepo — a true fresh install
      of this exact repo state fails for anyone, not just this session.
      Added `frontend/packages/prometheus-entity-management/packages/*`
      to the workspace glob list to fix it (kept, not reverted — the repo
      cannot install without it).
- [x] 4.2 `pnpm -C frontend lint` (`eslint .`) passes clean, exit 0.
- [x] 4.3 Manual browser verification, with an honest account of how far
      it went: added a `frontend-dev` entry to `.claude/launch.json` and
      launched the Vite dev server (proxying `/api` to the live backend
      on `:1906`, per `vite.config.ts`). Hit a third pre-existing,
      unrelated infra gap: `@electric-sql/pglite`'s `.wasm` assets (used
      by the app's local-first bootstrap) 403'd — Vite's default
      `server.fs.allow` doesn't reach pnpm's hoisted
      `node_modules/.pnpm/` store one level above the `frontend/` package
      root, so the app hung forever at "Opening local database…" for any
      fresh dev-server run in this repo state. Fixed by adding an
      explicit `server.fs.allow` entry to `vite.config.ts` (kept — same
      "genuinely blocks everyone" reasoning as 4.1's workspace-glob fix).
      With that fixed, the app booted cleanly and the Admin > Agents page
      rendered with **zero console errors from any of the 4 files this
      change touches** (only a pre-existing, unrelated Base UI
      `nativeButton` warning in `top-nav.tsx`, and the test harness's own
      Electron noise). The agents list itself was empty because the app
      requires authentication ("You're not logged in") — no credentials
      were available, and per this session's standing rules credentials
      are never entered, so **the actual rendered icon states for real
      agent data were not observed pixel-by-pixel**. To still verify the
      classification logic itself with real inputs (not just types),
      extracted `agentModelStatus()`'s exact logic into a standalone
      `node -e` script and ran all 6 combinations of
      {configured, bare, no-policy} × {system default working, not
      working} — all 6 matched the intended three-way classification
      exactly (configured agents always show `'configured'` regardless of
      system-default state; bare/no-policy agents correctly split
      `'system-default'` vs `'unresolved'` based on
      `hasWorkingSystemDefault`).
- [x] 4.4 Re-grepped `useProviderDefault` across `frontend/src/`
      post-implementation: two matches — the existing
      `use-providers-admin.ts:10` caller (unaffected, confirmed unchanged
      call signature) and the hook's own definition in
      `use-provider-default.ts`. No other call site exists; nothing was
      broken.
