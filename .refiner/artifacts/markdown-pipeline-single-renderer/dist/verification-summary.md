# C-08 Verification Summary

Change: `markdown-pipeline-single-renderer`

## Passing evidence

- `pnpm -C frontend typecheck`
- `pnpm -C frontend lint`
- `node scripts/check-frontend-boundaries.mjs` — zero production violations
- `node scripts/check-flat2-style.mjs` — 391 tracked legacy violations, zero new
- `pnpm -C frontend exec vitest run src/shared/markdown/markdown-bubble.test.tsx` — 1 file, 14 tests
- Renderer census — `ReactMarkdown`, `MarkdownTextPrimitive`, and both plugin props are owned only by `frontend/src/shared/markdown/`
- `openspec validate markdown-pipeline-single-renderer --strict`
- `git diff --check`

## Requirement mapping

- Single shared renderer: `frontend/src/shared/markdown/markdown-bubble.tsx`, consumed by chat and the Skills editor preview.
- Complete plugin chain: `plugins/remark-chain.ts` and `plugins/rehype-chain.ts`.
- Untrusted HTML boundary: `plugins/sanitize-schema.ts`, with raw parsing immediately followed by sanitization.
- Math boundary: math marker classes pass through sanitization and the trusted KaTeX transform runs afterward with package-owned CSS.
- Provider/realtime preservation: the renderer consumes existing assistant-ui context or explicit strings and imports no store, service, persistence, or provider module.
- Focused evidence: `markdown-bubble.test.tsx` covers both renderer modes, GFM, hard breaks, valid/malformed KaTeX, block/inline code, AST metadata stripping, executable tags, handlers, styles, arbitrary classes, unsafe protocols, approved limited HTML/SVG, safe external links, and standalone DOMPurify SVG sanitization.

## Accessibility review

- Semantic markdown elements remain semantic; the visual spacer uses `role="separator"`.
- Links have visible text plus forced `_blank` isolation; no new icon-only controls, focus traps, animations, live regions, or media controls were introduced.
- Approved inline SVG requires an accessible role/label to be discoverable in tests; raw artifact-specific alt enforcement remains owned by later content-block work.
- Existing tokenized text/surface colors are reused; no new colors or contrast pairs were introduced.

## Isolated review

- Final round: PASS, 0 critical / 2 warnings / 0 suggestions.
- Judge: `k3`; producer: `openai/gpt-5`; cross-model check: verified-distinct; REST-gateway isolation.
- Anti-sycophancy score: 0.0.
- The raw-SVG no-DOM warning was resolved with a fail-closed guard and all deterministic gates were rerun. The retained token warning belongs to an earlier completed Tailwind/token change visible in the overlapping consumer diff.
