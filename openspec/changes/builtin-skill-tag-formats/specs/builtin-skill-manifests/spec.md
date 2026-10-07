## ADDED Requirements
### Requirement: Shipped metadata tags support scalar and array formats
Builtin skill manifest metadata.tags SHALL accept a string array or a comma-separated scalar string. Array values and ordering SHALL remain unchanged. Scalar labels SHALL split on commas, trim surrounding whitespace and omit empty separator segments while preserving internal spaces. Omitted tags SHALL remain an empty vector. The parser SHALL NOT alter any unrelated frontmatter or derive tool authority from tags.

#### Scenario: Packaged scalar manifest
- **WHEN** a shipped skill declares metadata.tags as ui, ux, prometheus-ui-review
- **THEN** the parser accepts the tags as three labels and does not reject the manifest solely because tags were a scalar

#### Scenario: Existing array manifest
- **WHEN** metadata.tags is an array of strings
- **THEN** existing string values, ordering, whitespace and duplicates are retained

#### Scenario: No metadata tags
- **WHEN** a manifest omits metadata.tags
- **THEN** metadata tags remain empty without changing any identity, digest, location, tool or refresh semantics
