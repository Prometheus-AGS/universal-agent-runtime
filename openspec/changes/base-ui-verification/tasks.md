# Tasks: base-ui-verification

## Task 1 — Run full Radix import audit
- [ ] Run: `grep -r "@radix-ui" frontend/src/ --include="*.tsx" --include="*.ts"`
  - Expected: zero results in own code
  - If non-zero: block — fix stragglers before continuing
- [ ] Run: `grep -r "asChild" frontend/src/` — must be zero
- [ ] Run: `grep -r "from '@radix-ui/react-slot'" frontend/src/` — must be zero

## Task 2 — Audit cmdk transitive dependencies
- [ ] Run: `pnpm why cmdk` — confirm it is installed
- [ ] Run: `pnpm why @radix-ui/react-dialog` — check if cmdk is listed as a source
- [ ] Run: `cat frontend/node_modules/cmdk/package.json | grep -A5 '"dependencies"'`
- [ ] **If cmdk imports @radix-ui**: proceed to Task 3
- [ ] **If cmdk is Radix-free**: mark Task 3 as N/A, proceed to Task 4

## Task 3 — Replace cmdk with Base UI Combobox (conditional on Task 2)
- [ ] Remove cmdk: `pnpm remove cmdk`
- [ ] Rewrite `frontend/src/components/ui/command.tsx` using `@base-ui/react/combobox`
- [ ] Maintain same exported names: `Command`, `CommandDialog`, `CommandInput`, `CommandList`,
      `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandSeparator`, `CommandShortcut`
- [ ] Find all files importing from `@/components/ui/command` and verify zero changes needed
  - Run: `grep -r "from.*components/ui/command" frontend/src/ --include="*.tsx"`
- [ ] Manual test: command palette opens, filters items, selects correctly, keyboard nav works

## Task 4 — Audit assistant-ui transitive dependencies
- [ ] Run: `pnpm why @radix-ui/react-dialog` — document full output
- [ ] If `@assistant-ui/react` is listed as source:
  - [ ] Check `@assistant-ui/react` latest version for Base UI support
  - [ ] If Base UI–compatible version exists: upgrade with `pnpm add @assistant-ui/react@latest`
  - [ ] If not: create `frontend/docs/third-party-radix-note.md` documenting the situation
- [ ] Check `@prometheus-ags/prometheus-entity-management` for Radix transitive deps too:
  - [ ] Run: `pnpm why @radix-ui/react-tooltip` — document output

## Task 5 — Full TypeScript check
- [ ] Run from `frontend/`: `pnpm typecheck`
- [ ] Must return zero errors

## Task 6 — Lint check
- [ ] Run from `frontend/`: `pnpm lint`
- [ ] Must return zero errors (fix any auto-fixable issues with `pnpm lint --fix`)

## Task 7 — E2E smoke tests
- [ ] Run: `pnpm test:e2e`
- [ ] Document any failures — classify as (a) pre-existing or (b) migration-introduced
- [ ] Fix any migration-introduced failures before proceeding

## Task 8 — Manual smoke tests
- [ ] **Chat flow**: Send a message via the chat interface — streaming works, model-selector opens
- [ ] **Agent selector**: Dropdown opens, agents list, selection updates UI
- [ ] **Admin settings form**: Open admin → settings, fill form, submit, validation errors appear
- [ ] **Sidebar mobile**: Resize to mobile viewport, sidebar toggles open/close via sheet
- [ ] **Combobox/Command**: Open command palette (if present), type to filter, select item
- [ ] **Dialog**: Open any dialog, close via X button and via backdrop click
- [ ] **Select**: Open a Select dropdown, navigate with keyboard, select option

## Task 9 — Cleanup backup
- [ ] After all tests pass: `rm -rf frontend/src/components/ui-radix-backup/`
- [ ] Commit cleanup: `chore: remove radix-backup after successful base-ui migration`

## Task 10 — Final audit summary
- [ ] Write a summary comment here documenting:
  - Whether cmdk was replaced (yes/no)
  - Whether assistant-ui brings transitive Radix (yes/no, version)
  - Whether prometheus-entity-management brings transitive Radix (yes/no)
  - `pnpm why @radix-ui/react-dialog` final output

## Completion Gate
All tasks checked. E2E passes. Commit: `feat: complete base-ui migration — verification and cleanup`
