# Specification — `repair-single-pages-portal`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: verify the one-publisher implementation and its fail-closed reference
  staging contract without claiming that an undeployed, incomplete portal works.
- Deterministic execution: required.

## Target state

- `docs.yml` is the only Pages publisher.
- Website installation and build use its frozen npm lockfile and npm commands.
- Genuine Rust and TypeScript reference entrypoints are required before staging.
- No placeholder or fail-open reference fallback exists.
- Actions contain only deployment execution and deployed-artifact validation.
- Full site build and live route evidence remain deferred to the final phase gate.

## Uncomfortable fact

The corrected workflow has not run on GitHub Pages from this source SHA. Local
structural evidence proves ownership and assembly behavior, not successful live
deployment.
