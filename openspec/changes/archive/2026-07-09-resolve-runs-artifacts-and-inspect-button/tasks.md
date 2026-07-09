## 1. Artifacts panel disclosure

- [x] 1.1 Replace the `EmptyRuntimeState` fallback on `RuntimeRunsPage`'s "Artifacts" panel with `NotWiredRuntimeState`.

## 2. Inspect button

- [x] 2.1 Add `onInspect?: (runId: string) => void` prop to `RunRow`; wire the "Inspect" button's `onClick` to call it when provided.
- [x] 2.2 In `RuntimeRunsPage`, add `selectedRunId` state (initialized from a `?run=` search param if present, else the first run's id once runs load); pass `onInspect={setSelectedRunId}` to each `RunRow`; render the Run Detail column for the selected run instead of always `runs[0]`.
- [x] 2.3 In `RuntimeCockpitPage`'s Live Runs panel, pass `onInspect` that navigates to the Runs page with `?run=<id>`.

## 3. Verify

- [x] 3.1 `pnpm run build` clean. `pnpm run typecheck` (`tsc -b`) also clean.
- [x] 3.2 Confirm `git status --short` shows only the expected file(s) changed. Matches exactly: `frontend/src/admin/pages/runtime-console-page.tsx`, rebuilt `static/index.html`, this change's own `openspec/changes/` dir, plus this phase's `progress.json`.
