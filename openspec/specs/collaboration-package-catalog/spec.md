# collaboration-package-catalog Specification

## Purpose
TBD - created by archiving change uar-collaboration-package-catalog. Update Purpose after archive.

## Requirements

### Requirement: exact immutable package installation

The runtime SHALL accept canonical collaboration documents only under `urn:prometheus:uar:collaboration:0.1.0-draft.2`, validate each document through the profile's top-level discriminator and matching machine schema, verify every manifest file against its exact UTF-8 byte digest before parsing, verify each document's canonical self digest, close all immutable references, reject dependency cycles, and commit the package, definitions, conversion reports, compatibility projections, catalog revision, and command receipt as one atomic catalog generation. A portable package SHALL contain only AgentDefinition, TeamDefinition, and WorkflowDefinition entrypoints and dependencies; PackageManifest describes that package but SHALL NOT include private DeploymentBinding or RepresentationGrant records.

#### Scenario: idempotent package retry

- **WHEN** an authenticated owner repeats an install with the same command ID and identical request bytes
- **THEN** the runtime returns the original receipt without creating another catalog revision

#### Scenario: reused immutable version

- **WHEN** a package or definition reuses an existing ID and semantic version with a different digest
- **THEN** the runtime rejects the install as a conflict

#### Scenario: portable package contains private state

- **WHEN** a package contains a DeploymentBinding, RepresentationGrant, credential reference, connection reference, executable approval, authority token, or secret in any field including extensions, source descriptors, legacy sections, skill configuration, contracts, or free text
- **THEN** the runtime rejects the package without registering any definition or catalog revision

### Requirement: lossless compatibility projection

The runtime SHALL retain the complete canonical AgentDefinition, original source descriptor, complete compiler IR, original authoring sections, source identity, rename mapping, and authored-presence information while exposing an ordinary AgentArtifact compatibility projection. The canonical SkillRef SHALL preserve `id`, `version`, `digest`, `required`, `config`, `entrypoint`, and `requiredTools`. The Agent Spec v2 fields `model_requirements`, `prompt_dialect`, `rag_configuration`, `context_strategy`, and `api_harness` SHALL survive conversion. Every field SHALL receive a conversion disposition of `exact`, `translated`, `optional-unsupported`, or `required-unsupported`; unsupported required fields SHALL prevent binding and activation rather than being replaced by defaults.

#### Scenario: required skill becomes effective

- **WHEN** an AgentDefinition requiring a versioned and configured skill is compiled, persisted, loaded, privately bound, and run through the ordinary agent path
- **THEN** the effective binding receipt identifies the exact installed skill version and digest with its required configuration, or binding fails with a diagnostic at that SkillRef JSON Pointer

#### Scenario: required v2 behavior is unsupported

- **WHEN** an imported descriptor marks a non-default v2 field as required and the selected runtime binding cannot enforce that behavior
- **THEN** the runtime preserves the authored field and reports `required-unsupported` without admitting execution

#### Scenario: team definition is installed before execution exists

- **WHEN** a structurally and semantically valid TeamDefinition package is installed before durable TeamInstance execution is implemented
- **THEN** the catalog stores and returns it but reports that TeamInstance activation is unsupported

### Requirement: private revisioned deployment bindings

The runtime SHALL scope DeploymentBinding and RepresentationGrant records to the authenticated owner and explicit workspace, return the authenticated caller's opaque `bindingOwnerId` from collaboration capability discovery, persist protected-store references without secret values, and apply expected-revision and idempotent-command semantics. Clients SHALL use that returned identifier rather than deriving or guessing the runtime's private principal-key encoding. A binding that cites `representationGrantRefs` SHALL resolve a current private grant using the accepted C02 `grantId`, `issuerPrincipalId`, `subjectPrincipalId`, `revision`, `status`, `notBefore`, `expiresAt`, and `constraintDigest` authority vocabulary. Grant validation in this capability SHALL NOT create, infer, or exercise C17 human representation behavior.

#### Scenario: cross-workspace binding request

- **WHEN** the binding document workspace differs from the authenticated request workspace
- **THEN** the runtime rejects the request without exposing another workspace's binding or grant

#### Scenario: authenticated owner discovery

- **WHEN** an authenticated caller reads collaboration capabilities before constructing a deployment binding
- **THEN** the runtime returns the opaque owner identifier that the binding document must contain

#### Scenario: stale representation grant reference

- **WHEN** a binding references a grant whose current revision, status, validity interval, or constraint digest no longer matches
- **THEN** binding or effect admission fails with a redacted field-level diagnostic and does not reuse an earlier grant decision

### Requirement: additive administration surfaces

The runtime SHALL expose package preflight, install, retrieval, and portable export plus binding preflight, install, list, effective-binding receipt, and sanitized-template export through authenticated REST and MCP administration while preserving existing compiler and agent APIs. Capability discovery SHALL advertise the exact collaboration profile, supported document and schema versions, conversion-report version, binding features, and implemented runtime semantics without claiming durable team execution or C17 representation behavior.

#### Scenario: capability discovery

- **WHEN** a host reads the runtime capability document
- **THEN** it can distinguish draft.2 document import, immutable package catalog, private binding, sanitized template export, and ordinary-agent compatibility support from durable team execution and human representation support

#### Scenario: sanitized binding export

