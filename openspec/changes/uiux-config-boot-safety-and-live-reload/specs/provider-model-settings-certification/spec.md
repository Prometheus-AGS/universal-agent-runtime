## MODIFIED Requirements

### Requirement: Provider bootstrap preserves persistent settings authority
Native bootstrap SHALL merge missing YAML entries and SHALL NOT replace API/UI-created provider settings or the selected default model held in persistent storage. A provider row seeded from YAML SHALL be stored as configuration-owned. For an operator-edited row, bootstrap and reload SHALL append YAML models never offered to it, SHALL NOT re-append a model an operator removed, and SHALL report other differing fields as drift by name.

#### Scenario: Persisted provider differs from catalog default
- **WHEN** the operator has edited a provider through the API or UI after initial seed
- **THEN** a service restart retains the persisted setting rather than restoring the YAML/catalog default

#### Scenario: New YAML model reaches an operator-edited provider
- **WHEN** the YAML adds a model to a provider whose stored row an operator has edited, and the service restarts or reloads
- **THEN** the model is present in the stored provider, the operator's other edits are unchanged, and the provider is not listed as drifted for the added model

#### Scenario: Operator-removed model stays removed
- **WHEN** an operator removes a model that the YAML still lists, and the service restarts or reloads
- **THEN** the model is not added back

#### Scenario: Seeded row follows the YAML until an operator edits it
- **WHEN** a provider row was seeded from YAML and never edited, and the YAML changes that provider and the configuration is reloaded
- **THEN** the stored row reflects the YAML change
