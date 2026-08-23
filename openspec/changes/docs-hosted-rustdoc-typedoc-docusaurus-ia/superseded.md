# Superseded portal change

**Superseded on:** 2026-08-23  
**Successor phase:** `uar-branded-documentation-site`  
**Successor contract:** `establish-documentation-publication-contract`

This change is retained as the historical minimum-portal implementation. It is not current publication authority and must not be archived again into canonical specs.

The successor replaces these assumptions:

- placeholder section content is not complete product documentation;
- a prose validator that is absent cannot be treated as passing;
- routine prose, link, accessibility, conformance, and product checks run locally, not in GitHub Actions;
- TypeScript API documentation cannot publish independently to the same Pages environment as the portal;
- generated Rust, TypeScript, or Python reference links are advertised only when the corresponding artifact is actually staged.

## Open task disposition

- **8.1 custom domain:** superseded. The current phase publishes to the existing GitHub Pages URL; a custom domain remains outside scope unless separately requested.
- **8.2 enable Pages:** satisfied historically. Pages is enabled with HTTPS, but the observed live root serves the competing TypeScript artifact; the successor owns repair and validation.
- **8.3 full content and generated references:** transferred to the registered documentation-site changes `document-uar-theory-and-architecture`, `document-inference-skills-knowledge-and-agents`, `document-security-tenancy-governance-and-operations`, `document-apis-sdks-tools-and-deployment`, and `repair-single-pages-portal`.

The original `proposal.md`, `design.md`, and `tasks.md` remain unchanged as evidence of what that change actually delivered and deferred.
