# Draft.2 schema family

Status: **contract checkpoint; runtime conformance is not claimed.** All schemas use JSON Schema 2020-12 and resolve locally. Their HTTPS `$id` values are stable identifiers, not a deployed hosting claim.

| Schema | Instance and export class |
|---|---|
| `collaboration-document.schema.json` | Top-level discriminator across every concrete draft.2 kind |
| `common.schema.json` | Shared IDs, immutable references, SkillRef, diagnostics, and private references |
| `agent-definition.schema.json` | Portable immutable AgentDefinition |
| `team-definition.schema.json` | Portable immutable TeamDefinition |
| `workflow-definition.schema.json` | Portable immutable WorkflowDefinition |
| `package-manifest.schema.json` | Portable manifest; files are limited to Agent/Team/Workflow definitions |
| `deployment-binding.schema.json` | Private installed DeploymentBinding |
| `representation-grant.schema.json` | Private authority-plane RepresentationGrant reference boundary |
| `conversion-report.schema.json` | Portable immutable field-level conversion evidence |
| `effective-binding-receipt.schema.json` | Private revisioned effective-binding evidence |
| `deployment-binding-template.schema.json` | Sanitized, non-executable template requiring fresh private binding |

Shape validity never grants activation. Semantic validation must additionally verify canonical and byte digests, exact lock closure, graphs, required capability/extension support, owner/workspace scope, current policy/grant state, recognized secret/private-authority exclusion, and effective runtime binding.

PackageManifest deliberately excludes DeploymentBinding, RepresentationGrant, ConversionReport, EffectiveBindingReceipt, and DeploymentBindingTemplate from `files[].kind`. Reports may accompany an export but are not executable package definitions.
