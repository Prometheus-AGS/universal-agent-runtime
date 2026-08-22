# Design — `rewrite-readme-and-docs`

## Approach

Retain the accurate product and protocol material already present in the README
and docs site. Add only the missing customer entrypoints and correct the claims
that no longer match repository policy or the Axum router.

The Docusaurus integrity guard uses the supported Docusaurus 3 configuration:
`onBrokenLinks: 'throw'` for generated routes and
`markdown.hooks.onBrokenMarkdownLinks: 'throw'` for Markdown targets. A real
temporary page proves the production build fails when a link target is absent.
The matching `@docusaurus/theme-mermaid` 3.10.2 package and
`markdown.mermaid: true` make Mermaid fences render as diagrams in the site.

OpenAPI remains the existing hand-authored JSON document. This change does not
introduce a generator or public Rust API. It derives the version from Cargo,
retains the explicitly mounted `/api/uar/skills/reload` path, adds the separate
storage-provider `/api/uar/skills/refresh` path, and adds representative paths
for the customer route groups verified in `src/server.rs`.

Root cleanup is limited to five tracked compiler/test-output captures reviewed
by exact name and hash. No glob deletion is used.

## Boundaries

- No application UI or runtime behavior changes.
- No deployment workflow changes.
- No generated SDK API documentation is committed.
- No claim that all catalog providers have certified execution support.
- No claim that the website development dependency graph is vulnerability-free.

## Uncomfortable fact

`npm ci` for the locked website graph reports 20 high-severity development
dependency findings. The production docs build, link guard, and typecheck are
valid evidence for this change, but they are not a dependency-audit clearance.
Remediating that graph is a separate dependency change.
