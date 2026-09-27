## Context

See [proposal.md](proposal.md) for motivation and the [collaboration-package-catalog delta](specs/collaboration-package-catalog/spec.md) for normative behavior. This repository already has an official `0.1.0-draft.1` collaboration document set, an immutable package catalog, private revisioned DeploymentBinding records, REST/MCP administration, and an AgentArtifact compatibility projection.

This change consumes these immutable predecessors:

| Contract | Exact input | Constraint carried into C03 |
|---|---|---|
| C01 convergence and identity | `afc.convergence-contract.v1` and `afc.identity-state-action.v1`, version `1.0.0` | Definition is immutable; DeploymentBinding is private and revisioned; Instance, Task, Attempt, and Run remain distinct; version domains do not substitute for one another. |
| I1 collaboration provider | UAR merge `79414bb7e134dad45330008999ee1dca999d44af`; implementation `676f995c73dc7dacdce71c929ffac88ac3618bb0`; draft publication `cbf5d560f578069e908faa0ba08b133121241cc5` | Canonical collaboration documents are JSON. UAR-AGENT-MD remains a legacy AgentDefinition adapter. Team definitions may be registered, but team activation is unsupported. |
| C02 governed effects | `afc.governed-effect/1`, version `1.1.0`; UAR `59df10a102c5d46da9db5983a73fbcf20ca09590` | Grant identity, issuer, subject, revision, status, validity, and constraint digest are current admission inputs. Approval and grant checks are revalidated after waits. |
| C03 initiative | `afc-c03-lossless-definitions-and-collaboration-document-profile`, tasks C03.1-C03.3 | Required authored semantics remain effective through persistence and binding or fail with field-level diagnostics; private authority never enters portable packages. |

The observed gaps are concrete: the runtime uses handwritten shape checks rather than the published JSON schemas; conversion diagnostics contain only a dotted field name, disposition, and message; AgentArtifact projection reduces SkillRef to preferred IDs; the draft has no RepresentationGrant, conversion-report, effective-binding-receipt, sanitized binding-template, or top-level discriminator schemas; and portable-authority validation is a recursive key blacklist even though arbitrary extension values are allowed.

## Goals / Non-Goals

**Goals:**

- Make one draft.2 JSON schema family authoritative for documented shape and runtime validation.
- Preserve legacy v1.1 and v2 authoring without introducing another collaboration representation.
- Prove full SkillRef and Agent Spec v2 semantics at the private binding and ordinary-agent execution boundaries.
- Define typed, redacted conversion and binding evidence that downstream adapters can consume.
- Define a private RepresentationGrant storage/reference boundary that composes with C02.
- Provide lossless canonical import/export, explicit sanitized private-binding export, and immutable dependency behavior.
- Produce one final integrated acceptance receipt at the complete implementation boundary.

**Non-Goals:**

- Durable TeamInstance, task, messaging, or workflow execution; those remain I2/C06/C09 work.
- Human executive representation, consent collection, or grant issuance; those remain C17 work.
- BossFang, full skill-pack, mini, or native-harness adapter implementation.
- A Team, Workflow, DeploymentBinding, or RepresentationGrant Markdown format.
- New providers, services, ports, event protocols, frontend state stores, or dependency versions.

## Decisions

### 1. Publish an additive draft.2 JSON profile

Create `docs/agents/collaboration/v0.1.0-draft.2/` rather than changing draft.1 in place. Every draft.2 AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, DeploymentBinding, and RepresentationGrant uses `profile: "urn:prometheus:uar:collaboration:0.1.0-draft.2"`; each `$id` is new and versioned. Definition semantic versions/content digests, mutable binding/grant revisions, runtime capability versions, and event/cursor versions remain independent.

The normative document headings are fixed as:

1. `Status and Conformance`
2. `Version Domains`
3. `Common Document Envelope`
4. `AgentDefinition`
5. `TeamDefinition`
6. `WorkflowDefinition`
7. `DeploymentBinding`
8. `RepresentationGrant`
9. `PackageManifest`
10. `Digests and Immutable Resolution`
11. `Legacy Import and Conversion Diagnostics`
12. `Effective Binding and Activation`
13. `Export and Private Authority Separation`
14. `Compatibility, Upgrade, and Rollback`
15. `Conformance Fixtures`

The schema inventory is:

- `common.schema.json`
- `collaboration-document.schema.json`
- `agent-definition.schema.json`
- `team-definition.schema.json`
- `workflow-definition.schema.json`
- `package-manifest.schema.json`
- `deployment-binding.schema.json`
- `representation-grant.schema.json`
- `conversion-report.schema.json`
- `effective-binding-receipt.schema.json`
- `deployment-binding-template.schema.json`

`collaboration-document.schema.json` selects the exact schema by `profile` and `kind`; callers do not pass an independent kind that could disagree with the document. Runtime snapshot and event/action envelope schemas keep their existing independent version domains.

Alternative considered: amend draft.1 schemas in place. Rejected because changed required fields and new document kinds would make the same stable `$id` identify incompatible shapes.

### 2. Keep Markdown as one fixed legacy adapter

