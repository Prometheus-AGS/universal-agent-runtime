## 1. Skill installs
- [ ] 1.1 `/plugin marketplace add pbakaus/impeccable`
- [ ] 1.2 `/plugin install impeccable`
- [ ] 1.3 Install `ui-ux-pro-max` (verify exact marketplace name during execute)
- [ ] 1.4 Confirm via `~/.claude/plugins/installed_plugins.json`

## 2. Playwright
- [ ] 2.1 `pnpm --filter ./frontend add -D @playwright/test`
- [ ] 2.2 `pnpm --filter ./frontend exec playwright install chromium`
- [ ] 2.3 Add `frontend/playwright.config.ts` (baseURL 127.0.0.1:8088, 1440×900, headless)
- [ ] 2.4 Add `test:visual` script in `frontend/package.json`

## 3. Aesthetic spec
- [ ] 3.1 Author `docs/admin-aesthetic-spec.md` — contract from plan.md §Aesthetic Pin
- [ ] 3.2 Append terminal CSS tokens to `frontend/src/app.css`
- [ ] 3.3 Add `<html data-admin-theme="terminal">` toggle in `frontend/src/main.tsx` (only when route starts with `/admin`)

## 4. Screenshot dir
- [ ] 4.1 `mkdir -p .kbd-orchestrator/phases/direct-entity-migration-models/screenshots`
- [ ] 4.2 `mkdir -p .kbd-orchestrator/phases/direct-entity-migration-models/audits`

## 5. Verification
- [ ] 5.1 `pnpm --filter ./frontend exec playwright --version` succeeds
- [ ] 5.2 `pnpm --filter ./frontend test` → 36/36
- [ ] 5.3 `pnpm --filter ./frontend build` clean
