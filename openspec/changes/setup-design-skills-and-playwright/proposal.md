# setup-design-skills-and-playwright

## Why
Phase `direct-entity-migration-models` does both a data-layer migration AND a per-page UI rewrite. The rewrites must commit to a single visual contract or they'll drift. This change installs the design skills, adds the screenshot harness, and pins the aesthetic in a doc.

## What changes
- Install `pbakaus-impeccable` and `ui-ux-pro-max` Claude Code skills.
- Add `@playwright/test` as a frontend devDependency.
- New scripts: `test:visual`, `screenshot:<page>` in `frontend/package.json`.
- New file `docs/admin-aesthetic-spec.md` — the visual contract (terminal aesthetic, banned fonts, palette, density, motion, a11y).
- New CSS tokens in `frontend/src/app.css` under `:root[data-admin-theme="terminal"]`.
- Empty `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/` directory.

## Impact
Zero runtime behaviour change. Sets up tooling for changes 3+.
