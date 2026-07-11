## ADDED Requirements

### Requirement: Feature flags control capability cost
Disabling an optional capability SHALL remove its implementation dependency and public runtime surface from the corresponding build.

#### Scenario: Minimal build
- **WHEN** UAR is built with the supported minimal bundle
- **THEN** local model, document intelligence, desktop and sandbox dependencies are absent and core run/tool protocol tests pass

### Requirement: Maintainer tooling is not a release feature
Model/catalog generation SHALL run through explicit maintainer commands and SHALL NOT participate in `--all-features` product validation.

#### Scenario: Product release build
- **WHEN** a release bundle is built
- **THEN** it performs no model regeneration or download
