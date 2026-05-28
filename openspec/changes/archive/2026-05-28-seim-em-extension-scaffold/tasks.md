# Tasks: seim-em-extension-scaffold

## 1. Add DevtoolsEventBus.inject() method

- [x] 1.1 In `src/devtools-event-bus.ts`, add `inject(event: DevtoolsEvent): void` to the
  `DevtoolsEventBus` interface and implementation (calls the internal `handleEngineEvent`)
- [x] 1.2 Run `pnpm test` — all tests pass

## 2. Add external bus + forceOpen props

- [x] 2.1 In `src/ui/entity-explorer/context.tsx`, add optional `bus?: DevtoolsEventBus`
  prop to `EntityExplorerProviderProps`; when provided, skip internal bus creation
- [x] 2.2 In `src/ui/entity-explorer/panel.tsx`, add optional `forceOpen?: boolean` prop
  to `EntityExplorerPanel`; when true, render panel visible regardless of `state.open`
  and skip portal (render inline)
- [x] 2.3 Export `forceOpen` support from index (no new exports needed — just prop addition)
- [x] 2.4 Run `pnpm test` — all tests pass

## 3. Create extension scaffold files

- [x] 3.1 `mkdir -p ~/.claude/worktrees/seim-entity-management/extension`
- [x] 3.2 `mkdir -p ~/.claude/worktrees/seim-entity-management/src/extension`
- [x] 3.3 Create `extension/manifest.json` (MV3, permissions: scripting + activeTab + tabs)
- [x] 3.4 Create `extension/background.js` (service worker: port relay only)
- [x] 3.5 Create `extension/content.js` (window.postMessage → chrome.runtime relay)
- [x] 3.6 Create `extension/devtools.html` (minimal HTML loading devtools.js)
- [x] 3.7 Create `extension/devtools.js` (chrome.devtools.panels.create)
- [x] 3.8 Create `extension/panel.html` (React root)
- [x] 3.9 Create `src/extension/create-extension-bus.ts` (wraps createDevtoolsEventBus + port wiring)

## 4. Verify .npmignore / package.json files exclusion

- [x] 4.1 Confirm `extension/` and `src/extension/` are not in `package.json` `files` array
  (they default to excluded since only `dist`, `README.md`, `CHANGELOG.md` are listed)

## 5. Run typecheck

- [x] 5.1 `pnpm run typecheck` — zero errors

## 6. Commit

- [x] 6.1 `git add extension/ src/extension/ src/devtools-event-bus.ts src/ui/entity-explorer/context.tsx src/ui/entity-explorer/panel.tsx`
- [x] 6.2 Commit: `feat(extension): Chrome MV3 extension scaffold + bus.inject() (W8)`

## 7. Update progress.json + archive

- [x] 7.1 Update progress.json: changes_completed: 9 (or 10 if W7a+W7b counted separately)
- [x] 7.2 Archive this change
