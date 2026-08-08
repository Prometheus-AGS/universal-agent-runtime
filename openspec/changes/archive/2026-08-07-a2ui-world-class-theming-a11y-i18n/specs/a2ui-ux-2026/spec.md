## ADDED Requirements

### Requirement: Certified surface themes
The renderer SHALL provide scoped light, dark, and high-contrast semantic CSS-variable themes without mutating the host document theme.

#### Scenario: Explicit theme selection
- **WHEN** a host renders an A2UI surface with a certified theme
- **THEN** the surface root exposes that theme and all renderer primitives consume its scoped semantic variables

#### Scenario: Forced colors
- **WHEN** the operating system enables forced-colors mode
- **THEN** controls, focus indicators, text, borders, and error states remain perceivable using system colors

### Requirement: WCAG 2.2 AA interaction semantics
Every certified A2UI surface SHALL preserve semantic roles, keyboard navigation, visible focus, associated labels and validation feedback, responsive reflow, and non-color state cues required for WCAG 2.2 AA.

#### Scenario: Invalid control
- **WHEN** a rendered control is invalid
- **THEN** assistive technology can discover both its invalid state and its explanatory validation message

#### Scenario: Narrow or zoomed layout
- **WHEN** a surface is rendered at a narrow viewport or high zoom
- **THEN** structural rows wrap without horizontal loss and interactive targets remain operable

### Requirement: Localized renderer-owned copy
The renderer SHALL provide English, Spanish, Japanese, and Simplified Chinese resources for every renderer-owned user-facing string and SHALL support explicit LTR, RTL, and automatic direction.

#### Scenario: Supported locale
- **WHEN** a host selects `es`, `ja`, or `zh`
- **THEN** renderer-owned empty, error, retry, status, choice, and validation copy uses that locale while agent-authored content remains unchanged

#### Scenario: RTL framework
- **WHEN** a host selects RTL direction
- **THEN** the surface root and logical layout primitives expose RTL direction without requiring agent-authored physical positioning

### Requirement: Purposeful reduced-motion-aware transitions
The renderer SHALL use Motion for surface entrance, exit, update, and streaming transitions and SHALL honor the user's reduced-motion preference.

#### Scenario: Surface update
- **WHEN** a live surface changes state
- **THEN** the update uses a bounded product transition that does not change semantic ordering or block interaction

#### Scenario: Reduced motion
- **WHEN** the user requests reduced motion
- **THEN** the surface remains fully usable with transitions reduced or disabled

### Requirement: Recoverable surface boundary
Every `UarSurface` SHALL contain render failures, distinguish empty and failed states, expose localized diagnostics, and offer retry when the host supplies a retry action.

#### Scenario: Unsupported component
- **WHEN** a surface references a component outside the approved catalog
- **THEN** the renderer fails closed inside the surface boundary and presents a visible safe error without executing agent code

#### Scenario: Retry recovery
- **WHEN** the user activates retry after a surface failure
- **THEN** the boundary resets, invokes the host retry action, and can render the next valid surface update

### Requirement: Automated accessibility evidence
The A2UI renderer SHALL run axe-core against representative themes, locales, controls, entity surfaces, and recovery states in package tests and path-filtered CI.

#### Scenario: Accessibility regression
- **WHEN** axe-core reports a serious or critical violation in a certified fixture
- **THEN** the Change 21 validation job fails
