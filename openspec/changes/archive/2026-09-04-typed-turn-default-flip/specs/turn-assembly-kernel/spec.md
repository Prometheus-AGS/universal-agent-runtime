## ADDED Requirements

### Requirement: Typed assembly becomes the default only on recorded evidence
The runtime's default harness mode SHALL change from `legacy` to `typed` only after a checked-in parity report shows zero unexpected differences over the parity corpus and a live smoke run in `shadow` mode shows zero unexpected differences, both recorded in the project decision log with the corpus size and smoke set contents, and `legacy` SHALL remain selectable for one minor release after the change.

#### Scenario: Evidence present
- **WHEN** the parity report and the live smoke record both show zero unexpected differences
- **THEN** a fresh installation uses `typed` by default and `mode: legacy` still selects the legacy path

#### Scenario: Evidence absent
- **WHEN** either record is missing or shows an unexpected difference
- **THEN** the default remains `legacy` and the change is not merged
