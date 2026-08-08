## 1. Workflow and dependency setup

- [x] 1.1 Validate the proposal, design, and `frontend-content-rendering` delta against the C-08 plan row
- [x] 1.2 Add `remark-math`, `remark-breaks`, `rehype-raw`, `rehype-sanitize`, `rehype-katex`, `katex`, and `dompurify` in one dependency update

## 2. Shared markdown security pipeline

- [x] 2.1 Implement the shared remark plugin chain for GFM, hard breaks, and math
- [x] 2.2 Implement the restrictive sanitizer schema with approved limited HTML/SVG, safe protocols, and KaTeX input marker classes
- [x] 2.3 Implement the ordered rehype chain with raw parsing immediately followed by sanitization and then KaTeX

## 3. Single renderer and consumer migration

- [x] 3.1 Move the shared typography/code/table presentation map into `shared/markdown/markdown-components.tsx` with Flat 2.0 surface separation
- [x] 3.2 Implement `shared/markdown/markdown-bubble.tsx` for assistant-ui context and explicit `source` modes using the same pipeline
- [x] 3.3 Update chat and Skills preview consumers to use `MarkdownBubble`, then remove the legacy enhanced renderer and direct Skills `ReactMarkdown` chain

## 4. Focused evidence and acceptance

- [x] 4.1 Add focused tests for GFM, hard breaks, KaTeX, safe limited HTML, and XSS fixtures covering executable elements, handlers, styles, arbitrary classes, and unsafe protocols
- [x] 4.2 Run focused renderer tests plus frontend typecheck, lint, boundary, Flat 2.0, and duplicate-renderer checks
- [x] 4.3 Run strict OpenSpec verification, artifact-refiner validation, accessibility review, and isolated adversarial review; resolve actionable findings
- [x] 4.4 Prepare the canonical C-08 completion transition and verified OpenSpec archive handoff
