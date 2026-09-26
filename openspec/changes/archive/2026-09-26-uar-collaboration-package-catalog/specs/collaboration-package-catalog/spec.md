# Collaboration package catalog

## ADDED Requirements

### Requirement: exact immutable package installation

The runtime SHALL verify every manifest file against its exact UTF-8 byte digest before parsing, verify each document's canonical self digest, close all immutable references, reject dependency cycles, and commit the package, definitions, compatibility projections, catalog revision, and command receipt as one atomic catalog generation.

#### Scenario: idempotent package retry

- **WHEN** an authenticated owner repeats an install with the same command ID and identical request bytes
- **THEN** the runtime returns the original receipt without creating another catalog revision

#### Scenario: reused immutable version

- **WHEN** a package or definition reuses an existing ID and semantic version with a different digest
- **THEN** the runtime rejects the install as a conflict

### Requirement: lossless compatibility projection

The runtime SHALL retain the complete canonical AgentDefinition and original source descriptor while exposing an ordinary AgentArtifact compatibility projection. Unsupported required fields SHALL remain visible as field-level diagnostics and SHALL prevent activation claims.

#### Scenario: team definition is installed before execution exists

- **WHEN** a structurally and semantically valid TeamDefinition package is installed during I1
- **THEN** the catalog stores and returns it but reports that TeamInstance activation is unsupported

### Requirement: private revisioned deployment bindings

The runtime SHALL scope deployment bindings to the authenticated owner and explicit workspace, return the authenticated caller's opaque `bindingOwnerId` from collaboration capability discovery, persist protected-store references without secret values, and apply expected-revision and idempotent-command semantics. Clients SHALL use that returned identifier rather than deriving or guessing the runtime's private principal-key encoding.

#### Scenario: cross-workspace binding request

- **WHEN** the binding document workspace differs from the authenticated request workspace
- **THEN** the runtime rejects the request without exposing another workspace's binding

#### Scenario: authenticated owner discovery

- **WHEN** an authenticated caller reads collaboration capabilities before constructing a deployment binding
- **THEN** the runtime returns the opaque owner identifier that the binding document must contain

### Requirement: additive administration surfaces

The runtime SHALL expose package preflight/install/retrieval and binding preflight/install/list operations through authenticated REST and MCP administration while preserving existing compiler and agent APIs.

#### Scenario: capability discovery

- **WHEN** a host reads the runtime capability document
- **THEN** it can distinguish collaboration package catalog and private binding support from durable team execution support
