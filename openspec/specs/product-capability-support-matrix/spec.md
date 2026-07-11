# product-capability-support-matrix Specification

## Purpose
TBD - created by archiving change publish-capability-support-matrix. Update Purpose after archive.
## Requirements
### Requirement: Support claims are evidence backed
Every advertised capability, provider, feature combination and platform SHALL have a maturity classification and verification reference.

#### Scenario: Catalog-only provider
- **WHEN** a provider exists in the catalog but lacks integration evidence
- **THEN** it is labeled catalog/community support and not production-certified

#### Scenario: Mobile claim
- **WHEN** mobile packaging and platform tests are absent
- **THEN** mobile is labeled experimental rather than equivalent to web/desktop
