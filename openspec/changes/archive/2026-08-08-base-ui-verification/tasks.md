# Tasks: base-ui-verification

## Task 1 — Run full Radix import audit
- [x] Run: `grep -r "@radix-ui" frontend/src/ --include="*.tsx" --include="*.ts"`
  - Expected: zero results in own code
  - If non-zero: block — fix stragglers before continuing
- [x] Run: `grep -r "asChild" frontend/src/` — must be zero
- [x] Run: `grep -r "from '@radix-ui/react-slot'" frontend/src/` — must be zero

## Task 2 — Audit cmdk transitive dependencies
- [x] Run: `pnpm why cmdk` — confirmed pre-remediation `cmdk@1.1.1`
- [x] Run: `pnpm why @radix-ui/react-dialog` — confirmed cmdk as a pre-remediation source
- [x] Inspect `frontend/node_modules/cmdk/package.json` dependencies — four Radix packages recorded in `dependency-audit.md`
- [x] **If cmdk imports @radix-ui**: proceed to Task 3 — condition met
- [x] **If cmdk is Radix-free**: N/A — condition not met

## Task 3 — Replace cmdk with Base UI Combobox (conditional on Task 2)
- [x] Remove cmdk: `pnpm remove cmdk`
- [x] Rewrite `frontend/src/components/ui/command.tsx` using `@base-ui/react/autocomplete` per the current search/action guidance recorded in `design.md`
- [x] Maintain same exported names: `Command`, `CommandDialog`, `CommandInput`, `CommandList`,
      `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandSeparator`, `CommandShortcut`
- [x] Find all files importing from `@/components/ui/command` and verify zero changes needed
  - Run: `grep -r "from.*components/ui/command" frontend/src/ --include="*.tsx"`
- [x] Browser verification: the application command palette filters to Providers and
      selects with Enter; the chat agent-selector command facade filters and selects
      with Enter (`runtime-console-visual.spec.ts`, `chat-agent-selection.spec.ts`)

## Task 4 — Audit assistant-ui transitive dependencies
- [x] Run: `pnpm why @radix-ui/react-dialog` — full ownership documented
- [x] If `@assistant-ui/react` is listed as source:
  - [x] Check `@assistant-ui/react` latest version for Base UI support — current 0.15.10 still declares Radix runtime dependencies
  - [x] Base UI–compatible runtime version: N/A; no upgrade performed because the current release does not remove Radix
  - [x] Create `frontend/docs/third-party-radix-note.md` documenting the situation
- [x] Check `@prometheus-ags/prometheus-entity-management` for Radix transitive deps too:
  - [x] Run: `pnpm why @radix-ui/react-tooltip` — output documented; PEM is not an owner

## Task 5 — Full TypeScript check
- [x] Run from `frontend/`: `pnpm typecheck`
- [x] Must return zero errors

## Task 6 — Lint check
- [x] Run from `frontend/`: `pnpm lint`
- [x] Must return zero errors

## Task 7 — E2E smoke tests
- [x] Run: `pnpm test:e2e` — 36 passed, 4 skipped, 8 failed in the no-backend profile
- [x] Document failures — classified in `verification.md`; real-backend specs, two
      pre-existing replay expectations, two stale async guard checks, and one test run
      under the wrong performance profile were unrelated to the command migration
- [x] Fix any migration-introduced failures before proceeding — no command-facade
      failure remained; the agent-selector test was made deterministic and its two
      focused browser cases pass

## Task 8 — Acceptance smoke evidence

No human manual-interaction receipt is claimed. Repeatable automated evidence replaces
the original manual checklist where available; real-backend gaps remain classified in
`verification.md` for C-15.
- [x] **Chat flow**: repeatable chat-send/stream contracts pass in the full Vitest suite;
      the migrated selector facade is exercised in the browser agent-selector flow
- [x] **Agent selector**: deterministic browser test opens, filters two agents, selects
      with Enter, updates the trigger, and closes the popover
- [x] **Admin settings form**: settings composition/structure and shared form
      validation/submission contracts pass in the full Vitest suite and settings browser smoke
- [x] **Sidebar mobile**: existing runtime-console mobile browser smoke passes
- [x] **Combobox/Command**: browser command palette filters and selects with Enter;
      pointer, Enter, filtering, and empty-state facade tests pass
- [x] **Dialog**: Base UI dialog close contracts remain covered by the existing primitive
      test suite and browser command/mobile-dialog flows
- [x] **Select**: existing Base UI composition test selects with keyboard navigation

## Task 9 — Cleanup backup
- [x] `frontend/src/components/ui-radix-backup/` was already absent; no deletion was needed
- [x] No cleanup commit was created because the operator did not authorize staging or commits

## Task 10 — Final audit summary
- [x] Final audit summary:
  - Whether cmdk was replaced (yes/no)
  - Whether assistant-ui brings transitive Radix (yes/no, version)
  - Whether prometheus-entity-management brings transitive Radix (yes/no)
  - `pnpm why @radix-ui/react-dialog` final output

  `cmdk` was replaced and is absent from the manifest, root and nested lockfiles, and
  root and nested dependency graphs.
  `@assistant-ui/react@0.14.26` still brings Radix through `radix-ui@1.6.2`;
  `@prometheus-ags/prometheus-entity-management@3.0.0-alpha.0` brings none. Final
  Dialog ownership is `vaul@1.1.2` (1.1.15 in the nested graph, deduplicated to
  1.1.19 in the root graph) and `@assistant-ui/react@0.14.26 -> radix-ui@1.6.2 ->
  @radix-ui/react-dialog@1.1.19`.

## Completion Gate
All change-scoped tasks checked. Migration-sensitive E2E passes; unrelated broad-suite
failures are retained as explicit evidence for C-15. No commit was created because the
operator did not authorize repository history changes.
