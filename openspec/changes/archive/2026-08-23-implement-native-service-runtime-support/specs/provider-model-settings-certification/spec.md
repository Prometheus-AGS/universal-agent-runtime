## ADDED Requirements

### Requirement: YAML-defined providers are catalog-enriched before registration
Before a YAML-defined provider is first stored, UAR SHALL resolve omitted catalog-backed API-key environment names, endpoint URLs, provider metadata, and model metadata through the same embedded-catalog enrichment used by the LLM configuration path. Persisted API/UI-created provider settings remain authoritative after initial seeding.

#### Scenario: YAML provider omits catalog metadata
- **WHEN** a known provider is declared in YAML without duplicating its embedded catalog URL, key variable, or model metadata
- **THEN** UAR enriches the definition before first registration and does not overwrite a later persisted operator edit
