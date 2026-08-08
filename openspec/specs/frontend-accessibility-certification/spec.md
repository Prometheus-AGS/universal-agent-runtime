# frontend-accessibility-certification Specification

## Purpose
TBD - created by archiving change a11y-and-responsive-certification. Update Purpose after archive.
## Requirements
### Requirement: Automated WCAG certification is fail closed
The React frontend SHALL run axe against representative integrated application surfaces
and Storybook components with no disabled accessibility checks and no serious or critical
violations in light and dark themes.

#### Scenario: An integrated surface is certified
- **WHEN** the accessibility certification suite loads a representative application route
- **THEN** axe reports no in-scope WCAG 2.2 AA violations and the receipt names the route, theme, and viewport

#### Scenario: A component story has an accessibility defect
- **WHEN** the fail-closed Storybook accessibility test detects a violation
- **THEN** the test fails instead of suppressing the result through story parameters

### Requirement: Keyboard operation and focus are perceivable
The React frontend SHALL support logical keyboard-only operation with no trap, SHALL return
focus when modal surfaces close, and SHALL render a 3px ember focus indicator on every
focused actionable control.

#### Scenario: A desktop operator navigates without a pointer
- **WHEN** the operator uses Tab, Shift-Tab, Enter, and Escape through the shell and command palette
- **THEN** focus follows the interaction order, the selected action runs, the palette closes, and focus returns to its trigger

#### Scenario: A compact-shell operator navigates without a pointer
- **WHEN** the operator tabs through compact navigation and opens and closes the configure dialog
- **THEN** every action is reachable, the dialog has no keyboard trap, and focus returns to the Configure trigger

#### Scenario: An actionable element receives keyboard focus
- **WHEN** focus-visible applies to a native, Base UI, shell, or feature control
- **THEN** its computed indicator is 3px and uses the resolved ember focus colour with at least 3:1 adjacent contrast

### Requirement: Semantics expose structure and changing state
The React frontend SHALL expose one main landmark, named navigation landmarks, accessible
names for controls, polite live regions for streaming and asynchronous status, and textual
or iconographic status cues in addition to colour.

#### Scenario: A screen-reader user enters the application shell
- **WHEN** the shell renders in desktop or compact mode
- **THEN** a skip link, banner, named navigation, and main landmark expose the page structure

#### Scenario: Runtime state changes
- **WHEN** streaming, loading, tool, readiness, or replay status changes
- **THEN** meaningful text is available programmatically and colour is not its only carrier

#### Scenario: Media or generated content renders
- **WHEN** an image, diagram, chart, or generated surface is displayed
- **THEN** informative content has an accessible text alternative and decorative content is hidden from assistive technology

### Requirement: Motion and target sizing meet WCAG 2.2 AA
The React frontend SHALL honor `prefers-reduced-motion: reduce` and standalone actionable
targets SHALL be at least 24 by 24 CSS pixels unless a documented WCAG exception applies.

#### Scenario: Reduced motion is requested
- **WHEN** the operating system preference requests reduced motion
- **THEN** non-essential animations and transitions complete without visible motion while interaction remains available

#### Scenario: A standalone control is rendered
- **WHEN** its bounding box is measured at a required responsive width
- **THEN** both dimensions are at least 24 CSS pixels or the evidence identifies the applicable spacing or inline exception

### Requirement: Responsive accessibility is certified across the required matrix
The React frontend SHALL remain operable at 320, 768, 1024, and 1440 CSS pixels in both
light and dark themes without horizontal page overflow, clipped primary actions, or
overlapping shell and content regions.

#### Scenario: A phone-width surface is certified
- **WHEN** a representative route renders at 320 CSS pixels
- **THEN** compact bottom navigation is available, desktop navigation is absent, core content remains reachable, and no page-level horizontal overflow occurs

#### Scenario: A tablet-width surface is certified
- **WHEN** a representative route renders at 768 CSS pixels
- **THEN** compact navigation remains operable, content reflows without overlap, and all primary actions remain reachable

#### Scenario: A desktop-width surface is certified
- **WHEN** a representative route renders at 1024 or 1440 CSS pixels
- **THEN** the navigation rail is available, compact navigation is absent, and shell chrome does not overlap the main content

### Requirement: The binding acceptance checklist has an auditable result
The phase SHALL record every applicable KnowMe §12 acceptance statement as verified,
failed, not applicable, backend-bound, or separately owned and SHALL NOT report the change
complete while an applicable C-15-owned statement is failed.

#### Scenario: Certification evidence is reviewed
- **WHEN** the final C-15 report is prepared
- **THEN** each accessibility and responsive claim links to repeatable evidence and every unverified claim is explicitly classified

