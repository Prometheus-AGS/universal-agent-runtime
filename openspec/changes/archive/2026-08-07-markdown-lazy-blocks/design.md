## Context

C-08 established one `MarkdownBubble` and deliberately left fenced blocks as escaped source. C-09 is the Wave 3 boundary change that adds finalized Mermaid and Shiki projections without changing provider payloads, AG-UI events, persistence, stores, or services. The initial frontend graph already exceeds its eventual budget, so both libraries must remain behind `import()` edges and produce auditable named chunks.

Assistant-ui exposes the current message status to the bubble. Explicit `source` mode is finalized by definition; context mode is finalized only when the owning message status is no longer `running`. The installed assistant-ui API, current Mermaid 11 documentation, and current Shiki 4 documentation were inspected before selecting the seams below.

## Goals / Non-Goals

**Goals:**

- Keep a readable escaped source block present during streaming, lazy loading, unsupported-language handling, and renderer failure.
- Activate Mermaid and Shiki only for finalized fenced blocks.
- Isolate every asynchronous block with Suspense and a React error boundary.
- Configure Mermaid with `startOnLoad: false` and `securityLevel: "strict"`, sanitize its renderer-produced SVG before insertion, and expose diagram source as a text alternative.
- Render Shiki tokens as React text/span nodes so untrusted code never becomes injected highlighter HTML.
- Emit explicit `vendor-mermaid` and `vendor-shiki` chunks and prove neither is reachable from the initial entry.
- Preserve the shared KaTeX CSS import and package-managed font URL strategy.

**Non-Goals:**

- Changing the markdown AST/security chain, provider adapters, realtime events, PGlite records, or chat stores.
- Removing `highlight.js` from the manifest or unrelated legacy consumers; C-14c owns dependency retirement after C-09 is established.
- Adding chart, SVG-artifact, image, video, modal expansion, pan/zoom, or the full C-12 content-block catalog.
- Executing Mermaid directives, click handlers, external callbacks, or links.

## Decisions

### 1. Carry finalization through a render-phase context

`MarkdownBubble` reads the optional assistant message status. Explicit-source bubbles provide `finalized`; assistant bubbles provide `streaming` while `message.status.type === "running"`. The shared `code` component reads that context. During streaming it returns only the escaped source block, even when a fence is already syntactically closed.

Alternatives considered:

- Treat a closed fence as final: rejected because a provider can append to an already closed block and because the binding plan explicitly keys finalization to `TEXT_MESSAGE_END`.
- Add streaming state to stores or services: rejected because finalization is already represented by assistant-ui message state and the renderer must remain a pure projection.

### 2. Lazy-load block components through named dynamic entries

The static markdown component map uses `React.lazy` imports for `vendor-shiki.ts` and `vendor-mermaid.ts`. Each named dynamic entry re-exports one block component, and each block statically imports its engine, so neither library can enter the initial graph. The entry filenames give the emitted lazy chunks stable `vendor-shiki` and `vendor-mermaid` names without package-wide Rolldown groups. Package-wide groups are intentionally avoided: Mermaid and Shiki share transitive modules with the initial application, so forcing their full dependency trees into named groups creates circular chunks that Rolldown promotes into the entry imports.

Alternatives considered:

- Import engines in the shared component map: rejected because the initial graph would eagerly include them.
- Load from a CDN: rejected because it breaks local-first/offline operation and weakens dependency provenance.

### 3. Use one source fallback contract for loading, expected failures, and render crashes

`SourceCodeBlock` is the stable, escaped baseline. `LazyMarkdownBlockBoundary` wraps each finalized block and resets by source/language key. Suspense shows the baseline while the module loads; expected parse/tokenization errors show it with a plain-language status; unexpected React errors fall back through the boundary. A failed block is never blank.

Alternatives considered:

- A bubble-level boundary: rejected because one malformed block would replace otherwise valid sibling content.
- Spinner-only loading: rejected because it hides useful source and creates layout churn.

### 4. Mermaid is strict, finalized, theme-aware, and sanitized at insertion

The Mermaid module initializes with `startOnLoad: false`, `securityLevel: "strict"`, `theme: "base"`, and token-derived theme variables immediately before `mermaid.render`. It uses unique render IDs, re-renders on resolved theme changes, sanitizes the returned SVG through the existing DOMPurify SVG profile, and inserts only a non-empty sanitized result. The figure has an accessible label and a disclosure containing the diagram source; errors show source plus a concise parse status.

Alternatives considered:

- `securityLevel: "loose"`: rejected at the untrusted model/tool-content boundary because it enables interactive links and tags.
- Trust Mermaid output without the existing SVG sanitizer: rejected because the DOM insertion point is an actual trust boundary and the sanitizer already owns standalone SVG.

### 5. Shiki produces React tokens from a cached browser singleton

The lazy Shiki module uses the package bundle's async language/theme loading and cached shorthand highlighter. It tokenizes with a light/dark theme selected from the resolved application theme and returns plain token data. The component maps token content to React spans; code remains text, not HTML. Unknown or unsupported language identifiers degrade to plain source. Blocks longer than eight lines show line numbers, and block chrome provides copy and wrap controls.

Alternatives considered:

- `codeToHtml` plus `dangerouslySetInnerHTML`: rejected because it would introduce a second HTML insertion path for untrusted code.
- Load every grammar before first render: rejected because it increases first-block latency and memory use without an observed need.

### 6. Keep KaTeX CSS and fonts at the shared renderer entry

`katex/dist/katex.min.css` remains imported once by `markdown-bubble.tsx`. Vite resolves the stylesheet's package-relative font URLs into fingerprinted assets. No lazy block imports KaTeX CSS and no external font host is introduced.

## Risks / Trade-offs

- [Mermaid and Shiki are large even when lazy] → Keep explicit import boundaries, inspect the production manifest at this Wave 3 boundary, and reserve size enforcement for C-13's budget.
- [A theme change can leave stale async output] → Cancel stale effects and key rendering by resolved theme/source.
- [Mermaid uses global configuration] → Initialize immediately before each render with the same strict policy and current token theme; never expose caller-controlled configuration.
- [Unsupported Shiki grammar] → Show escaped source with its language label; unsupported syntax is not a fatal bubble error.
- [Assistant status is unavailable in standalone tests/previews] → Treat missing context as finalized, matching explicit read-only preview behavior.

## Migration Plan

1. Add `mermaid` and `shiki` in one frontend package transaction and update both maintained lockfiles.
2. Add source fallback, per-block boundary, render-phase context, and lazy block modules behind the shared renderer.
3. Add named dynamic facade entries and focused tests for streaming/finalization, strict Mermaid configuration, Shiki token output, failures, theme behavior, and import boundaries.
4. Run implementation checks, then the Wave 3 frontend test/build boundary and inspect emitted chunks/manifest.
5. Roll back by removing the two lazy dispatches and dependencies; escaped source remains the pre-existing safe behavior.

## Open Questions

None. C-09's library choice and security/finalization policies are binding in the phase plan.
