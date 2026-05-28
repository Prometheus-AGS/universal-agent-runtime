# Proposal: Chrome MV3 Extension Scaffold (W8)

## Why
The Entity Explorer panel is most valuable as a Chrome DevTools extension that
can inspect any production or staging app using the entity management library
without modifying the host app's codebase (beyond adding `enableWindowBridge`).

W7b established the architecture. W8 creates the working scaffold so the extension
can be loaded as an unpacked extension in Chrome DevTools.

## What Changes
Creates `extension/` directory inside the entity-management worktree with:
- MV3 `manifest.json`
- Content script (`content.js`)
- Service worker (`background.js`)
- DevTools page (`devtools.html` + `devtools.js`)
- Panel React app (`panel.html` + `panel.tsx`)
- `src/extension/create-extension-bus.ts` — `inject()` method + port wiring
- `DevtoolsEventBus.inject()` method added to bus interface + implementation

## Capabilities
- extension-manifest
- extension-messaging-bridge
- extension-panel-react-app
- extension-bus-inject

## Impact
- Adds `extension/` directory (not published to npm; excluded via `files` in package.json)
- Adds `DevtoolsEventBus.inject()` method to the bus interface (additive, non-breaking)
- Consumers who don't use the extension are unaffected
