## ADDED Requirements

### Requirement: Customer documentation describes the whole product accurately
The README and docs site SHALL describe runtime, frontend, SDKs, skills and
deployment with mermaid architecture/flow/scenario diagrams including fabric
context, and the published OpenAPI version SHALL match the product version.

#### Scenario: New customer orientation
- **WHEN** a customer reads the README
- **THEN** they see mermaid architecture/flow/scenario diagrams, the fabric relationship, SDK and skills sections, deployment quickstart, and a working docs-site link

#### Scenario: Docs build breaks on broken links
- **WHEN** a docs page links to a missing target
- **THEN** the site build fails rather than shipping the broken link
