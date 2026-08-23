## Why

All documentation content lanes are complete, but source validation alone does
not prove that Docusaurus builds, generated API references stage, routes render,
search works, brand/theme behavior survives production output, accessibility is
acceptable, or GitHub Pages serves the intended artifact. These deferred checks
must run once against the completed portal before publication.

## What Changes

- Add reusable local browser and deployed-route validators for the complete site.
- Expand the Pages deployment validation step to verify every required product
  route plus representative documentation/history and generated API routes.
- Run the frozen local install, production build, Rustdoc, TypeDoc, reference
  staging, composed publication/link/privacy controls, and strict OpenSpec gate.
- Exercise desktop/mobile and light/dark rendering, local search, Mermaid,
  keyboard focus, accessibility, console, network, and route behavior locally.
- Publish only through the sole deployment workflow, validate the live URL, and
  set the repository homepage to that observed URL.
- Record documentation-scoped evidence; make no runtime-readiness claim.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `documentation-publication-contract`: Require complete local artifact and live
  route validation before the public URL is represented as working.
- `dev-portal-2026`: Require rendered responsive/theme/search/Mermaid/keyboard/
  accessibility evidence, staged API references, one Pages deployment, and a
  repository homepage pointing to the observed site.

## Impact

- Documentation-only validation scripts, package commands, deployment-route
  validation in `.github/workflows/docs.yml`, final evidence, KBD reflection,
  repository metadata, and publication state.
- No runtime, product UI, provider, model, persistence, public API, dependency,
  lockfile, vendor, or submodule change.
- GitHub Actions performs deployment assembly and deployed-artifact validation
  only; all routine and content checks remain local.
