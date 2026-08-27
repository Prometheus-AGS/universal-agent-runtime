# Decision Log: provider-settings-search-then-accessibility

Append-only decisions produced by KBD Analyze.

### 2026-08-26T18:40:29Z — Retain the existing responsive-layout and browser-test stack

Options: Tailwind CSS v4 native container queries; `@tailwindcss/container-queries`; `use-resize-observer`

Decision: Adopt the installed Tailwind CSS v4 container-query feature and installed Playwright Test runner. Reject the separate Tailwind plugin and ResizeObserver hook. Add no dependency.

Rationale: The official Tailwind v4 documentation directly supports container-width variants, the repository already uses `@container`, and Playwright already provides the required browser geometry and keyboard-test primitives. The remaining work is provider-specific integration and proof.

Provenance: research

Elicitation ID: N/A — stack was specified and the choice was not contested.

### 2026-08-26T18:47:30Z — Evaluate intrinsic CSS Grid before retaining container queries

Options: Tailwind container variants; intrinsic CSS Grid `auto-fit/minmax`

Decision: Retain Tailwind container variants as the adoption choice and record intrinsic grid as `cand-005` with a reference verdict.

Rationale: Intrinsic grid removes an explicit breakpoint but can produce more than two columns at large widths. The governing requirement explicitly preserves a two-column desktop composition, so the container-query state transition maps more directly to the contract.

Provenance: K3 adversarial warning plus official Tailwind arbitrary-grid documentation

Elicitation ID: N/A — this is an in-stack implementation-pattern decision, not a contested stack choice.
