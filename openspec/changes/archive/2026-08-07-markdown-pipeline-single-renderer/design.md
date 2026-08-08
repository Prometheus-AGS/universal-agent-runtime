## Context

Chat message parts currently render through `MarkdownTextPrimitive` in `enhanced-markdown-text.tsx`, while the Skills editor preview creates a separate `ReactMarkdown` instance with only GFM. Both surfaces receive user- or agent-authored text, so raw HTML is an actual trust boundary. C-09 and later content surfaces also require a stable shared pipeline to extend.

The implementation must preserve assistant-ui's message-part context and deferred streaming behavior while also accepting an explicit markdown string for previews and other read-only surfaces. It must follow the repository's Flat 2.0 and strict frontend layering contracts without changing stores, services, provider APIs, or persisted realtime state.

## Goals / Non-Goals

**Goals:**

- Provide one public `MarkdownBubble` component and one shared plugin/component configuration.
- Support assistant-ui context rendering and explicit `source` rendering through that component.
- Parse GFM, model-style hard breaks, math, and limited raw HTML.
- Treat raw HTML as untrusted and remove executable elements, handler attributes, unsafe URLs, and unapproved presentation attributes.
- Preserve deferred assistant-ui rendering and safe external-link behavior.
- Render KaTeX with its packaged CSS and accessible MathML output.
- Remove the two old renderer entry points rather than retain compatibility aliases.

**Non-Goals:**

- Mermaid, Shiki, and finalized-only lazy block rendering (C-09).
- Migrating every remaining plain-text surface in the frontend; this change migrates the two existing markdown renderers and establishes the shared target.
- Raw SVG artifact rendering. C-08 provides the shared DOMPurify helper for that trust boundary, but the dedicated raw-artifact block is implemented by the later content-block work.
- Changes to provider adapters, AG-UI normalization, message persistence, or realtime graph state.

## Decisions

### One component with two input modes

`MarkdownBubble` accepts an optional `source`. With `source`, it renders `ReactMarkdown`; without it, it renders assistant-ui's `MarkdownTextPrimitive`. Both paths receive the same remark chain, rehype chain, and component map. This preserves assistant-ui context integration without forcing independent previews to fabricate message runtime context.

Alternative: expose separate `AssistantMarkdownBubble` and `SourceMarkdownBubble` components. Rejected because it recreates two public renderer entry points and makes plugin drift possible.

### Security plugins are one ordered unit

The rehype chain is exactly `rehypeRaw`, `[rehypeSanitize, schema]`, `rehypeKatex`. Raw parsing and sanitization are introduced together. The schema begins with `defaultSchema`, adds only the documented limited HTML/SVG vocabulary and explicit attributes/protocols, and does not allow `style`, `script`, `iframe`, `object`, event-handler attributes, or JavaScript URLs.

KaTeX input marker classes (`math-inline`, `math-display`, and existing `language-*`) are allowed through sanitization. KaTeX runs afterward as the trusted transform, matching upstream `rehype-sanitize` guidance and avoiding a broad MathML/style allowlist.

Alternative: sanitize after KaTeX. Rejected because it either strips required KaTeX output or requires broad style/MathML permissions that weaken the untrusted-input boundary.

### Shared presentation map, Flat 2.0 surfaces

The existing typography and code/table behavior move into `shared/markdown/markdown-components.tsx`. Existing border-based chrome is replaced with spacing and surface fills so the new shared path does not create fresh Flat 2.0 allowlist debt. External links force `target="_blank"` with `rel="noopener noreferrer"`.

### KaTeX CSS is package-owned

The shared renderer imports `katex/dist/katex.min.css`, so CSS and font URLs remain version-matched to the installed package and do not depend on a CDN. C-09 may further tune chunking, but C-08's math output is usable on delivery.

## Risks / Trade-offs

- **[Risk] Allowing limited raw SVG/HTML enlarges the parser surface.** → Keep `rehype-sanitize` immediately after raw parsing and cover known XSS vectors with DOM assertions.
- **[Risk] Two internal rendering primitives could drift.** → Export only `MarkdownBubble`; keep plugin arrays and component maps in shared modules imported by both internal branches.
- **[Risk] KaTeX increases the initial markdown bundle.** → Accept for the C-08 correctness boundary; C-09 owns expensive-block lazy-loading and bundle strategy.
- **[Risk] The old markdown renderer's imperative highlight.js HTML assignment does not belong inside the new sanitizer boundary.** → Render fenced code as safe plain text in C-08 while keeping block/inline presentation distinct; C-09 installs the planned Shiki renderer before the legacy dependency is removed.
- **[Risk] Not every text surface is migrated in this change.** → The public shared renderer becomes the only allowed target; later feature migrations consume it as their surfaces move.

## Migration Plan

1. Add the complete markdown security/math dependency set in one lockfile update.
2. Add shared plugin, sanitizer-schema, component-map, renderer, and focused XSS/math tests.
3. Update chat and Skills preview imports/usages, then remove the old chat renderer file and direct Skills `ReactMarkdown` imports.
4. Run focused tests and cheap frontend gates; verify no second markdown renderer remains.
5. Roll back all renderer, dependency, and consumer edits together if validation fails so no raw-without-sanitize state can exist.

## Open Questions

None. Plugin order, capability name, source path, and the C-08/C-09 boundary are fixed by the phase plan.
