## Context

`README.md` (683 lines) opens with `# Universal Agent Runtime` and a plain `##` tagline, followed immediately by prose. There are two mermaid diagrams: an Architecture Overview (`graph TB`, ~line 67) and a realtime Entity Graph data flow (`graph LR`, ~line 400). The architecture diagram's node labels use literal `\n` for line breaks, which several mermaid renderers (including some GitHub states) display verbatim instead of breaking. No status badges, hero treatment, or inline logo exist today. Logo assets already in-repo: `models.dev/logo-dark.svg`, `models.dev/logo-light.svg`, and Tauri PNG icons under `src-tauri/icons/`.

## Goals / Non-Goals

**Goals:**
- Add a hero block at the very top: title, tagline, and a badge row (license, provider count, and build/CI status).
- Convert `\n` line breaks in mermaid node labels to `<br/>` so labels render correctly.
- Keep the diagrams' depicted subsystems and relationships unchanged (readability only).
- Reference only existing assets if an inline logo is used.

**Non-Goals:**
- No prose rewrite; no changes to provider counts, model IDs, endpoints, or commands.
- No new artwork, no external image hosting beyond standard shields.io badge URLs.
- No change to the second diagram's structure beyond line-break safety if needed.

## Decisions

- **Badges via shields.io markdown image URLs.** Standard, dependency-free, and render on GitHub. Use a license badge (repo license), a providers badge (`142+ providers`, static shields), and a CI badge pointing at the existing GitHub Actions `ci.yml` workflow. These are plain markdown — no build or CI change.
- **Hero as an HTML `<div align="center">` block.** GitHub markdown renders centered HTML; this gives a clean title + tagline + badge row without a heavy template. Keep it small and text-first (anti-template: intentional, not a stock gradient hero).
- **Fix mermaid line breaks in place.** Replace `\n` with `<br/>` inside node label strings only; do not restructure `subgraph` grouping or edges. This is the smallest change that fixes rendering.
- **Inline logo optional and asset-reuse-only.** If a small logo improves the hero, reference `models.dev/logo-*.svg`; if none fits cleanly at README scale, omit the logo rather than commission art. Prefer omission over a poorly-fitting asset.

## Risks / Trade-offs

- **Risk:** a CI-status badge pointing at the wrong workflow file shows "unknown". **Mitigation:** point it at the actual `.github/workflows/ci.yml` workflow name/badge URL; verify the badge URL resolves before finishing.
- **Risk:** editing mermaid labels could accidentally alter an edge or node id. **Mitigation:** change only the text inside label quotes; leave node ids and `-->` edges untouched; eyeball a diff of the diagram blocks.
- **Trade-off:** shields.io badges are external image requests. **Accepted:** this is the universal README convention; the Tauri/no-CDN rule applies to the *app*, not to README documentation rendered on GitHub.
