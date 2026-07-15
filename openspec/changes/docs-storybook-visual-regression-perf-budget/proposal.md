## Why

The phase plan frames this as "Storybook 8 with Chromatic for visual
regression; 30+ components; performance budget CI gate; A2UI Inspector as a
Storybook addon (from Change 22)." Auditing before building found two
corrections to that premise:

- **Storybook 8 is stale.** The plan predates Storybook's current release
  line -- `npm view storybook version` resolves `10.5.0` today. This change
  installs and configures the current stable major (10.5), not 8.
- **The literal 16ms/8ms performance-budget gate already exists.** Change 17
  (`a2ui-uar-renderer-on-webcore`) built `src/perf/measure.ts` and a real,
  CI-wired gate (`a2ui-renderer-performance.yml`, triggered on
  `frontend/packages/a2ui-uar/**`) asserting exactly those numbers
  (`CI_INITIAL_RENDER_BUDGET_MS`/`CI_STREAMING_UPDATE_BUDGET_MS`, scaled for
  CI-runner noise per that change's README). This change's new stories live
  under `frontend/packages/a2ui-uar/src/**`, so they're already covered by
  that existing trigger path -- building a second, parallel perf harness
  here would duplicate CI cost for no additional coverage. What *is* new is
  a Storybook-specific CI job (`storybook-visual-regression.yml`'s `test`
  job) that builds the Storybook bundle and runs every story through
  `@storybook/addon-vitest` + `@storybook/addon-a11y` in a real headless
  Chromium browser -- a functional/a11y regression gate the perf workflow
  doesn't provide.

## What Changes

- **Storybook 10.5** configured at `frontend/.storybook/`, covering both
  `frontend/src/**` (the shadcn/ui visual baseline) and
  `frontend/packages/a2ui-uar/src/**` (the A2UI protocol + entity
  components) via `tailwind.config.ts`'s `content` glob and the Storybook
  `stories` glob.
- **38 stories across 35 components** (16 `@prometheus-ags/a2ui-uar`
  components: all 9 `uar.a2ui/1` protocol components + all 7 `Entity*`
  components, each rendered through a real `MessageProcessor` surface via
  a new `src/dev/build-surface.tsx` helper -- not hand-constructed props
  that could drift from the real wire protocol; plus 19 representative
  shadcn/ui primitives from `frontend/src/components/ui/`), clearing the
  plan's "30+ components" done-condition.
- **`@storybook/addon-a11y` wired fail-closed** (`test: 'error'` in
  `.storybook/preview.tsx`, not the scaffold default `'todo'`), catching
  real accessibility regressions as CI failures rather than silent
  dashboard entries.
- **`storybook-visual-regression.yml`**: a `test` job (build Storybook,
  run every story through `vitest --project=storybook` in headless
  Chromium) that blocks on any story that throws or fails its a11y check,
  and a `chromatic` job that publishes to Chromatic for visual review --
  skipped with a `::notice::` (not a failure) until `CHROMATIC_PROJECT_TOKEN`
  is set, an operator credential decision (a Chromatic account/project must
  exist first).

## Real bugs found and fixed while wiring the a11y gate

Turning `addon-a11y` on fail-closed immediately caught two categories of
pre-existing issues, not story-authoring mistakes:

- **Fixed in this change:** `EntityCard`'s sync-origin badge
  (`bg-amber-500/15 text-amber-600`) measured 2.83:1 contrast against WCAG
  AA's 4.5:1 minimum. Bumped to `text-amber-800` (light) /
  `text-amber-400` (dark, unchanged) -- now passes. `Slider`
  (`frontend/src/components/ui/slider.tsx`) had no way to label its thumb
  (`aria-label` landed on `SliderPrimitive.Root`, but the accessible
  `<input type="range">` lives inside `SliderPrimitive.Thumb`) -- added an
  `aria-label` passthrough to the Thumb.
- **Flagged as follow-up, not fixed here (design-system-wide token
  changes, out of this change's scope):** `--primary`/
  `--primary-foreground` (white text on `#ff5500`, 3.2:1) affects every
  default `Button`/`Badge` variant app-wide; `--muted-foreground`/`--muted`
  (4.3:1, 3.56:1 on `TabsTrigger`'s inactive state) affects `Kbd`, `Avatar`
  fallback, and inactive tabs app-wide. Excluded via per-story
  `a11y: { test: 'off' }` with an inline comment pointing back here, so
  the gate stays fail-closed for everything else without silently losing
  the finding.

## Capabilities

### New Capabilities

- `visual-regression-2026`: Storybook + Chromatic + the story-level
  functional/a11y CI gate.

## Impact

- New `frontend/.storybook/` config, `frontend/src/stories.tsx` equivalents
  (per-component `*.stories.tsx` files), `frontend/packages/a2ui-uar/src/dev/build-surface.tsx`.
- New `.github/workflows/storybook-visual-regression.yml`.
- `frontend/tailwind.config.ts`'s `content` glob extended to include
  `packages/a2ui-uar/src/**/*.{ts,tsx}` so Tailwind classes used there are
  generated when Storybook renders them.
- Two small, targeted a11y fixes: `EntityCard.tsx` (contrast), `slider.tsx`
  (thumb labeling).
- `frontend/eslint.config.js`: `storybook-static` added to `ignores`
  (Storybook's build output was otherwise being linted as source).

## Out of scope

- **A2UI Inspector as a Storybook addon** (the plan's third done-condition
  bullet). Change 22 (`a2ui-inspector-lit-svelte-renderers`), which owns
  building the Inspector itself, has not landed yet -- there is nothing to
  wrap in a Storybook addon. Genuinely blocked, not soft: unlike several
  other cross-change "dependencies" in this phase that turned out to be
  aspirational, this one is a hard prerequisite (you cannot address a
  component that does not exist). Revisit once Change 22 ships.
- **Fixing the `--primary`/`--primary-foreground` and
  `--muted-foreground`/`--muted` contrast gaps.** Both are foundational
  design tokens used throughout the shipped product, not local to any
  story or component this change owns. Fixing them is a design-system
  decision (which shade, whether it changes the brand orange) better made
  deliberately than as a side effect of a Storybook change -- see "Real
  bugs found" above for the specific measurements.
- **A dedicated Storybook-specific performance budget.** The literal
  16ms/8ms numbers are already gated by Change 17's
  `a2ui-renderer-performance.yml`, which already triggers on this change's
  new story files (they live under `frontend/packages/a2ui-uar/**`).
  Building a second harness would duplicate CI cost for the same
  coverage.
- **Cross-testing every story against `@a2ui/react`** (Google's reference
  renderer). Change 17 already does this for a representative subset;
  extending it to the full story set is that change's scope, not this
  one's.