- **WHEN** an authorized owner exports an installed DeploymentBinding
- **THEN** the runtime emits a non-executable DeploymentBindingTemplate without owner, workspace, runtime instance, credential, connection, approval, secret, or RepresentationGrant values and reports every field that requires fresh installation binding

### Requirement: fixed legacy Agent Markdown adapter

UAR-AGENT-MD SHALL remain a legacy AgentDefinition input adapter rather than the canonical collaboration format. Its only accepted top-level heading SHALL be `# Agent: <name>`. Its fixed section headings SHALL be `## Metadata`, `## Identity`, `## UI (A2UI)`, `## Capabilities`, `## Skills`, `## Tools`, `## MCP Servers`, `## Knowledge Base`, `## Memory Model`, `## A2A Contracts`, `## Governance`, `## Budgets & Constraints`, `## Execution Model`, `## Observability`, `## Deployment Profiles`, `## Model Requirements`, `## Prompt Dialect`, `## RAG Configuration`, `## Context Strategy`, and `## API Harness`. The runtime SHALL NOT interpret `# Team:`, `# Workflow:`, `# Deployment Binding:`, or `# Representation Grant:` as canonical collaboration documents.

#### Scenario: unknown mandatory legacy section

- **WHEN** a legacy descriptor contains an unrecognized section declared mandatory by the source format
- **THEN** import fails with a diagnostic naming the source heading and target JSON Pointer rather than silently dropping the section

#### Scenario: omitted and explicit v2 defaults

- **WHEN** one legacy descriptor omits a v2 section and another explicitly authors its default value
- **THEN** their canonical source records preserve that authorship distinction through export and reimport

### Requirement: typed conversion and effective binding evidence

Every conversion report SHALL identify the source document ID, version, digest, revision and profile; target profile or harness; field JSON Pointer; disposition; stable reason code; redacted message; and effective binding reference and revision when one exists. Every successful binding SHALL emit an effective-binding receipt that records requested and effective semantics, exact resolved package, skill and model identities and digests, current policy and grant revisions, supported runtime capabilities, and all conversion diagnostics. A dotted field name or retained extension alone SHALL NOT count as effective-binding evidence.

#### Scenario: optional unsupported extension round trip

- **WHEN** a definition contains an unknown extension marked optional
- **THEN** import preserves its exact JSON value, reports `optional-unsupported` at its JSON Pointer, and export reproduces it without claiming runtime support

#### Scenario: unknown required extension

- **WHEN** a definition contains an unknown required capability or extension
- **THEN** the conversion report marks it `required-unsupported`, no successful binding receipt is emitted, and activation is refused

### Requirement: lossless migration and export compatibility

Existing supported legacy descriptors SHALL remain readable. Canonical import and export SHALL preserve immutable definition identity, content digest, complete supported semantics, optional unsupported values, source provenance, and exact dependency locks. Native or downgraded export SHALL refuse any required semantic that the target cannot represent. Portable export SHALL fail closed if private authority or secret material is detected; redaction SHALL occur only through the explicit DeploymentBindingTemplate export.

#### Scenario: canonical export and reimport

- **WHEN** an installed canonical package is exported and reimported without a semantic edit
- **THEN** every definition retains the same semantic version and canonical content digest and the dependency lock resolves to the same exact identities

#### Scenario: required unsupported native export

- **WHEN** a target harness or older UAR profile cannot represent a required definition field
- **THEN** export is refused with a conversion report at the unsupported field rather than reducing it to prompts or defaults

#### Scenario: immutable alias independence

- **WHEN** a mutable alias changes after a package and binding resolve exact definition and skill digests
- **THEN** the installed binding continues to use the recorded immutable identities until an explicit revisioned rebind

### Requirement: completed-path conformance

UAR SHALL claim draft.2 runtime conformance only after one completed-phase integration gate observes the real production path from legacy and canonical import through catalog persistence, retrieval, private binding, ordinary agent admission and execution, and portable export/reimport. The gate SHALL also observe required-unsupported refusal, private-authority exclusion, immutable resolution, and downgrade refusal. Schema validation, source inspection, compatibility projection storage, or mock-only tests SHALL NOT establish runtime conformance.

#### Scenario: ordinary agent round trip

- **WHEN** the completed C03 implementation is evaluated for promotion
- **THEN** the acceptance receipt identifies the exact source revision, schema/profile versions, package and definition digests, policy and grant revisions, field-level binding outcomes, and observed ordinary-agent execution result

### Requirement: Deployment bindings target the live runtime

An installed deployment binding MUST target the live configured UAR instance,
MUST require only capabilities implemented by that instance, and MUST retain
credential and connection identifiers solely as opaque references. Its effective
receipt MUST record the admitted service binding and MUST be revalidated before
activation and protected tool claims.

#### Scenario: Binding targets another runtime

- **WHEN** `runtimeInstanceId` differs from the live configured instance
- **THEN** preflight records a required-unsupported diagnostic
- **AND** activation is not admitted.

#### Scenario: Required capability is absent

- **WHEN** a binding requires a capability the live runtime does not advertise
- **THEN** preflight identifies the unsupported capability
- **AND** activation is not admitted.

#### Scenario: Runtime changes after binding installation

- **WHEN** the effective receipt no longer matches the live instance at run or tool admission
- **THEN** UAR refuses activation rather than using the stale receipt.
