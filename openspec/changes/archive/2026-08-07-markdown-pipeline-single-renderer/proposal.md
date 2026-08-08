## Why

UAR currently has two markdown renderers with different plugin chains, and neither provides the complete raw-HTML sanitization and math contract required for untrusted agent output. Consolidating them now establishes the shared content boundary required by the later lazy-block, trace, and rich-content changes.

## What Changes

- Introduce one shared `MarkdownBubble` renderer for assistant-ui message parts and explicit markdown sources.
- Apply one remark chain for GFM, model-style line breaks, and math.
- Apply `rehype-raw` immediately followed by a restrictive `rehype-sanitize` schema, then render trusted math nodes with KaTeX.
- Add XSS regression fixtures covering scripts, event handlers, unsafe protocols, unsafe elements, and unsafe presentation attributes.
- Replace the Skills editor's direct `react-markdown` renderer and the chat-specific enhanced renderer with the shared component.
- Add the markdown pipeline dependencies and preserve the existing provider-neutral message/realtime data contracts.

## Capabilities

### New Capabilities

- `frontend-content-rendering`: Defines the single-renderer, plugin-chain, sanitization, math, and cross-surface content-rendering contract.

### Modified Capabilities

None.

## Impact

- Affects `frontend/src/shared/markdown/`, the assistant chat thread renderer, the Skills markdown preview, frontend dependency metadata, and focused renderer tests.
- Runtime UX gains consistent GFM, hard breaks, math, and safe limited HTML across migrated markdown surfaces.
- Provider compatibility is unchanged because all provider/model text continues to enter through the same string/assistant-ui interfaces.
- Realtime state and persistence schemas are unchanged; streaming message text is rendered with the shared chain and deferred assistant-ui parsing.
- KBD change `C-08` must transition to complete only after focused security tests, frontend gates, OpenSpec verification, and the per-change quality gate pass.
