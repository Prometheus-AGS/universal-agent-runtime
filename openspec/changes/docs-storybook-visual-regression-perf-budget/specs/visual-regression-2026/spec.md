# Visual regression 2026

## Purpose

Give every A2UI protocol/entity component and a representative slice of
the shadcn/ui visual baseline a real, browser-rendered Storybook story,
gated in CI for functional and accessibility regressions, with visual
review wired through Chromatic once an operator provisions a project
token.

## ADDED Requirements

### Requirement: Storybook covers 30+ components across both component sources
`frontend/.storybook/main.ts`'s `stories` glob MUST include both
`frontend/src/**` and `frontend/packages/a2ui-uar/src/**`, and the
combined story set MUST cover at least 30 distinct components.

#### Scenario: Storybook builds with stories from both sources
- **WHEN** `pnpm run build-storybook` runs in `frontend/`
- **THEN** the build succeeds
- **AND** the resulting story index includes stories from both
  `frontend/packages/a2ui-uar/src/**` and `frontend/src/components/ui/**`

### Requirement: A2UI stories render through the real protocol path
Every `@prometheus-ags/a2ui-uar` component story MUST render via a real
`MessageProcessor` surface (`createSurface`/`updateComponents`/
`updateDataModel` messages processed through the component's actual
catalog), not hand-constructed props passed directly to the component
function.

#### Scenario: A protocol component story uses the real surface path
- **WHEN** a story under `frontend/packages/a2ui-uar/src/components/protocol.stories.tsx` renders
- **THEN** it does so via `renderStorySurface`, which drives a real `MessageProcessor` and `UarSurface`

### Requirement: The accessibility gate is fail-closed
`frontend/.storybook/preview.tsx`'s `a11y` parameter MUST be `test: 'error'`
by default. A story MAY opt out via a per-story `parameters.a11y.test: 'off'`
override only when the underlying issue is a documented, out-of-scope
design-system-wide gap (not a per-story authoring choice), with an inline
comment explaining why.

#### Scenario: A story with a real a11y violation fails CI
- **WHEN** `vitest run --project=storybook` runs against a story whose
  rendered output has an axe-detectable violation and no `a11y.test: 'off'` override
- **THEN** that story's test fails

### Requirement: CI gates on story functional and a11y regressions
`.github/workflows/storybook-visual-regression.yml`'s `test` job MUST
build the Storybook bundle and run every story through
`vitest --project=storybook` in a real headless browser, blocking the
workflow on any story that throws during render or fails its a11y check.

#### Scenario: A broken story blocks the workflow
- **WHEN** a story's `render` throws (e.g. an invalid component payload)
- **THEN** `storybook-visual-regression.yml`'s `test` job fails

### Requirement: Chromatic publish is opt-in via a repo secret, not a hard CI failure
`storybook-visual-regression.yml`'s `chromatic` job MUST only attempt to
publish to Chromatic when `CHROMATIC_PROJECT_TOKEN` is set, and MUST emit
a non-failing notice (not a workflow failure) when it is unset.

#### Scenario: Chromatic publish is skipped without a token
- **WHEN** `CHROMATIC_PROJECT_TOKEN` is not configured as a repository secret
- **THEN** the `chromatic` job's publish step is skipped
- **AND** the job still succeeds overall
