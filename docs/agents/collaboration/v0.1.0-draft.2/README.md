# UAR Collaboration Document Profile 0.1.0-draft.2

Status: **contract checkpoint; runtime conformance is not claimed.** Normative owner: Universal Agent Runtime (UAR). Profile identifier: `urn:prometheus:uar:collaboration:0.1.0-draft.2`. This project-owned identifier is not an external standards registration.

MUST and MUST NOT are normative. A schema-valid value proves document shape only. Activation additionally requires immutable dependency closure, supported required semantics, a current private binding, current policy, and current private authority. The draft.1 tree remains an immutable predecessor.

## Status and Conformance

This checkpoint defines document-profile conformance for AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, private DeploymentBinding, private RepresentationGrant, ConversionReport, EffectiveBindingReceipt, and DeploymentBindingTemplate. It does not establish runtime conformance, durable team execution, or C17 human representation behavior.

The schema family is rooted at [schemas/collaboration-document.schema.json](schemas/collaboration-document.schema.json). Valid examples and intentionally invalid fixtures are documentation evidence only. A future runtime claim requires the completed production-path integration gate described by the owning OpenSpec change.

## Version Domains

These versions are independent:

- collaboration profile `0.1.0-draft.2`;
- definition semantic version and canonical content digest;
- package exact-byte digest and immutable lock;
- mutable DeploymentBinding, RepresentationGrant, and EffectiveBindingReceipt revision;
- runtime capability name/version;
- AG-UI/A2UI/A2A envelope version and event cursor.

Compatibility in one domain does not imply compatibility in another. A draft.1 input is migrated explicitly and retains its source profile and digest; it is never relabeled as draft.2.

## Common Document Envelope

Portable definitions, PackageManifest, ConversionReport, and DeploymentBindingTemplate use the draft.2 profile, closed `kind`, stable ID, semantic version, canonical `contentDigest`, provenance, required capabilities, and namespaced extensions. Private installed records use the same profile but carry a private `exportClass` and mutable revision where their schema requires it.

Definition and package digests are `sha256:` lowercase hexadecimal digests of RFC 8785 canonical JSON with only the top-level `contentDigest` omitted. Package `byteDigest` values hash exact UTF-8 file bytes. Private revisioned records do not become immutable definitions merely because they have a content digest.

## AgentDefinition

AgentDefinition is portable and immutable. It retains source identity, rename mapping, authored field presence, complete source descriptor, original legacy sections, contracts, full SkillRef values, model/context/limit requests, and the five Agent Spec v2 requirements.

Canonical SkillRef requires `id`, `version`, `digest`, `required`, `config`, `entrypoint`, and `requiredTools`. The five v2 fields are `modelRequirements`, `promptDialect`, `ragConfiguration`, `contextStrategy`, and `apiHarness`; each carries `{ required, value }`. Retention alone is not support: a required field must appear in an EffectiveBindingReceipt with an enforcing runtime component or block activation.

## TeamDefinition

TeamDefinition is portable and immutable. It pins agent or subteam definitions by ID, version, and digest; declares unique roles, bounded cardinality, one coordinator role, communication edges, allowed workflows, routing, limits, budget, and input/output contracts. Team graphs must be acyclic and within root ceilings.

Registration does not create a TeamInstance. This checkpoint does not claim durable team activation.

## WorkflowDefinition

WorkflowDefinition is portable and immutable. It describes a finite task dependency DAG, role requirements, constrained input mappings, outputs, effect classification, approval class, retry bounds, and completion criteria. It is not an executable expression language.

Workflow registration does not prove that every role or effect can execute. Required unsupported semantics block activation.

## DeploymentBinding

DeploymentBinding is private, owner/workspace/runtime-scoped, and revisioned. It resolves one exact package to model connections, complete skill bindings, storage references, current policy, effective limits/budget, context grants, RepresentationGrant references, and an optional EffectiveBindingReceipt reference.

Credential and connection values remain in protected stores; the document contains references only. DeploymentBinding is forbidden from portable packages and ordinary package export.

A binding used only for a TeamDefinition planning board may have an empty `modelBindings` array. Planning does not start a model turn. Ordinary AgentDefinition activation still requires its authored model requirements to resolve to configured models; an empty array does not admit ordinary execution. This additive draft.2 clarification permits fresh offline team planning without a fabricated model credential.

## RepresentationGrant

