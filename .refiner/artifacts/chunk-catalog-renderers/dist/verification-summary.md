# C-12 Artifact Refinement Verification

Change: `chunk-catalog-renderers`

## Passing evidence

- `pnpm -C frontend typecheck`
- `pnpm -C frontend lint`
- `node scripts/check-frontend-boundaries.mjs` — zero production violations
- `pnpm frontend:style-gate` — 385 tracked legacy findings, zero new findings; negative fixtures pass
- Six focused C-12 files — 42 tests passed
- Full Wave 4 frontend suite — 59 files and 300 tests passed, including the chunk catalog Storybook axe gate
- Root and frontend frozen installs
- Normal production build plus `vite build --manifest`
- Lazy Mermaid/Shiki graph — zero forbidden static modules, missing dynamic entries, invalid names, or absolute module ids
- Static bundle validation — seven referenced assets present

## Constraint evaluation

- `c12-exhaustive-protocol`: satisfied by exact typed unions, `assertNever`, typed disposition maps, deterministic projections, and compile-time fixtures for every block and chunk kind.
- `c12-complete-accessible-catalog`: satisfied by all visible Assistant UI data registrations, intentional trace-only omissions, semantic divider/disclosures/status/media handling, passing axe, and responsive screenshots.
- `c12-artifact-trust-boundaries`: satisfied by the existing Markdown/Mermaid and SVG sanitizers, empty-sandbox HTML iframe, escaped JSON, closed finite chart parser, A2UI policy component, scheme/data-MIME URL allowlist, safe media fallback, and hidden raw payloads.
- `c12-wave4-gates`: satisfied by the recorded focused and aggregate command receipts. The build retains the already-known PGlite direct-eval and initial-chunk warnings owned by later C-13 budget work.

## Manual audit, critique, and polish

- The initial aggregate story exposed duplicate named region landmarks for repeated reasoning kinds. Generic chunk surfaces now use neutral labeled/live containers, avoiding landmark noise while keeping accessible descriptions and announcements.
- Screenshots at 320, 768, 1024, and 1440 pixels show one readable column, wrapping metadata, no horizontal clipping, and stable disclosure/state hierarchy.
- All new interactive disclosure and approval/source/download controls preserve 44px minimum targets where rendered. The surfaces add no decorative borders, shadows, gradients, blur, or parallel theme-specific colors.
- New chunk surfaces are static except Recharts `isAnimationActive="auto"`, which follows reduced-motion preferences. The reasoning summary explicitly includes `motion-reduce:transition-none`.

## Trust boundaries

- Runtime data never enters chart CSS/markup configuration; charts accept only `bar`/`line`, bounded labels, bounded series, and finite numeric values, then use application-owned color tokens.
- HTML remains in an empty-sandbox iframe with source disclosure; JSON and raw values render as React text; SVG and Mermaid retain sanitizer boundaries; A2UI remains policy-gated.
- Unknown CUSTOM and RAW events are preserved with durable identity for trace inspection and never become chat prose.

## Review remediation

- Round one found missing late A2UI lifecycle propagation and legacy terminal run finalization; both paths now update canonical state and have focused coverage.
- Round two found incomplete durable A2UI envelopes, thinking/reasoning collapse, persistence errors entering transport retry, and terminal tools without output. The store now persists fixed-MIME envelopes and explicit terminal results; stream reduction preserves both chunk kinds; persistence errors remain local and logged.
- The renderer boundary now rejects executable provider-authored citation, artifact, file, image, video, and poster URLs, including unsafe `data:` payloads.
- The post-remediation aggregate gate is green. The second receipt remains `BLOCK` because it describes the pre-remediation packet, and it records `harness-native` / `same-model-collision`; the two-round cap prevents a third judge pass.

## Review status

Deterministic refinement and post-review verification pass. Two isolated review rounds are recorded; every reported defect has an evidence-backed remediation, with the same-family fallback limitation retained for downstream visibility.
