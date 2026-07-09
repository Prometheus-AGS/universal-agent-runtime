## 1. Apply to RuntimeCockpitPage

- [x] 1.1 Replace the `EmptyRuntimeState` fallback on the "Provider Health" panel with `NotWiredRuntimeState`.
- [x] 1.2 Replace the `EmptyRuntimeState` fallback on the "Memory Activity" panel with `NotWiredRuntimeState`.
- [x] 1.3 Leave Live Runs, Execution Timeline, and the four stat tiles untouched — they are backed by real, populated entities.

## 2. Verify

- [x] 2.1 `pnpm run build` clean.
- [x] 2.2 Confirm `git status --short` shows only the expected file(s) changed. Matches exactly: `frontend/src/admin/pages/runtime-console-page.tsx`, rebuilt `static/index.html`, this change's own `openspec/changes/` dir, plus this phase's `progress.json`.
