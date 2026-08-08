## 1. Dependency and Build Graph

- [x] 1.1 Add current `mermaid` and `shiki` dependencies in one frontend package transaction and synchronize both maintained lockfiles.
- [x] 1.2 Add explicit dynamic facade entries for `vendor-mermaid` and `vendor-shiki` while leaving legacy Highlight.js retirement to C-14c.

## 2. Finalized Lazy Blocks

- [x] 2.1 Add render-phase context, escaped `SourceCodeBlock`, and a resettable per-block Suspense/error-boundary contract to the shared markdown renderer.
- [x] 2.2 Add the lazy Shiki block with cached async tokenization, light/dark theme refresh, React text/span output, long-block line numbers, copy, wrap, and source degradation.
- [x] 2.3 Add the lazy Mermaid block with finalized-only dispatch, strict/start-disabled configuration, token-derived theme variables, sanitized SVG insertion, source disclosure, and parse fallback.
- [x] 2.4 Preserve the single shared KaTeX stylesheet import and verify lazy block modules introduce no KaTeX CSS or remote font path.

## 3. Focused Evidence

- [x] 3.1 Add focused tests for explicit and assistant contexts, streaming-to-finalized transitions, lazy loading, block isolation, and source fallbacks.
- [x] 3.2 Add focused Mermaid tests for strict configuration, sanitization, accessible alternatives, theme refresh, and parse failure.
- [x] 3.3 Add focused Shiki tests for token output without injected HTML, theme refresh, unsupported languages, line numbers, copy, and wrap.
- [x] 3.4 Add deterministic graph assertions for dynamic-only engine imports, named chunk configuration, and the single KaTeX CSS owner.

## 4. Wave 3 Verification and Handoff

- [x] 4.1 Pass frontend typecheck, lint, architecture boundaries, Flat 2.0 gate, focused tests, strict OpenSpec validation, and diff integrity.
- [x] 4.2 At the Wave 3 boundary, pass the full frontend test/build sequence and verify the production manifest keeps both vendor chunks outside the initial entry.
- [x] 4.3 Complete artifact-refiner constraint validation, manual accessibility/visual critique and polish fallback, and isolated adversarial review with all blocking findings resolved.
- [x] 4.4 Prepare the canonical C-09 completion transition and verified OpenSpec archive handoff.
