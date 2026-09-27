## Why

The Agent Fabric C03 initiative requires authored agent and collaboration semantics to survive UAR compilation, catalog storage, private binding, export, and actual execution. The current collaboration catalog preserves canonical documents, but it does not yet prove that complete skill references and Agent Spec v2 fields become effective runtime bindings, and its draft profile lacks typed conversion, sanitized export, and private representation-grant artifacts.

## What Changes

- Advance the canonical collaboration JSON profile from `urn:prometheus:uar:collaboration:0.1.0-draft.1` to a new `0.1.0-draft.2` contract without changing the accepted rule that UAR-AGENT-MD is only a legacy AgentDefinition adapter.
- Freeze the exact legacy Agent Markdown heading vocabulary and normalize legacy v1.1/v2 descriptors into canonical AgentDefinition records while preserving source identity, authored presence, the complete compiler IR, and original authoring sections.
- Define matching machine schemas for AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, private DeploymentBinding, private RepresentationGrant, conversion reports, effective-binding receipts, sanitized deployment-binding templates, and the top-level document discriminator.
- Define a canonical SkillRef superset that preserves identity, semantic version, digest, required status, configuration, entrypoint, and required tools; a required field either binds effectively or blocks activation with a field-level diagnostic.
- Keep RepresentationGrant as a private authority-plane record aligned with the accepted governed-effect grant vocabulary. This change defines storage, reference, validation, and export exclusion only; it does not implement C17 human representation behavior.
- Add lossless legacy migration and canonical import/export behavior, immutable dependency closure, downgrade refusal, and fixtures that prove packages contain neither private grants nor secrets.
- Add one final integration gate covering compile, persistence, retrieval, private binding, ordinary agent execution, and export/reimport at the completed phase boundary.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `collaboration-package-catalog`: Extend the accepted immutable catalog and private binding contract with the draft.2 schema family, lossless legacy conversion, full effective bindings, private RepresentationGrant references, sanitized export, and end-to-end conformance evidence.

## Impact

This repository-scoped change implements initiative `afc-c03-lossless-definitions-and-collaboration-document-profile`, tasks C03.1-C03.3, in Universal Agent Runtime only. It consumes C01 contracts `afc.convergence-contract.v1` and `afc.identity-state-action.v1` version `1.0.0`, accepted I1 UAR merge `79414bb7e134dad45330008999ee1dca999d44af` with collaboration implementation `676f995c73dc7dacdce71c929ffac88ac3618bb0`, and C02 `afc.governed-effect/1` version `1.1.0` at UAR `59df10a102c5d46da9db5983a73fbcf20ca09590`.

Affected surfaces are the UAR collaboration profile and schemas, compiler/IR conversion, catalog validation and persistence, private deployment bindings and grant references, capability discovery, REST/MCP administration, AgentArtifact projection, and ordinary agent admission. Existing compiler and agent APIs remain readable. Durable TeamInstance execution, C17 representation behavior, BossFang translation, and full/mini consumer adoption remain separate changes and gates.

Runtime UX receives structured, redacted diagnostics and effective-binding receipts through existing administration surfaces. No new provider route or realtime protocol is introduced; capability discovery must expose the supported profile and binding features without claiming team execution. KBD workflow state is updated only by the owning Agent Fabric convergence orchestrator after repository evidence is linked; this change does not move the unrelated local active phase.
