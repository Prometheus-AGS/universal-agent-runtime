# frontend-design-authority Specification

## Purpose
TBD - created by archiving change amend-goal4-base-ui-divergence. Update Purpose after archive.
## Requirements
### Requirement: Frontend design authority is explicit

The project SHALL maintain a canonical, self-contained frontend design-authority document that identifies the binding design sources and defines how current phase goals, recorded operator decisions, and vendored standards are reconciled.

#### Scenario: Contributor resolves conflicting guidance

- **WHEN** a current phase goal or recorded operator decision conflicts with a vendored design standard
- **THEN** the authority document identifies which project decision governs active implementation, links internally to its rationale, and records its control-plane provenance without requiring that working-state path to resolve

#### Scenario: Operator records a new public design divergence

- **WHEN** an approved operator decision creates or changes a public frontend design divergence
- **THEN** the authority document reproduces the decision's scope and rationale before that divergence governs distributable frontend documentation

### Requirement: Base UI divergence is accurately scoped

The frontend design authority and phase Goal 4 SHALL identify Base UI-backed local wrappers as UAR's owner for general controls, navigation, overlays, and sidebars, and SHALL classify that choice as operator decision D1's override of the KnowMe UI/UX standard §6.1 and the corresponding §6.3 ownership row rather than as compliance with those provisions.

#### Scenario: Primitive ownership is reviewed

- **WHEN** a downstream frontend change selects or reviews a general control, navigation, overlay, or sidebar primitive
- **THEN** it uses the Base UI-backed local wrapper contract and records no claim that this satisfies the standard's shadcn requirement

### Requirement: Unaffected design requirements remain binding

The frontend design authority SHALL preserve all KnowMe UI/UX standard requirements not explicitly overridden by a recorded project decision, including the §6.3 ownership of Assistant UI for thread, composer, and streaming behavior; Prometheus Entity Management for durable server entities; Zustand for transient UI state; and PGlite for client-owned conversation persistence.

#### Scenario: Contributor applies the divergence

- **WHEN** a contributor applies the D1 Base UI override
- **THEN** only the named primitive-ownership provisions change and all other design, architecture, accessibility, and acceptance requirements continue to apply

### Requirement: Historical references do not silently supersede current authority

The project SHALL retain historical design artifacts as context while making clear that current phase goals and recorded decisions govern active implementation when an older document names a superseded primitive owner.

#### Scenario: Older artifact names shadcn

- **WHEN** an older plan or ADR names shadcn for a surface covered by D1
- **THEN** the contributor follows the current Base UI authority without rewriting the historical artifact as part of this reconciliation change

