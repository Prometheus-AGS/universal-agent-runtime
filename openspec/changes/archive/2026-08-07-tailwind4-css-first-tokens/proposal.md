## Why

The frontend still builds through Tailwind 3, a JavaScript Tailwind config, and PostCSS even though the approved migration requires Tailwind 4's CSS-first token contract. C-02 establishes that foundation before the Flat 2.0 gate and later call-site migration, while reversing the superseded 2026-07-21 decision to retain HSL-channel tokens as the long-term design source.

## What Changes

- Upgrade `tailwindcss` and add `@tailwindcss/vite` at the exact current phase pin, `4.3.3`.
- Converge the frontend and root workspaces on Vite 8.1.4 so the frontend and its A2UI/Storybook/Vitest packages share one Vite 8 peer graph.
- Register the Tailwind Vite plugin and move the utility/theme contract into `frontend/src/shared/theme/tokens.css` using `@import "tailwindcss"` and `@theme`/`@theme inline`.
- Port the complete-color KnowMe surface, text, role, run-phase, typography, radius, easing, and duration tokens while retaining staged compatibility aliases for call sites assigned to C-05.
- Delete `frontend/tailwind.config.ts` and `frontend/postcss.config.js` after their live configuration is represented in CSS.
- Clear the Tailwind config path in `frontend/components.json` and repoint Storybook visual-regression path filters at `frontend/src/shared/theme/tokens.css`.
- Verify there are no live dangling references to the deleted configuration files.

## Capabilities

### New Capabilities

- `frontend-design-system`: Defines the CSS-first Tailwind foundation, stable UAR token vocabulary, theme compatibility, and build/configuration invariants for downstream UI migration.

### Modified Capabilities

None.

## Impact

- **Runtime UX:** Theme utility generation moves to the approved CSS-first token source; dark, light, and high-contrast semantics must remain equivalent during this plumbing change.
- **Provider compatibility:** No provider, model-routing, API, or transport behavior changes.
- **Realtime state:** No AG-UI, A2UI, entity-graph, store, or persistence behavior changes.
- **Dependencies/build:** Frontend development moves from Tailwind 3/PostCSS integration to exact Tailwind 4.3.3 plus `@tailwindcss/vite` 4.3.3. Both workspace roots pin Vite 8.1.4 so frontend, A2UI, Storybook, and Vitest peers resolve one compiler version. Legacy configuration files are removed.
- **KBD workflow state:** C-02 start and completion are recorded through canonical KBD change transitions.
- **Deferred work:** C-02 does not rewrite the 337 `hsl(var())` call sites or perform the Flat 2.0 purge; C-05 and later page migrations own those edits.
