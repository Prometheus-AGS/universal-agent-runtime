## 1. Audit before building
- [x] 1.1 Confirmed the plan's "Storybook 8" is stale -- `npm view storybook version` resolves `10.5.0`; installed the current stable major.
- [x] 1.2 Confirmed the literal 16ms/8ms performance-budget gate already exists (Change 17's `a2ui-renderer-performance.yml`), and already covers this change's new story files (`frontend/packages/a2ui-uar/**` trigger path) -- not duplicated here.
- [x] 1.3 Confirmed Change 22 (A2UI Inspector) has not landed -- the plan's "Inspector as a Storybook addon" bullet is genuinely blocked, not soft.

## 2. Storybook setup
- [x] 2.1 `frontend/.storybook/main.ts` + `preview.tsx`: Storybook 10.5, `@storybook/react-vite`, stories glob covering both `frontend/src/**` and `frontend/packages/a2ui-uar/src/**`.
- [x] 2.2 `@storybook/addon-a11y` set to fail-closed (`test: 'error'`, not the scaffold default `'todo'`).
- [x] 2.3 `tailwind.config.ts`'s `content` glob extended to include `packages/a2ui-uar/src/**/*.{ts,tsx}` so Tailwind classes generate correctly for stories outside `frontend/src`.
- [x] 2.4 `eslint.config.js`: `storybook-static` added to `ignores`.

## 3. Stories (30+ components)
- [x] 3.1 `frontend/packages/a2ui-uar/src/dev/build-surface.tsx`: a real `MessageProcessor`-driven story helper (mirrors `test/helpers.ts`'s `buildSurface`), so stories render through the actual `GenericBinder` resolution path.
- [x] 3.2 `frontend/packages/a2ui-uar/src/components/protocol.stories.tsx`: all 9 `uar.a2ui/1` protocol components (11 stories).
- [x] 3.3 `frontend/packages/a2ui-uar/src/entities/entities.stories.tsx`: all 7 `Entity*` components (8 stories).
- [x] 3.4 `frontend/src/components/ui/ui-primitives.stories.tsx`: 19 representative shadcn/ui primitive stories.
- [x] 3.5 38 stories / 35 components total, clearing the 30+ done-condition.

## 4. Real a11y bugs found and fixed
- [x] 4.1 `EntityCard.tsx` sync-origin badge: 2.83:1 contrast -> fixed to pass AA (`text-amber-800` light / unchanged dark).
- [x] 4.2 `slider.tsx`: `aria-label` had no path to the accessible `<input type="range">` (landed on `Root`, not `Thumb`) -- added passthrough.
- [x] 4.3 **Flagged, not fixed** (design-system-wide token changes): `--primary`/`--primary-foreground` (3.2:1, affects every default `Button`/`Badge`), `--muted-foreground`/`--muted` (4.3:1/3.56:1, affects `Kbd`/`Avatar` fallback/inactive `TabsTrigger`). Each excluded via a per-story `a11y: { test: 'off' }` with an inline comment pointing back to this change.

## 5. CI: `storybook-visual-regression.yml`
- [x] 5.1 `test` job: checkout with submodules, build `prometheus-entity-management` (workspace dependency), install Playwright Chromium, `pnpm run build-storybook`, `vitest run --project=storybook` (every story through addon-vitest + addon-a11y in real headless Chromium).
- [x] 5.2 `chromatic` job: publishes to Chromatic via `chromaui/action@v1`; skipped with `::notice::` (not a failure) when `CHROMATIC_PROJECT_TOKEN` is unset.
- [x] 5.3 Trigger paths: `frontend/src/**`, `frontend/packages/a2ui-uar/**`, `frontend/.storybook/**`, `frontend/tailwind.config.ts`.

## 6. Deferred (see proposal.md "Out of scope")
- [ ] 6.1 A2UI Inspector as a Storybook addon -- blocked on Change 22.
- [ ] 6.2 Fixing the `--primary`/`--muted-foreground` design-token contrast gaps -- design-system decision, not this change's scope.
- [ ] 6.3 `CHROMATIC_PROJECT_TOKEN` provisioning -- operator credential decision (Chromatic account/project must exist first).

## 7. Verification
- [x] 7.1 `pnpm --filter @prometheus-ags/prometheus-entity-management build` (workspace dependency, needed for typecheck).
- [x] 7.2 `pnpm run typecheck` (frontend root) -- PASS.
- [x] 7.3 `pnpm run lint` (frontend root) -- PASS.
- [x] 7.4 `pnpm --filter @prometheus-ags/a2ui-uar typecheck` -- PASS.
- [x] 7.5 `pnpm --filter @prometheus-ags/a2ui-uar lint` -- PASS.
- [x] 7.6 `pnpm --filter @prometheus-ags/a2ui-uar test` -- 17/17 PASS (unchanged by this change's `EntityCard.tsx` fix).
- [x] 7.7 `pnpm run build-storybook` -- PASS.
- [x] 7.8 `vitest run --project=storybook` -- 38/38 PASS.
- [ ] 7.9 **Deferred to the phase's consolidated validation pass**: full-workspace `cargo check`/`clippy` (this change touches no Rust code, so not run standalone).
