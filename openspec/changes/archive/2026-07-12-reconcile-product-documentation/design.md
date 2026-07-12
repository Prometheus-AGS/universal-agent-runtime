## Context

UAR's executable product moved to a React-first frontend, committed provider catalog, tiered certification model, and explicit platform bundles, while several prominent documents continued to describe the former HTMX/Web Component direction and blanket provider/platform maturity. The support matrix and executable gates are the evidence authority. Documentation must project that authority without erasing useful historical research.

## Goals / Non-Goals

**Goals:**

- Make the README, package metadata, architecture guide, and docs site agree with the certified support matrix.
- Explain the component/hook/store/service flow, entity graph, PGlite cache, SurrealDB authority, and SSE reconciliation.
- Separate AG-UI transport from A2UI rendering and catalog metadata from provider certification.
- Make retired claims machine-detectable and prevent their return to canonical surfaces.

**Non-Goals:**

- Delete historical research or rewrite every dated assessment.
- Expand support status beyond existing evidence.
- Change runtime behavior, dependencies, protocol profiles, or release version.

## Decisions

1. **The support matrix is the maturity authority.** Canonical prose summarizes it and links to its machine-readable JSON instead of maintaining parallel capability lists. A broad prose-only rewrite was rejected because it would drift again.
2. **Canonical surfaces are checked, historical surfaces are classified.** The automated gate scans README, root metadata, current architecture/configuration, and docs-site pages. Retired directories receive directory-level historical markers; prominent current-looking files receive individual supersession banners. Deleting or mechanically rewriting research was rejected because it would destroy provenance.
3. **Local links and metadata are build gates.** The validator compares Cargo and package version/license fields, rejects known misleading phrases, and resolves canonical relative links including Docusaurus extensionless routes. A generic external link crawler was rejected because network availability must not affect ordinary release validation.
4. **Protocol and persistence ownership are explicit.** AG-UI is described as event transport, A2UI as allowlisted declarative rendering, SurrealDB as Stable server authority, and PGlite as a reconciled client cache. This avoids presenting client storage or UI artifacts as hidden business-state authorities.

## Risks / Trade-offs

- [Historical files can still be found directly] → Directory markers, prominent-file banners, and a central inventory point readers to canonical replacements.
- [A phrase scanner can produce false positives] → Restrict scanning to a small explicit canonical set and use claim-specific patterns.
- [Internal links may use docs-site routing conventions] → Resolve both literal paths and `.md` variants.
- [Provider counts change during explicit refresh] → State the current catalog size only where paired with the metadata-not-certification qualification; certification remains tier-based.

## Migration Plan

1. Replace canonical product and architecture prose.
2. Add historical inventory and supersession markers.
3. Add the documentation truth command to CI and release validation.
4. Run the truth gate, strict OpenSpec validation, and optimized Docusaurus build.

Rollback is a normal git revert. The validator and prose changes do not alter runtime data or APIs.

## Open Questions

None. Future provider snapshot refreshes must update any stated catalog count while preserving the tier qualification.
