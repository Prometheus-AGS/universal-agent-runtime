## MODIFIED Requirements

### Requirement: Customer documentation describes the whole product accurately
The README and public documentation site SHALL describe the complete supported UAR product surface, including runtime theory, frontend, profiles, provider and model routing, agents, SDKs, tools, skills, knowledge, memory, tenancy, governance, security, realtime behavior, operations, and deployment boundaries. Architecture, flow, and scenario diagrams SHALL identify the fabric context where applicable; present-tense support claims SHALL name their governing source and profile; and the published OpenAPI version SHALL match the product version.

#### Scenario: New customer orientation
- **WHEN** a customer reads the README
- **THEN** they see the project purpose, architecture and flow diagrams, fabric relationship, primary product surfaces, deployment quickstart, support boundaries, and a working documentation-site link

#### Scenario: Customer guide coverage
- **WHEN** a customer opens the documentation site
- **THEN** every surface in the authoritative product inventory has a canonical guide or an explicit documented exclusion
- **AND** the guide links to the relevant configuration, API, security, operational, and profile contracts

#### Scenario: Docs build breaks on broken links
- **WHEN** a public page links to a missing internal target
- **THEN** the local production site build fails rather than producing a publishable artifact

#### Scenario: Published API metadata matches the runtime
- **WHEN** a client reads the generated OpenAPI document
- **THEN** its version matches `CARGO_PKG_VERSION` and it documents the actual chat, runs, providers, skills, knowledge, authentication, and realtime route groups

#### Scenario: Repository root contains no captured test output
- **WHEN** a customer inspects a release source tree
- **THEN** the reviewed `TEST_EXECUTION_REPORT.md`, `output*.txt`, and `u00261` scratch artifacts are absent

#### Scenario: Profile-specific behavior is documented
- **WHEN** behavior or evidence differs between `server-full`, `minimal`, and `embedded-mobile`
- **THEN** the public guide reports those results separately and makes no aggregate or cross-profile claim

