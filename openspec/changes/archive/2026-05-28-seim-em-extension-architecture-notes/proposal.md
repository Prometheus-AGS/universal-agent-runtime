# Proposal: Chrome Extension Architecture Notes (W7b)

## Why
The Entity Explorer panel is designed to run both injected into any React app
AND as a Chrome MV3 DevTools panel. Before scaffolding the extension (W8),
we need a concise architecture document covering:

- How a Chrome MV3 devtools extension communicates with the inspected page
- The message bridge pattern (devtools page → background → content script → page)
- How the `DevtoolsEventBus` fits into that pipeline
- Which pieces of the existing library need adaptation for the extension context

## What Changes
Docs-only: `docs/extension-architecture-notes.md` committed to the entity-management
worktree.

## Capabilities
- chrome-mv3-extension-architecture

## Impact
- Zero runtime changes.
- Gates W8 scaffold with a concrete architecture reference.
