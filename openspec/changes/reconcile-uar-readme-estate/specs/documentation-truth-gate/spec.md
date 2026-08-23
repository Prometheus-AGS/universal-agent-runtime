## MODIFIED Requirements

### Requirement: Canonical documentation matches certified product behavior
Present-tense public documentation SHALL identify React as the primary UI,
derive support statements from current source and the support matrix, and route
broader guidance from every UAR-owned README to a declared current authority.
Retained historical documents SHALL preserve their original text under a dated
supersession banner. Public content MUST NOT expose machine-local paths,
credential material, private keys, raw event/session payloads, or raw private
history.

#### Scenario: Stale HTMX claim
- **WHEN** an unmarked current README or canonical document claims the primary UI avoids React, presents HTMX/Web Components as the product UI, or recommends a retired package-manager path
- **THEN** documentation validation MUST fail and name the source

#### Scenario: Historical design
- **WHEN** superseded design material remains in the repository
- **THEN** it MUST contain a dated historical banner and link to current authority without rewriting the original decision as if it were current

#### Scenario: Current README lacks authority
- **WHEN** a UAR-owned README describes product-wide behavior without a current portal authority
- **THEN** documentation validation MUST fail

#### Scenario: Public source contains private operational material
- **WHEN** a public or public-normalized README/document contains a machine-local path, credential-shaped assignment, private key, raw event/session payload, or raw private-history excerpt
- **THEN** publication validation MUST fail before site assembly
