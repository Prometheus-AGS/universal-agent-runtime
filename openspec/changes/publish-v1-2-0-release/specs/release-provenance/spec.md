## ADDED Requirements

### Requirement: Version aliases preserve canonical release identity
Release publication SHALL bind both version spellings to the same complete immutable OCI index and identify its attested source commit.

#### Scenario: Retrospective v1.2.0 release
- **WHEN** the existing 1.2.0 image is released as v1.2.0
- **THEN** both image tags resolve to sha256:dec5284e97b78ff65a66ef19eb43632154e23ea946f05a1d241cf979dd09566d and the source tag resolves to035bbdc702ba20daaa1add7294b35f5783ed6e0e
- **AND** release notes disclose that the build preceded the git tag and functional acceptance remains pending

#### Scenario: Subsequent main publication
- **WHEN** main publishes a development image
- **THEN** it does not move the released 1.2.0 or v1.2.0 aliases

#### Scenario: Historical source tag
- **WHEN** v1.2.0 is pushed at its attested historical source
- **THEN** deployment is paused during that push so the historical workflow cannot rebuild or overwrite the release

#### Scenario: Future semantic release
- **WHEN** a future semantic release image is published
- **THEN** prefixed and unprefixed version tags are assigned together