RepresentationGrant is private authority-plane state aligned with `afc.governed-effect/1` version `1.1.0`. It records grant ID, issuer and subject principals, grantee AgentInstance, organization/office/purpose, scoped audience/action/resource/data permissions, approval/disclosure requirements, protected evidence references, revision, status, validity, constraint digest, revocation, retention, offboarding, and restrictions.

It contains no token, credential, signature, raw consent evidence, or secret. This profile defines storage, reference, current-state validation, and export exclusion only. It does not issue grants, collect consent, authorize impersonation, delegate human approval, or implement C17 representation actions.

## PackageManifest

PackageManifest is portable and immutable. Its `files[].kind` is closed to AgentDefinition, TeamDefinition, and WorkflowDefinition. It verifies exact bytes and canonical content, closes all immutable references, records capability declarations, and uses `exact-version-and-digest` resolution.

DeploymentBinding, RepresentationGrant, EffectiveBindingReceipt, credentials, connections, executable approvals, authority tokens, and secrets are forbidden anywhere in a package, including extensions, source descriptors, legacy sections, skill configuration, contracts, and free text.

## Digests and Immutable Resolution

The importer verifies byte digests before JSON parsing, then canonical content digests, document identity, and lock closure. Reusing an ID and semantic version with a different digest is a conflict. Mutable aliases are never resolved during activation; installed bindings continue to use recorded immutable IDs, versions, and digests until an explicit revisioned rebind.

JSON Schema cannot compute digests or prove graph closure. These remain semantic checks and are represented by the [semantic-invalid fixture set](fixtures/semantic-invalid/).

## Legacy Import and Conversion Diagnostics

UAR-AGENT-MD remains a legacy AgentDefinition input adapter. Its exact v1.1 and v2 headings are frozen in [compatibility.md](compatibility.md). There is no Team, Workflow, DeploymentBinding, or RepresentationGrant Markdown dialect.

Every conversion emits an immutable ConversionReport with source identity/profile, target profile or harness, RFC 6901 JSON Pointer, disposition (`exact`, `translated`, `optional-unsupported`, or `required-unsupported`), stable reason code, redacted message, and optional effective binding reference. Unknown required semantics block activation; unknown optional values survive export unchanged.

## Effective Binding and Activation

EffectiveBindingReceipt is private, revisioned evidence of requested versus enforced semantics. It identifies the exact package, definition, skill, and model resolutions; current policy and grant revisions; runtime capabilities; field diagnostics; and admission result.

Retaining canonical JSON in an extension or compatibility projection is not effective binding. Draft.2 runtime conformance requires the ordinary agent admission path to consume the receipt. Team execution remains unsupported by this checkpoint.

## Export and Private Authority Separation

Canonical package export preserves immutable identities, exact locks, optional unsupported values, provenance, and conversion reports. It refuses required semantics unsupported by the target. It fails closed when recognized private authority or secret material is present; portable content is not silently redacted because redaction would change semantics and digests.

The separate DeploymentBindingTemplate export is non-executable. It removes owner, workspace, runtime instance, credential, connection, policy decision, approval, secret, and grant values and lists every JSON Pointer requiring a fresh private binding.

## Compatibility, Upgrade, and Rollback

Draft.1 and supported legacy descriptors remain readable through explicit migration. The source profile, source digest, normalized draft.2 digest, and field diagnostics remain inspectable. Existing bindings remain pinned until deliberate migration.

Provider support and its acceptance receipt precede downstream enforcement. An older runtime must refuse bindings that require draft.2 semantics. Rollback may require an explicit compatible rebind and never reverses external effects already performed.

## Conformance Fixtures

[examples/](examples/) contains one valid instance for every concrete draft.2 schema branch plus a full SkillRef and optional-extension example. [fixtures/schema-invalid/](fixtures/schema-invalid/) contains structural failures. [fixtures/semantic-invalid/](fixtures/semantic-invalid/) contains schema-valid values that fail digest, lock, capability, or private-authority rules. [fixtures/legacy/](fixtures/legacy/) fixes the accepted Agent Markdown headings and omitted-versus-explicit v2 authoring distinction.

[fixture-manifest.json](fixture-manifest.json) maps each fixture to its expected validation layer and outcome. [sources.md](sources.md) and [source-receipt.json](source-receipt.json) pin the inputs used for this checkpoint. These artifacts are not a runtime integration receipt.
