## 1. Disclosure component

- [x] 1.1 Add a `NotWiredRuntimeState` component alongside `RuntimeProtocolsPage`'s existing local `EmptyRuntimeState` in `runtime-console-page.tsx`, visually distinct (different icon/tone) and with copy explicitly stating the panel is not yet wired to live backend data — not simply "no activity yet". Built on the project's existing, previously-unused `Alert`/`AlertTitle`/`AlertDescription` shadcn primitive rather than inventing a new one.

## 2. Apply to RuntimeProtocolsPage

- [x] 2.1 Replace the `EmptyRuntimeState` fallback on the "Recent AG-UI Events" panel with `NotWiredRuntimeState`.
- [x] 2.2 Replace the `EmptyRuntimeState` fallback on the "Model Routing" panel with `NotWiredRuntimeState`.
- [x] 2.3 Replace the `EmptyRuntimeState` fallback on the "A2UI Surfaces" panel with `NotWiredRuntimeState`.
- [x] 2.4 Leave the static protocol-surface cards (Anthropic REST, OpenAI REST, MCP) untouched — they are real, not facades.

## 3. Verify

- [x] 3.1 `pnpm run build` clean.
- [x] 3.2 Confirm `git status --short` shows only the expected file(s) changed. Matches exactly: `frontend/src/admin/pages/runtime-console-page.tsx`, rebuilt `static/index.html`, this change's own `openspec/changes/` dir, plus this phase's `progress.json`.