The legacy adapter accepts `# Agent: <name>` and the 20 exact H2 headings listed in the spec delta. The first 15 are the v1.1 vocabulary and the last five are Agent Spec v2 additions. Aliases currently accepted for compatibility may normalize to these canonical names, but conversion records the source spelling. Unknown mandatory headings fail; explicitly optional unknown content is retained in original authoring and diagnosed.

The parser records authored presence separately from parsed/defaulted values. This preserves omission versus an explicit default. Metadata source ID and version remain canonical identity inputs; a heading-derived slug is retained only as an explicit rename/compatibility mapping.

Alternative considered: define parallel Team and Workflow Markdown grammars. Rejected because it contradicts I1, duplicates the JSON schemas, and creates another lossy conversion surface.

### 3. Use one canonical SkillRef superset

Draft.2 SkillRef has these required canonical members:

| Field | Canonical form | Binding rule |
|---|---|---|
| `id` | stable skill ID | Resolve exactly; never substitute a display name. |
| `version` | semantic version | Resolve the requested version, not a mutable latest alias. |
| `digest` | immutable content digest | Installed bytes must match. |
| `required` | boolean | Missing or unsupported when true blocks binding. |
| `config` | JSON object | Preserve unchanged and apply to the bound instance. |
| `entrypoint` | string or null | Preserve legacy entrypoint; null is explicit when absent. |
| `requiredTools` | sorted unique skill tool IDs | Each tool must be available within the effective tool policy. |

Authored-presence metadata distinguishes missing legacy members from explicit null, empty, or default values. The importer does not manufacture execution support. AgentArtifact projection retains the canonical definition reference, while the private binding resolves and records the exact installed skill identity, config, and effective tool intersection.

Alternative considered: keep mapping only skill IDs into `policy.skills.prefer`. Rejected because preference neither pins a dependency nor enforces required/configured behavior.

### 4. Make schemas and semantic validators one runtime pipeline

Import first validates UTF-8 bytes, JSON parse constraints, the top-level discriminator, and the selected draft.2 schema. It then runs existing semantic checks for digests, immutable lock closure, graphs, capability/extension support, and authority separation. Rust domain types represent the same closed shapes; handwritten validators remain only for relational, temporal, digest, and policy checks that JSON Schema cannot express.

Draft.1 input remains readable through an explicit draft.1-to-draft.2 migration adapter. The original source bytes, profile, compiler IR, and authoring sections remain attached to the immutable record, while the normalized draft.2 content gets its own identity and migration receipt. Draft.1 is never silently relabeled as draft.2.

Alternative considered: keep documentation schemas outside runtime and manually duplicate their constraints. Rejected because the observed schema/runtime drift would remain possible.

### 5. Separate conversion evidence from effective-binding evidence

`ConversionReport` is immutable with the converted definition. It contains source ID/version/digest/revision/profile, target profile or harness, and diagnostics addressed by RFC 6901 JSON Pointer. Each diagnostic carries disposition, stable reason code, redacted message, and an optional effective binding reference/revision.

`EffectiveBindingReceipt` is private and revisioned with the DeploymentBinding. It records requested versus effective definition fields, package/definition/skill/model identities and digests, policy revision, representation grant ID/revision/constraint digest, runtime capability set, and conversion diagnostics. The ordinary agent admission path consumes this receipt; retaining JSON under an extension does not count as support.

The five v2 fields must each reach an enforcing runtime component or yield `required-unsupported`. Existing conformance for model requirements, prompt dialect, and context strategy is reused. RAG configuration and API harness receive the same rule; this change cannot label them supported merely because their values are stored.

Alternative considered: expand the existing three-field diagnostic in place and continue inferring execution from projection storage. Rejected because it conflates conversion fidelity with runtime enforcement.

### 6. Model RepresentationGrant as private mutable authority state

RepresentationGrant is a private `kind` under the draft.2 schema family, stored outside packages and immutable definition maps. It uses a stable grant ID plus monotonic revision, status, issuer and subject principals, grantee AgentInstance ID, represented organization/office, purpose, audience, action/resource/data scopes, approval/disclosure requirements, `notBefore`, `expiresAt`, `constraintDigest`, protected consent/organizational-evidence references, revocation metadata, retention/deletion/offboarding rules, and forbidden-claim restrictions.

No schema field accepts a token, credential, signature, raw consent evidence, or secret. DeploymentBinding contains only `representationGrantRefs`. Binding and effect admission resolve the current private record and reuse C02 grant identity/revision/status/validity/digest checks. This change adds no grant issuer, consent workflow, human impersonation, delegated approval, or representation action.

Alternative considered: make RepresentationGrant a portable definition or extension payload. Rejected because authored documents cannot confer installed authority and C02 requires current private authority at effect time.

### 7. Export portable definitions and private templates through separate operations

Canonical package export emits AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, lock, and a conversion report. It preserves unknown optional extension values and refuses required semantics unsupported by the requested target. It never exports DeploymentBinding or RepresentationGrant.

An explicit binding-template export emits `DeploymentBindingTemplate`, which is non-executable and omits owner, workspace, runtime instance, credential, connection, policy decision, approval, secret, and grant values. Its report names every field that requires fresh installation binding. Reimport cannot activate the template until new private state is supplied.

