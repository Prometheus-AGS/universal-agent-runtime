# Implementation Tasks — ssed-entity-explorer-browser-extension (STRETCH)

> Status per phase plan: **stretch / v2**. This change ships the **manifest, bridge contract, scaffolding, and architecture notes**. The substantial extension UI work + extension testing is deferred to a focused implementation session.

## 1. Pre-flight (per change 8 routing discipline)

- [ ] 1.1 Web search "Chrome MV3 devtools panel patterns 2026" — capture URLs + anchor keywords
- [ ] 1.2 Web search "react-devtools bridge architecture" — capture URLs + anchor keywords
- [ ] 1.3 Web search "postMessage origin validation security best practices" — capture URLs + anchor keywords
- [ ] 1.4 Write `prometheus-entity-management/chrome-extension/docs/architecture-notes.md` summarising the synthesis

## 2. Package scaffold — `prometheus-entity-management/chrome-extension/`

- [ ] 2.1 `package.json` declaring its own scripts + workspace membership
- [ ] 2.2 `tsconfig.json` for the extension's TS targets
- [ ] 2.3 Build pipeline (vite / tsup) producing the extension bundle into `dist/`
- [ ] 2.4 `manifest.json` with MV3 + devtools_page + content_scripts entries
- [ ] 2.5 `public/icons/` placeholder PNGs (16/48/128)

## 3. Bridge module — `src/bridge/`

- [ ] 3.1 `envelope.ts` — exhaustive `BridgeEnvelope` union with discriminator + nonce
- [ ] 3.2 `page-to-extension.ts` — page-side helpers (called by host app's `EntityExplorerFab`)
- [ ] 3.3 `extension-to-page.ts` — extension-side `createBridgeDataSource` returning a `DevtoolsDataSource`
- [ ] 3.4 Nonce-matching + drop-unmatched logic
- [ ] 3.5 Origin verification (`event.source === window`, ignore cross-origin iframes)

## 4. Page-hook + content-script

- [ ] 4.1 `src/page-hook.ts` — installs `window.__PROMETHEUS_DEVTOOLS_HOOK__` (injected into MAIN world)
- [ ] 4.2 `src/content-script.ts` — relays page ↔ extension; emits `__PROMETHEUS_DEVTOOLS_PRESENT__` CustomEvent on page load to flip `isExtensionPresent`
- [ ] 4.3 `chrome.scripting.executeScript({ world: "MAIN", files: ["page-hook.js"] })` from content-script
- [ ] 4.4 Per-tab scoping via `chrome.devtools.inspectedWindow.tabId`

## 5. Panel host — `src/panel.tsx`

- [ ] 5.1 Mounts `EntityExplorerPanel` from `@prometheus-ags/prometheus-entity-management`
- [ ] 5.2 Wires the bridge `DevtoolsDataSource` (defined in change 10's `src/devtools/data-source.ts`)
- [ ] 5.3 Connection-state UI (connecting / connected / disconnected / waiting-for-host-hook)

## 6. Host-side hook installation (in `prometheus-entity-management/src/devtools/EntityExplorerFab.tsx`)

- [ ] 6.1 On mount in dev mode: install `window.__PROMETHEUS_DEVTOOLS_HOOK__` with documented surface
- [ ] 6.2 Listen for `__PROMETHEUS_DEVTOOLS_PRESENT__` CustomEvent; set `isExtensionPresent = true`
- [ ] 6.3 Production tree-shake: install code guarded by `process.env.NODE_ENV !== "production"`

## 7. README / dev side-load

- [ ] 7.1 `chrome-extension/README.md` documenting: build steps (`pnpm build`), side-load via `chrome://extensions` Developer Mode, expected console output, debugging tips
- [ ] 7.2 Cross-link to the in-app explorer (change 10) so devs know which surface to use when

## 8. Tests

- [ ] 8.1 Unit tests on the bridge envelope types (TS type assertions + runtime discriminator checks)
- [ ] 8.2 Unit tests on nonce matching
- [ ] 8.3 Manual smoke: load extension, open DevTools, navigate to a Prometheus-entity-management-using page, confirm the panel shows entities (script the QA pass in the README)

## 9. Production bundle audit

- [ ] 9.1 Verify production build of `prometheus-entity-management` does NOT include the `__PROMETHEUS_DEVTOOLS_HOOK__` installation
- [ ] 9.2 Verify the `chrome-extension/` package is not pulled into the main package's published bundle

## 10. Cross-repo PRs

- [ ] 10.1 `prometheus-entity-management` PR carrying the new package + host-side hook installation
- [ ] 10.2 (No skill-system PR — extension is documented in the existing entity-graph-optimize "Dev-mode entity explorer" subsection from change 10)

```
prometheus-entity-management commit: <fill in after merge>
PR URL:                              https://github.com/Prometheus-AGS/prometheus-skill-system/pull/3
```

## 11. Closeout

- [ ] 11.1 **All §1-9 deferred** to a dedicated implementation session — see proposal "Stretch status" + design D9. This change archives with scaffolding intent + spec contract.
- [ ] 11.2 `/opsx:verify` will flag the entire implementation as deferred CRITICAL — proceed to archive with documented stretch status
- [ ] 11.3 progress.json `changes_completed: 11` with all_done; active_change → null; phase ready for `/kbd-reflect`
