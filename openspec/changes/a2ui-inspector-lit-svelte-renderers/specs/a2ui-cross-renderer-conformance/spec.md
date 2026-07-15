## ADDED Requirements

### Requirement: Renderers share semantic output
React, Lit, and Svelte renderers SHALL produce equivalent accessibility-relevant semantics for the same certified A2UI fixture, including roles, accessible names, states, and visible text.

#### Scenario: Shared fixture renders in every framework
- **WHEN** the conformance suite renders the same baseline surface in React, Lit, and Svelte
- **THEN** normalized semantic snapshots SHALL match even when framework wrapper elements and CSS classes differ

### Requirement: Conformance is a CI gate
The frontend validation workflow SHALL execute cross-renderer conformance tests when any renderer or shared fixture changes.

#### Scenario: Renderer semantics regress
- **WHEN** one renderer changes an accessibility role, name, state, or visible text relative to the shared fixture
- **THEN** the conformance job SHALL fail with the differing framework and semantic path