Authority separation uses closed schemas, recursive rejection of reserved private authority shapes in every extensible JSON value, and the runtime's redaction/secret classification at import/export. Registered secret values are compared without logging their plaintext. Private-source values carry provenance into serialization so export fails closed. The acceptance fixtures include private values in extensions, source descriptors, legacy sections, skill config, contracts, and free text. Arbitrary unregistered prose cannot be mathematically proven secret; the contract is limited to the runtime's recognized secret and authority classes, and the acceptance receipt identifies the classes exercised.

Alternative considered: silently redact portable package content. Rejected because redaction changes immutable semantics and digests. Only the explicit template operation may remove private binding fields, and it reports each removal.

### 8. Version capability discovery additively

Keep existing v1 capability names while draft.1 support remains. Add `collaboration_definition_packages_v2`, `collaboration_deployment_bindings_v2`, `collaboration_conversion_reports_v1`, and `collaboration_representation_grant_refs_v1` only with their owning behavior. The structured collaboration capability response lists accepted profiles, document kinds, schema IDs, export classes, and activation limits. It never advertises team execution or human representation.

Alternative considered: reinterpret existing `*_v1` capability names to mean draft.2. Rejected because a caller could mistake shape compatibility for execution compatibility.

### 9. Keep implementation ownership bounded and ordered

One writer owns each surface in this sequence:

1. **Profile/schema owner:** `docs/agents/collaboration/v0.1.0-draft.2/**` and schema fixtures only.
2. **Domain/validation owner:** `src/uar/domain/collaboration.rs` and `src/uar/compiler/collaboration/validation/**`; align types, schema loading, diagnostics, and semantic checks.
3. **Legacy/projection owner:** `src/uar/compiler/{ir.rs,parser.rs,to_artifact.rs,storage.rs}` and collaboration projection; preserve source/IR and build effective skill/v2 inputs.
4. **Private-state owner:** `src/uar/compiler/collaboration/{bindings.rs,storage.rs,service.rs}`; add grant records, binding receipts, migration, and export transactions.
5. **Administration owner:** `src/uar/api/{capabilities.rs,collaboration.rs,administration_capabilities.rs}` and the existing collaboration MCP methods; expose typed results without another service or port.
6. **Integration-fixture owner:** dedicated collaboration fixtures and one production-path integration gate after all production owners finish.

Production work is serialized where these modules overlap. Reviewer/verifier work starts only after all six implementation groups are complete.

## Risks / Trade-offs

- **Draft.2 and draft.1 identities can be conflated** → preserve both source and normalized profile/digests and issue a migration receipt; never rewrite draft.1 records in place.
- **Schema and Rust types can drift again** → register the versioned schema set in the runtime pipeline and exercise every document kind with valid and invalid fixtures.
- **Existing descriptors lack skill entrypoint/tool fields** → represent absence explicitly and retain authored presence; required missing semantics block rather than being guessed.
- **RAG/API harness storage may be mistaken for support** → require an enforcing component and effective-binding receipt before a supported disposition.
- **Private records can leak through arbitrary JSON** → use closed schemas, recursive authority classification, secret provenance, fail-closed portable export, and explicit redacted-template export.
- **False positives in recognized secret scanning can block export** → return a redacted field pointer and reason code so an operator can remove or reclassify content; never log the value.
- **Adding RepresentationGrant could be read as C17 delivery** → capability names and receipts state reference validation only; no issuance or representation action enters this change.
- **An older binary cannot safely interpret draft.2 bindings** → record a minimum reader capability and refuse downgrade while draft.2 private revisions are active.
- **Cross-repository consumers may enforce before UAR is ready** → publish provider behavior and acceptance receipt first; D-MINI and other consumer checkpoints remain blocked.

## Migration Plan

1. Publish draft.2 documents, schemas, examples, invalid fixtures, and a source receipt while leaving draft.1 immutable.
2. Add draft.2 domain types and schema-backed validation. Continue accepting draft.1 and legacy descriptors through explicit adapters.
3. Add immutable migration records and conversion reports; do not mutate existing catalog identities or installed bindings.
4. Add effective binding receipts and full SkillRef/v2 binding. Do not advertise v2 capabilities until ordinary-agent admission consumes the receipt.
5. Add private RepresentationGrant storage/reference validation and sanitized DeploymentBindingTemplate export. Existing bindings without grant refs remain valid.
6. Add REST/MCP/capability fields after the underlying behavior exists.
7. Complete all fixtures, then run one final integration gate over the real compile → catalog → retrieval → binding → ordinary execution → export/reimport path. Link its exact receipt to C03.1-C03.3.
8. The convergence owner may then adopt the exact UAR revision in downstream work. This repository change does not update external KBD state.

Rollback keeps draft.1 records and APIs intact. Before returning to a binary without draft.2 support, quiesce new installs, export operator-readable receipts, and verify no active binding requires draft.2 semantics. If any does, downgrade is refused until the operator explicitly rebinds to a compatible immutable definition; rollback never rewrites already executed external effects.
