## 1. Workspace and contracts

- [x] 1.1 Add package skeletons for `a2ui-inspector`, `a2ui-lit`, and `a2ui-svelte` with strict TypeScript, lint, test, and build lifecycles.
- [x] 1.2 Add verified `lit@3.3.3`, `svelte@5.56.5`, and `@sveltejs/vite-plugin-svelte@7.2.0` dependencies without adding Storybook runtime dependencies.
- [x] 1.3 Define a shared certified conformance fixture and normalized semantic snapshot contract.

## 2. Lit renderer

- [x] 2.1 Implement a reactive Lit surface element over `web_core` `SurfaceModel` state.
- [x] 2.2 Implement the 9-component certified baseline with dynamic bindings, structural children, actions, inputs, and fail-closed unknown-component errors.
- [x] 2.3 Add Lit renderer unit tests for baseline semantics, reactive updates, actions, and unknown components.

## 3. Svelte renderer

- [x] 3.1 Implement a Svelte surface component over `web_core` `SurfaceModel` state.
- [x] 3.2 Implement the 9-component certified baseline with dynamic bindings, structural children, actions, inputs, and fail-closed unknown-component errors.
- [x] 3.3 Add Svelte renderer unit tests for baseline semantics, reactive updates, actions, and unknown components.

## 4. Inspector

- [x] 4.1 Implement the injected SSE service and bounded Zustand store for connection, messages, selection, freeze/resume, queued count, dropped count, and last-good state.
- [x] 4.2 Implement hooks that expose Inspector state and actions without service access from components.
- [x] 4.3 Implement the responsive Inspector timeline, synchronized preview/source panes, explicit status/freeze states, filtering, copy affordances, and malformed/empty/disconnected recovery UI.
- [x] 4.4 Export a stable Storybook addon descriptor and Inspector panel entrypoint without bundling Storybook.
- [x] 4.5 Add Inspector store/component tests covering ingestion, freeze/resume, selection, bounded history, malformed input, and recovery.

## 5. Conformance and CI

- [x] 5.1 Run the shared fixture through React, Lit, and Svelte and compare normalized roles, accessible names, states, and visible text.
- [x] 5.2 Add a path-filtered CI workflow for all three package lifecycles and cross-renderer conformance.
- [x] 5.3 Document package usage, freeze semantics, security/redaction boundaries, addon consumption, and framework parity guarantees.

## 6. Quality gates

- [x] 6.1 Run Impeccable audit and dual-agent critique; resolve all P0/P1 findings applicable to Change 22.
- [x] 6.2 Run Impeccable polish against the completed Inspector and record remaining separately-scoped issues.
- [x] 6.3 Pass package-local typecheck, lint, tests, builds, cross-renderer conformance, frontend workspace typecheck/lint, and `git diff --check`. Also fixed 2 build hygiene issues found in this pass: a2ui-lit's/a2ui-inspector's dist/ output had been accidentally committed (removed, added `frontend/packages/*/dist/` to .gitignore); a stale rebuilt `static/index.html` bundle reference reverted to main's checked-in version.
- [x] 6.4 Pass `openspec validate a2ui-inspector-lit-svelte-renderers --strict`, artifact-refiner QA, and update canonical KBD progress/waypoint state.
