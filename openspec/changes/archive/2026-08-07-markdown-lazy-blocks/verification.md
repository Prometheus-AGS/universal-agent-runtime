# Verification: markdown-lazy-blocks

## Verdict

Implementation evidence satisfies all six C-09 requirements and fourteen scenarios. Artifact refinement converged in four iterations, and final isolated review passed with its sole warning resolved.

## Summary

| Dimension | Status |
|---|---|
| Completeness | 14/14 tasks complete; 6/6 requirements mapped |
| Correctness | 6/6 requirements and 14/14 scenarios covered by implementation and deterministic evidence |
| Coherence | Design decisions followed; shared renderer layering and project conventions preserved |

No critical, warning, or suggestion issues remain. All checks passed; the change is ready for canonical completion and archive.

## Goal-backward evidence

### Finalization and failure isolation

- `MarkdownBubble` derives `streaming` from an assistant message whose status is `running`; explicit source mode is finalized.
- Streaming fences return escaped `SourceCodeBlock` content and never mount a lazy engine component.
- Each finalized fence owns a resettable React error boundary and Suspense source fallback.
- Focused renderer and boundary tests cover explicit and assistant contexts, transition to final, a controllably pending/rejected lazy module, render failure, source fallback, sibling isolation, and plain markdown.

### Mermaid security and accessibility

- Mermaid is imported only by its lazy block and initialized with `startOnLoad: false`, `securityLevel: "strict"`, and secured policy keys.
- Renderer SVG crosses the DOM boundary only after the existing DOMPurify standalone-SVG profile; an empty sanitized result fails to escaped source.
- The rendered diagram has an accessible image name and a native disclosure retaining original source.
- Focused tests cover strict configuration, sanitization, theme refresh, successful output, parse failure, and empty sanitized output.

### Shiki source safety and theme behavior

- Shiki tokenizes through `shiki/bundle/full`; token content is mapped to React text/span nodes with no highlighter HTML insertion.
- Supported syntax refreshes with the resolved light/dark theme; unknown languages and errors preserve escaped source.
- Focused tests cover token output, theme refresh, unsupported languages, line numbering, copy, and wrap.

### Production import graph and math assets

- `frontend/package.json` declares `mermaid ^11.16.1` and `shiki ^4.4.2`. Both maintained lockfiles resolve Mermaid 11.16.1 and Shiki 4.4.2 from matching importer specifiers.
- `pnpm -C frontend install --frozen-lockfile --ignore-scripts` and root `pnpm install --frozen-lockfile --ignore-scripts` both report the lockfile is current and pass supply-chain policy checks.
- `vendor-shiki.ts` and `vendor-mermaid.ts` are the entry's two named dynamic engine imports.
- The build plugin emits package-relative chunk-to-module metadata from Rolldown's `chunk.modules`. `scripts/check-markdown-lazy-chunks.mjs` traverses emitted chunk imports from the production entry, rejects any Mermaid/Shiki module ID in that static closure, requires both dynamic entries, verifies their emitted filenames, confirms each named entry owns engine modules, and rejects absolute build-host paths.
- `static/index.html` has no Mermaid/Shiki module preload.
- KaTeX CSS remains imported exactly once from `markdown-bubble.tsx`; lazy blocks add no KaTeX import or remote math font.

## Commands run

- `pnpm -C frontend typecheck` — pass.
- `pnpm -C frontend lint` — pass.
- `node scripts/check-frontend-boundaries.mjs` — pass, zero production violations.
- `node scripts/check-flat2-style.mjs` — pass, 391 tracked legacy and zero new violations.
- `pnpm -C frontend exec vitest run` — pass, 45 files / 214 tests.
- `pnpm -C frontend build` — pass.
- `pnpm -C frontend exec vite build --manifest` — pass.
- Frontend and root frozen-lockfile installs — pass; both importers resolve Mermaid 11.16.1 and Shiki 4.4.2.
- `node scripts/check-markdown-lazy-chunks.mjs` — pass against Vite manifest plus emitted module graph, with zero forbidden static engine modules, zero missing dynamic entries, and zero invalid names.
- `openspec validate markdown-lazy-blocks --strict` — pass.
- Artifact-refiner manifest and constraints schemas — pass.
- `git diff --check` — pass.

## Review evidence

- Final fresh-context harness review verdict: PASS with 0 critical / 1 warning / 0 suggestion findings.
- Anti-sycophancy gate: pass at 0.0803571417927742 under strict screening.
- The warning identified raw Rolldown module identifiers as a potential build-host path disclosure. The emitted graph now normalizes identifiers to package-relative paths, and the production checker rejects any absolute module identifier; the post-fix graph reports `absoluteModuleIds: []`.
- The configured external review endpoints timed out, so the final receipt uses harness-native fresh-context isolation with a disclosed same-model collision rather than claiming cross-model independence.

## Known external conditions

The production build retains pre-existing PGlite direct-`eval` diagnostics and the known initial large-chunk warning assigned to later budget work. Neither named engine entry is part of the initial static graph.

The final frozen install exposed Vitest's missing repository-root filesystem allowance for root-hoisted PGlite and Storybook package assets. The test config now mirrors the existing production Vite allowance; this is the observed Wave 3 gate repair, not speculative scope.
