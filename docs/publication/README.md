# Documentation publication contract

This directory defines what UAR may publish and how the public portal proves that its claims are complete, current, and safe. It does not replace implementation, committed dependency manifests such as [`Cargo.toml`](../../Cargo.toml), canonical [OpenSpec capabilities](../../openspec/specs/), or the canonical [KBD waypoint](../../.kbd-orchestrator/current-waypoint.json). The project rules also name an operator-owned `versions.toml`; it is not present in this checkout, so no claim in this phase treats it as inspected evidence.

## Manifests

[`sources.json`](sources.json) classifies every tracked README and every tracked path under `docs/`, `website/`, `.prometheus/`, `.kbd-orchestrator/`, and `openspec/`. Each path must resolve to exactly one rule:

- `public` — authored directly for public delivery;
- `public-normalize` — public source that must be reconciled or synthesized before portal publication;
- `private-synthesis-only` — evidence that may inform reviewed prose but must never be copied directly;
- `excluded` — third-party or otherwise non-publishable material.

Rules record ownership, current or historical status, canonical authority, and either a public destination or a reason the source is not directly published. Generated mirrors also name `generatedFrom`; change that source and regenerate the mirrors instead of editing each copy.

[`routes.json`](routes.json) maps every row in the [product surface inventory](../product-surface-inventory.md) to one Docusaurus document ID and public route. Profile lists are limits: a `server-full` statement transfers to no other profile unless the route entry and its governing evidence say so.

## Current and historical material

Current guidance states what the checked-in implementation and canonical specs support now. Retained historical material keeps its original claims but adds a dated supersession banner and a link to current authority. Never rewrite an old decision to make it appear that the project always held today's position.

The earlier minimum portal change is retained as history and explicitly [superseded](../../openspec/changes/docs-hosted-rustdoc-typedoc-docusaurus-ia/superseded.md). Its placeholder content and GitHub Actions routine-testing requirements are not current authority.

## Public history provenance

Architecture and testing history pages derived from internal records use front matter like this:

```yaml
source_records:
  - docs/adr/0017-relicense-runtime-to-mit.md
  - openspec/changes/docs-hosted-rustdoc-typedoc-docusaurus-ia/proposal.md
  - .prometheus/decisions.md
current_authority: /docs/architecture
```

The page body is reviewed synthesis. Do not paste `.prometheus` records, KBD events, session or conversation logs, wiki entries, machine-local paths, credentials, or private-key material into public Markdown or assets.

## README ownership

UAR-owned READMEs describe their local package or directory and link to the portal for broader guidance. The iterative-evolver READMEs are generated mirrors; update their declared source and regenerate. Files beneath `vendor/` are third-party content and remain semantically unchanged.

## Local verification

After documentation implementation is complete, run:

```bash
npm run docs:publication:validate
```

The command validates source classification, route coverage, provenance, historical banners, privacy sanitization, documentation truth, and the one-Pages-publisher policy. GitHub Actions is deployment-only; prose, link, truth, accessibility, and other routine development checks run locally.

The final branded-site change extends the local gate with the frozen production build, representative routes, responsive light/dark screenshots, keyboard behavior, and accessibility. A green publication-contract result is a documentation result only, not a runtime-readiness verdict.
