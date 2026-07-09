# readme-presentation Specification

## Purpose
TBD - created by archiving change refresh-readme-diagrams-and-branding. Update Purpose after archive.
## Requirements
### Requirement: README presents a branded hero with status badges
The `README.md` SHALL open with a hero block containing the project title, its tagline, and a row of status badges (at minimum: license, and provider count or build status), so the repository's public face reads as an intentional template.

#### Scenario: Hero and badges are present at the top of the README
- **WHEN** the README is opened
- **THEN** the first screenful MUST contain the project title, the tagline, and at least two status badges
- **AND** the badges MUST use standard markdown image syntax pointing at a badge provider (e.g. shields.io)

### Requirement: Architecture and data-flow diagrams render correctly
The README's mermaid diagrams SHALL use mermaid-safe line breaks so node labels render correctly on GitHub and other mermaid renderers, without changing the architecture they depict.

#### Scenario: Diagram labels use mermaid-safe line breaks
- **WHEN** a mermaid node label spans multiple lines
- **THEN** the label MUST use `<br/>` (or another mermaid-safe break) rather than a literal `\n` sequence

#### Scenario: Diagram content is preserved
- **WHEN** the diagrams are refreshed
- **THEN** the set of subsystems and their relationships depicted MUST be preserved (readability changes only, no architectural change)

### Requirement: Branding is internally consistent
The README's title, tagline, and badge row SHALL be consistent with one another and reference only existing repository assets.

#### Scenario: No new art is introduced
- **WHEN** an inline logo is used in the hero
- **THEN** it MUST reference an asset already present in the repository (no newly commissioned art)

#### Scenario: Title and tagline agree across the hero
- **WHEN** the hero block is read
- **THEN** the project name and tagline MUST match those used elsewhere in the README header

