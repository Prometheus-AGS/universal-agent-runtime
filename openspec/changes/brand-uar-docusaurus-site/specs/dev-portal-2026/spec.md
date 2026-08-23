## ADDED Requirements

### Requirement: Portal presents the shipped UAR identity
The documentation portal SHALL use the same UAR mark, wordmark, ember/cyan palette, surface hierarchy, typography roles, and Flat 2.0 interaction language as the shipped React application. It SHALL contain no stock Docusaurus identity, tutorial copy, sample illustration, or unrelated social-card asset.

#### Scenario: Reader opens the portal
- **WHEN** a reader opens the homepage or a documentation route
- **THEN** the UAR identity is visible in navigation and page presentation
- **AND** the page contains no stock Docusaurus product identity

#### Scenario: Flat 2.0 regions render
- **WHEN** navigation, hero, cards, sidebars, code blocks, or callouts distinguish adjacent regions
- **THEN** they use filled surface steps and spacing rather than decorative borders, separator lines, gradients, or shadows

### Requirement: Homepage orients readers to the product
The portal homepage SHALL explain the runtime's purpose, agent/host trust boundary, supported protocol and product surfaces, profile limits, and direct next steps into concepts, guides, reference, and operations.

#### Scenario: New reader chooses a path
- **WHEN** a reader reaches the homepage without prior UAR knowledge
- **THEN** they can identify what UAR does, what it does not claim across profiles, and which primary documentation path matches their goal

### Requirement: Portal interaction remains accessible across presentation modes
The portal SHALL preserve semantic navigation, visible keyboard focus, zoom and reflow, touch targets, heading hierarchy, light/dark/system themes, readable code and Mermaid output, and reduced-motion behavior at the responsive sizes certified by the final local gate.

#### Scenario: Keyboard navigation
- **WHEN** a reader navigates interactive portal controls without a pointer
- **THEN** focus order follows document order and every focused control has a visible UAR-token focus indicator

#### Scenario: Reduced motion preference
- **WHEN** the reader enables reduced motion
- **THEN** nonessential animation is removed or reduced without hiding content or state

#### Scenario: Narrow viewport
- **WHEN** the portal is viewed at a supported mobile width or browser zoom level
- **THEN** navigation, copy, code, tables, and calls to action remain reachable without page-level horizontal scrolling

### Requirement: Portal search is local and deterministic
The portal SHALL build a local search index from accepted public documentation and SHALL NOT require a hosted search or analytics service to discover content.

#### Scenario: Reader searches documentation
- **WHEN** a reader submits a query for indexed public content
- **THEN** matching documentation routes are returned from the locally built index
- **AND** private-synthesis-only or excluded sources are absent from results

#### Scenario: Search index cannot be produced
- **WHEN** the production documentation build cannot generate the configured local index
- **THEN** the build fails instead of publishing a portal with a falsely advertised search control
