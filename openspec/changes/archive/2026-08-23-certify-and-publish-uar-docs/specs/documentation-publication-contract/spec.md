## ADDED Requirements

### Requirement: Completed artifact certification

The portal SHALL be production-built only after all content lanes are complete.
The final local artifact SHALL contain the Docusaurus portal plus non-empty real
Rust and TypeScript references and SHALL pass source classification, privacy,
truth, link, and required-route validation. The Rust reference SHALL document
the public `universal-agent-runtime` library under `server-full`, excluding
workspace-only utility binaries from the represented public API.

#### Scenario: Generated reference is absent

- **WHEN** Rustdoc or TypeDoc output is missing or empty
- **THEN** staging exits non-zero and no placeholder reference is published

### Requirement: Rendered and live route evidence

The complete portal SHALL be inspected locally for responsive light/dark
rendering, brand identity, search, Mermaid, keyboard focus, accessibility,
console/network failures, and representative navigation. After deployment,
every required product route plus root, history, and generated-reference routes
SHALL return successful responses from the canonical Pages URL.

#### Scenario: Required deployed route is missing

- **WHEN** a required route returns a non-success response after bounded retries
- **THEN** deployment validation fails and the site/homepage claim remains unverified

### Requirement: Publication result remains documentation-scoped

A passing documentation artifact or Pages deployment MUST NOT be presented as
runtime, inference, release, security, or cross-profile readiness evidence.

#### Scenario: Portal is live

- **WHEN** the complete documentation site passes local and deployment checks
- **THEN** the result establishes documentation publication only and lists all product claims it did not exercise
