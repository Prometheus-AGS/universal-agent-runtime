## ADDED Requirements

### Requirement: The chart renders an immutable supported SurrealDB image
The chart SHALL default the SurrealDB workload to the supported server version and its published OCI digest, and the rendered image reference SHALL include both values.

#### Scenario: Default chart render pins SurrealDB
- **WHEN** the chart is linted and rendered with its default values
- **THEN** every SurrealDB container image is `surrealdb/surrealdb:v3.2.4@sha256:51baed8709f57f67dcf04b30e3177db846803fa9342dae2be58c6fa5f8d59843`
