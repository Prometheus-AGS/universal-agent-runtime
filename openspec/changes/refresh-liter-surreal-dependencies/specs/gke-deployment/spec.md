## ADDED Requirements

### Requirement: Kubernetes persistence manifests render an immutable supported SurrealDB image
Kustomize and OpenTofu deployment inputs SHALL select the supported SurrealDB server version by its exact tag and published OCI digest.

#### Scenario: Kubernetes deployment render pins SurrealDB
- **WHEN** the base and environment deployment configurations are rendered or planned
- **THEN** every SurrealDB workload image is `surrealdb/surrealdb:v3.2.4@sha256:51baed8709f57f67dcf04b30e3177db846803fa9342dae2be58c6fa5f8d59843`
