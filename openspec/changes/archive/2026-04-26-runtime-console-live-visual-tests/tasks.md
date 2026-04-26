## 1. Workflow And Test Scope

- [x] 1.1 Confirm `openspec validate runtime-console-live-visual-tests` passes before implementation begins.
- [x] 1.2 Confirm KBD progress marks `runtime-console-live-visual-tests` as the active in-progress OpenSpec change.
- [x] 1.3 Identify the exact targeted Playwright command for this change and record it in verification notes.
- [x] 1.4 Keep this change scoped to visual/navigation verification; defer realtime entity replay behavior to `runtime-event-replay-entity-sync-tests`.

## 2. Runtime Console Selectors And Accessibility

- [x] 2.1 Review the admin shell, runtime console pages, providers page, memory page, approvals page, protocols page, and A2UI testing page for stable accessible landmarks.
- [x] 2.2 Add narrow `data-testid` attributes or accessible labels only where Playwright cannot reliably target existing shell controls.
- [x] 2.3 Ensure any selector-only production changes preserve the components -> hooks -> stores -> services layering.
- [x] 2.4 Ensure mobile navigation controls and command palette entry points have stable labels or selectors for browser tests.

## 3. Playwright Runtime Console Coverage

- [x] 3.1 Add a targeted `frontend/e2e/runtime-console-visual.spec.ts` suite or equivalent focused spec file.
- [x] 3.2 Add desktop viewport coverage for `/admin/runtime` verifying shell navigation, cockpit content, and contextual provider/workflow panels.
- [x] 3.3 Add desktop navigation coverage for runs, approvals, protocols, providers, memory, and A2UI testing surfaces.
- [x] 3.4 Add mobile viewport coverage for opening navigation, routing to a runtime surface, and confirming the overlay no longer blocks selected content.
- [x] 3.5 Add command palette coverage that opens the palette and routes to provider diagnostics or another stable runtime destination.
- [x] 3.6 Add bounded overlap checks for primary headings, navigation controls, and visible action controls without relying on pixel-perfect snapshots.
- [x] 3.7 Assert intended empty states are accepted so tests do not require live runtime entities, provider credentials, or model responses.

## 4. Validation And KBD Closure

- [x] 4.1 Run `bun run lint` from `frontend/`.
- [x] 4.2 Run `bun run typecheck` from `frontend/`.
- [x] 4.3 Run the targeted runtime console Playwright suite from `frontend/`.
- [x] 4.4 Run `openspec validate runtime-console-live-visual-tests`.
- [x] 4.5 Update `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` with task completion and verification evidence.
- [x] 4.6 Run artifact-refiner QA for `runtime-console-live-visual-tests` before archive unless the final implementation remains documentation-only.
- [x] 4.7 Leave `openspec validate --changes` global failure attributed to `implement-opencode-suggestions` until `openspec-global-validation-cleanup` resolves it.
