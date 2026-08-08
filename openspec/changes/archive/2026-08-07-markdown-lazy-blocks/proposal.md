## Why

The shared markdown renderer currently leaves fenced code unhighlighted and cannot render Mermaid diagrams, while eagerly loading either renderer would worsen an already oversized initial frontend graph. C-09 adds finalized rich blocks without making partial streaming content unstable or sacrificing readable source when rendering fails.

## What Changes

- Render finalized `mermaid` fences through a lazy Mermaid block configured with `securityLevel: "strict"`.
- Render finalized non-Mermaid fences through a lazy Shiki block while showing the source `<pre>` during loading and after any error.
- Keep Mermaid and Shiki out of the initial module graph and emit explicit `vendor-mermaid` and `vendor-shiki` chunks.
- Give every asynchronous block an isolated error boundary, accessible source fallback, and text alternative.
- Keep KaTeX CSS imported once by the shared markdown entry point and rely on the package CSS's bundled font URLs instead of loading math assets per block.
- Preserve provider compatibility and realtime event/state contracts: this change only projects finalized markdown and does not alter provider payloads, AG-UI events, persistence, stores, or services.
- Record C-09 start and completion through canonical KBD change transitions; OpenSpec remains the implementation and verification surface.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-content-rendering`: add lazy finalized code and Mermaid rendering, strict diagram security, stable source fallbacks, and explicit asset/chunk requirements.

## Impact

- Affected frontend code: `frontend/src/shared/markdown/`, its focused tests, and `frontend/vite.config.ts` chunk grouping.
- Dependencies: add `mermaid` and `shiki`; retain `highlight.js` until the later C-14c dependency-retirement change, but remove it from the shared markdown path now.
- Runtime UX: finalized code gains themed syntax highlighting and finalized Mermaid fences gain diagrams without blank loading/error states.
- Provider and realtime compatibility: unchanged; the renderer receives the same markdown text and does not mutate streaming or durable state.
- Security boundary: untrusted Mermaid syntax is rendered only under strict Mermaid configuration, and renderer failures expose escaped source rather than library error markup.
