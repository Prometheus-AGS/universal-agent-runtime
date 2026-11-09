## ADDED Requirements

### Requirement: Classify explicit Cargo features

The support matrix SHALL catalog every explicit Cargo feature other than the `default` selector. It SHALL distinguish verification-only flags from product bundles and record the packaging scope of architecture-specific implementation features.

#### Scenario: Verification and packaging features are cataloged

- **WHEN** the support matrix catalogs the Agent Fabric and native admission acceptance flags, test probes, and the tract embedding backend
- **THEN** the acceptance and test flags are classified as development-only and excluded from product bundles
- **AND** the tract backend is classified as internal and scoped to Intel macOS sidecar packaging
- **AND** these entries preserve the historical verification date and existing release qualification claims
