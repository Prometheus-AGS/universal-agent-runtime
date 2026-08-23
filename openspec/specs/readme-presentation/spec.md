# readme-presentation Specification

## Purpose
TBD - created by archiving change refresh-readme-diagrams-and-branding. Update Purpose after archive.

## Requirements

### Requirement: README presents a branded hero with status badges
The `README.md` SHALL open with the project title, the UAR tagline used by the
public portal, an existing repository logo, and a row of status badges for
license, version, and documentation. Badges and links MUST resolve to real
repository or public destinations and MUST NOT imply unobserved build,
publication, or runtime status.

#### Scenario: Hero and badges are present at the top of the README
- **WHEN** the README is opened
- **THEN** the first screenful MUST contain the project title, tagline, existing logo, and at least three status badges
- **AND** the documentation badge or adjacent primary link MUST target the canonical Pages portal

#### Scenario: Badge would overstate status
- **WHEN** a badge has no current, independently observable source for its claim
- **THEN** README validation MUST reject the badge or require a non-status alternative

### Requirement: Architecture and data-flow diagrams render correctly
The README's Mermaid diagrams SHALL use Mermaid-safe labels and SHALL describe
the current trust, execution, protocol, and state-owner boundaries without
transferring a `server-full` claim to `minimal` or `embedded-mobile`.

#### Scenario: Diagram labels use mermaid-safe line breaks
- **WHEN** a Mermaid node label spans multiple lines
- **THEN** the label MUST use `<br/>` or another Mermaid-safe break rather than a literal `\n` sequence

#### Scenario: Diagram content is preserved
- **WHEN** the diagrams are reconciled
- **THEN** their subsystems and relationships MUST match current source and canonical portal architecture
- **AND** deprecated transport or UI descriptions MUST NOT appear as current behavior

### Requirement: Branding is internally consistent
The README's title, tagline, logo, badge row, and portal link SHALL use the
current UAR identity and reference only existing repository assets and observed
public destinations.

#### Scenario: No new art is introduced
- **WHEN** an inline logo is used in the hero
- **THEN** it MUST reference an asset already present in the repository

#### Scenario: Title and tagline agree across the hero
- **WHEN** the README and portal home are compared
- **THEN** the project name and tagline MUST agree

### Requirement: README estate follows documented ownership and navigation
Every tracked README SHALL have an authoritative source-manifest disposition. UAR-owned current README files SHALL identify their scope, link to the canonical portal or parent navigation, and avoid duplicating broad product claims that belong in the portal. Generated mirrors SHALL be updated from their source, historical README files SHALL carry dated supersession context, and vendored README files SHALL remain semantically unchanged.

#### Scenario: UAR-owned README is reconciled
- **WHEN** a reader opens a current UAR-owned README
- **THEN** it describes that directory or package accurately and links to the canonical public documentation for broader guidance

#### Scenario: Generated README changes
- **WHEN** a generated README requires a semantic correction
- **THEN** the correction is made in its declared source and the mirror is regenerated

#### Scenario: Vendored README is audited
- **WHEN** the documentation estate is reconciled
- **THEN** the vendored README remains unchanged and its third-party ownership and exclusion are recorded in the source manifest

### Requirement: Repository presentation exposes the working public portal
The root README SHALL present the UAR identity, purpose, support boundaries, and a public documentation link that resolves to the branded portal. The repository homepage field SHALL use the same URL after the deployed site has been observed working.

#### Scenario: Reader follows the documentation link
- **WHEN** a reader follows the root README documentation link
- **THEN** the branded Docusaurus portal root loads and its canonical deep routes are reachable

#### Scenario: Repository homepage is inspected
- **WHEN** a visitor opens the GitHub repository page after documentation deployment
- **THEN** the homepage field links to the same validated public portal URL

### Requirement: Every tracked README has one declared disposition
The documentation contract SHALL enumerate every tracked `README.md` exactly
once as root, UAR-owned current, generated mirror, or vendored exclusion. The
estate denominator SHALL be derived from the checkout rather than copied from a
prior assessment.

#### Scenario: README is added without ownership
- **WHEN** a tracked README has no manifest entry or matches more than one entry
- **THEN** local README validation MUST fail and name the path

#### Scenario: UAR-owned README is current
- **WHEN** a reader opens a subordinate UAR-owned README
- **THEN** it MUST describe its local directory or package and link broader guidance to a current portal authority

### Requirement: Generated and vendored READMEs preserve ownership
The five iterative-evolver mirrors SHALL match one declared canonical source
byte-for-byte after regeneration, and the two vendored READMEs SHALL remain
semantically unchanged.

#### Scenario: Generated mirror drifts
- **WHEN** any iterative-evolver mirror differs from its declared source
- **THEN** local README validation MUST fail and identify the mirror

#### Scenario: Vendored README is edited
- **WHEN** a change alters either vendored README rather than its upstream source
- **THEN** the permitted-surface gate MUST fail
