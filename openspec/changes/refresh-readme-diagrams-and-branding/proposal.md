## Why

The README is the repo's public face and doubles as a living template, but its top-of-file presentation is plain text with no visual identity: no hero/title treatment, no status badges (build, license, provider count), and no at-a-glance signal of project health. Its two mermaid diagrams (Architecture Overview, and the realtime Entity Graph data flow) have drifted slightly in readability and use `\n` literals for line breaks that some mermaid renderers show verbatim. A focused visual polish — diagrams + branding — raises the credibility of the template without a deep content rewrite.

## What Changes

- Add a README hero/title treatment: a centered title block with the tagline and a row of status badges (build status, license, Rust edition, provider count) using shields.io-style badges at the top of `README.md`.
- Refresh the two existing mermaid diagrams:
  - Normalize node labels to use `<br/>` (or mermaid-safe line breaks) instead of `\n` literals so labels render correctly across GitHub and other mermaid renderers.
  - Tidy grouping/labels for readability without changing the architecture they depict.
- Ensure branding consistency: confirm the project name/tagline usage is consistent between the header and the badge row, and reference an existing logo asset (e.g. `models.dev/logo-*.svg` or a Tauri icon) if a small inline logo improves the hero — only if an existing asset fits; do not commission new art.
- **Non-content:** no rewrite of prose sections, no changes to provider counts, model identifiers, endpoints, or commands (a full accuracy pass is explicitly out of scope for this change).

## Capabilities

### New Capabilities
- `readme-presentation`: The README SHALL present a branded hero (title, tagline, status badges) and correctly-rendering architecture/data-flow diagrams, so the repository's public face reads as an intentional, healthy template at a glance.

### Modified Capabilities
<!-- None. No existing capability's requirements change; this introduces README presentation requirements only. -->

## Impact

- **Affected code:**
  - Modified: `README.md` (hero/badges block added at top; two mermaid diagrams normalized).
  - Possibly referenced (not modified): an existing logo asset under `models.dev/` or `src-tauri/icons/` if an inline logo is used.
- **Runtime UX:** None. Documentation-only; no runtime, API, or UI code changes.
- **Provider compatibility:** None.
- **Realtime state:** None.
- **Dependencies:** None. Badges are external shields.io image URLs referenced in markdown; no package changes.
- **KBD workflow state:** Tracked as change 8/9 (Round 4) of phase `uar-production-ready-uiux-2026-07`; `progress.json` and the waypoint advance via `/kbd-apply` per task. The sibling Round-4 change `bootstrap-docusaurus-site` remains blocked pending a hosting/deployment target and is not part of this change.
