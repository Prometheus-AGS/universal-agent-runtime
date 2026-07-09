## Why

Two remaining Runtime Console dead-facade items from this phase's
assessment, both in `frontend/src/admin/pages/runtime-console-page.tsx`:

1. `RuntimeRunsPage`'s Artifacts panel is backed by `RuntimeArtifact`
   entities the backend never populates — same facade class, same
   fix-vs-remove tension already resolved (AskUserQuestion, 2026-07-09) for
   the Protocols/Cockpit panels: apply the cheap "not yet wired" disclosure.
2. `RunRow`'s "Inspect" button (used on both `RuntimeCockpitPage`'s Live
   Runs panel and `RuntimeRunsPage`'s Runs list) has no `onClick` at all —
   a literally dead button, worse UX than no button. This is a real,
   proportionate fix (not a facade decision): `RuntimeRunsPage` already
   loads full run/step/tool/artifact data into the graph; it just always
   shows `runs[0]` in its Run Detail column instead of whichever run the
   operator picked.

## What Changes

- Artifacts panel: replace its `EmptyRuntimeState` fallback with the
  existing `NotWiredRuntimeState` disclosure component (added by
  `resolve-runtime-protocols-page-facade`).
- Inspect button: `RuntimeRunsPage` gains `selectedRunId` state
  (defaulting to the first run) and passes an `onInspect` handler to
  `RunRow` that sets it; the Run Detail column renders the selected run
  (falling back to the first run only if none is selected), not always
  `runs[0]`.
- On `RuntimeCockpitPage` (no run-detail column to select into), Inspect
  navigates to the Runs page with the target run preselected via a
  `?run=<id>` search param, which `RuntimeRunsPage` reads on mount.

## Impact

- Affected capability: `runtime-console-ux` (extends the disclosure
  requirement to the Artifacts panel; adds a new requirement for
  run-selection/inspection).
- Affected code: `frontend/src/admin/pages/runtime-console-page.tsx`.
