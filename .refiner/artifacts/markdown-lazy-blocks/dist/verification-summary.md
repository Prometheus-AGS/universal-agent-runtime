# C-09 Artifact Refinement Verification

## Scope

Artifact `markdown-lazy-blocks` is a `direct:code` refinement of the shared Markdown projection. Its blocking authority is the C-09 OpenSpec delta because this project has no `.kbd-orchestrator/constraints.md`.

## Delta found during refinement

The first production build disproved the proposed package-wide Rolldown grouping: Mermaid and Shiki share transitive modules with the initial application, which created circular vendor chunks and promoted both engine groups into the entry imports. The implementation now uses named dynamic facade entries instead. This retains auditable `vendor-mermaid` and `vendor-shiki` filenames without forcing shared dependency trees into engine groups.

## Deterministic evidence

- `pnpm -C frontend typecheck`: pass.
- `pnpm -C frontend lint`: pass.
- `node scripts/check-frontend-boundaries.mjs`: pass, zero production violations.
- `node scripts/check-flat2-style.mjs`: pass, 391 tracked legacy violations and zero new violations.
- `pnpm -C frontend exec vitest run`: pass, 45 files and 214 tests.
- Frontend and root frozen-lockfile installs: pass; both maintained importers resolve Mermaid 11.16.1 and Shiki 4.4.2.
- `pnpm -C frontend build`: pass.
- `pnpm -C frontend exec vite build --manifest`: pass; diagnostic manifest emitted.
- `node scripts/check-markdown-lazy-chunks.mjs`: pass. It recursively traverses `imports`, rejects forbidden engine entries in the static closure, requires both dynamic entries, verifies their emitted names and module ownership, and rejects absolute build-host module identifiers.
- Named dynamic entries: `src/shared/markdown/blocks/vendor-shiki.ts` and `src/shared/markdown/blocks/vendor-mermaid.ts`.
- Emitted lazy entry sizes: `vendor-shiki` 134,024 bytes and `vendor-mermaid` 38,047 bytes before gzip. Their engine-owned grammars/diagrams remain downstream dynamic imports.
- `openspec validate markdown-lazy-blocks --strict`: pass.
- `git diff --check`: pass.
- Final harness-native adversarial review: pass with 0 critical / 1 warning / 0 suggestion findings; anti-sycophancy score 0.0803571417927742. The warning is resolved by package-relative diagnostic identifiers and the no-absolute-path checker assertion.

## Constraint evaluation

- `c09-finalization-and-fallbacks`: satisfied. Streaming and lazy/failure behavior is covered by focused renderer tests; each finalized block owns its Suspense and React error boundary.
- `c09-engine-security`: satisfied. Mermaid strict configuration and sanitized SVG insertion are tested; Shiki token content is rendered as React text/span nodes rather than injected HTML.
- `c09-lazy-production-graph`: satisfied by the recursive manifest and emitted module-ownership proof above, with package-relative diagnostic identifiers only.
- `c09-wave-three-gates`: satisfied by the consolidated Wave 3 results above.

## Accessibility and visual inspection

Code controls are native buttons with visible three-pixel focus outlines and text labels. Mermaid output has an accessible image name and retains source in a keyboard-operable native disclosure. Loading and failures preserve readable source, and neither status relies on color alone. Full WCAG/responsive certification remains assigned to C-15.

## Known external build conditions

The build retains the pre-existing PGlite direct-`eval` diagnostics and the known initial chunk-size warning. C-13 owns bundle budgets; neither warning is introduced by the two lazy engine entries.

The final frozen install exposed a missing repository-root Vite filesystem allowance in the Vitest config. The minimal parity repair is verified by the final 45-file suite, including browser Storybook tests and root-hoisted PGlite asset consumers.
