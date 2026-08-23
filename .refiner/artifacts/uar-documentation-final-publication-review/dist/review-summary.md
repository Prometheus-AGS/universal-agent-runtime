# Final publication constraint review

| Constraint | Observed evidence | Result | Limit |
|---|---|---|---|
| Complete artifact | Docusaurus production build completed; UAR library Rustdoc generated `target/doc/universal_agent_runtime/index.html` with 27 warnings; TypeDoc generated `sdks/typescript/docs/api`; staging produced non-empty Rust and TypeScript roots. | Satisfied | Internal workspace utility binaries are not represented as public UAR API. |
| Privacy and truth | Composed publication validation passed across 3,295 classified source and built paths. Focused controls rejected unclassified, ambiguous, private, credential-shaped, raw-event, machine-local, and synthetic-inference cases. | Satisfied | Generated Rust source browsing is omitted because build-output paths exposed a local home path. |
| Rendered accessibility | Playwright/Chromium plus axe-core passed desktop/mobile × light/dark with zero overflow, visible first-tab focus, and no WCAG A/AA violations. Four screenshots were visually inspected. | Satisfied | Chrome DevTools MCP was unavailable; checked-in Playwright was the documented fallback. Vale was unavailable, so prose lint is unverified. |
| Route and interaction | The local deployed-route validator returned 200 for 28 routes; a missing route returned 404 and exit 1. Browser checks observed navigation, local search for RustCrypto, Mermaid SVG, console, and network behavior. | Satisfied | Live GitHub Pages validation remains pending until the branch is pushed and deployment completes. |
| Deployment-only Actions | The Actions policy gate passed with `docs.yml` as the sole Pages publisher. The workflow contains build, deploy, and data-driven deployed-route validation only. | Satisfied | Workflow execution remains pending until push. |
| Claim boundary | Proposal, specs, task contract, and this review restrict results to documentation publication. | Satisfied | No runtime, inference, release, security, or cross-profile readiness claim is made. |

## Diff audit

The final change touches the Pages workflow, documentation-only validators and
controls, documentation-site CSS, package commands, OpenSpec evidence, and
refiner/KBD history. `Cargo.toml`, `Cargo.lock`, runtime `src/`, product
`frontend/`, dependencies, lockfiles, vendor trees, and submodule pointers are
unchanged.

## Observed defects and disposition

The first full-workspace Rustdoc attempt failed because the internal
`mcp-server-fetch` binary imports removed `rmcp::model::Content`. The publication
contract was corrected to document the public UAR library only; the product-code
defect remains explicit and untouched. The first staged Rustdoc leaked a local
build-cache path; staging now removes Rustdoc source pages and fails if any home
path survives. Browser validation then exposed three genuine contrast defects:
in-text links, Mermaid edge labels, and light-theme/footer tokens. All were fixed
in the owned portal CSS and the complete artifact was rebuilt.
