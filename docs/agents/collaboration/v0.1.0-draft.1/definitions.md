# Definitions, installed bindings and catalog

This draft extends RFC-0001 rather than replacing its required sections. JSON Schema files under schemas/ define field names and structural types; the semantic requirements here are additional. A validator MUST distinguish shape acceptance, dependency resolution and activation admission.

## Five identity layers

| Layer | Lifetime | Authority |
|---|---|---|
| Definition | Immutable version/digest, reusable | Requests capabilities; grants none |
| Deployment binding | Owner/workspace/runtime-specific revision | References current host-managed credentials/policy |
| Team/agent instance | Durable across bounded runs | Stable identity; each activation reacquires authority |
| Task | Durable work contract with dependency DAG | Assigned/claimed by UAR, not by a model message |
| Attempt | One execution try with lease epoch/run association | Current scoped admission, budget and effect context |

IDs MUST remain stable across renames; display labels are not identifiers. Definition dependencies MUST resolve by ID, version and digest before install. A signature authenticates provenance; it does not confer permission. Resolved immutable packages MUST be retained for existing bindings until those bindings are deliberately migrated. Runtime policy changes can restrict already bound definitions immediately.

## AgentDefinition

An agent document retains its original complete descriptor and declares role/when-to-use, input/output contracts, skill requirements, model capability requirements, allowed child definitions, context selections and limits. A child reference authorizes no spawn by itself. Every spawn still passes membership, owner/workspace and admission checks. Model requirements express capabilities; credentials and deployment-specific model connections belong to the binding.

Full SkillRef semantics include ID, version, required flag and configuration. Entrypoint/tool requirements from legacy source remain preserved. Five v2 sections — model_requirements, prompt_dialect, rag_configuration, context_strategy and api_harness — MUST survive compile → descriptor → catalog → binding or produce a field-level unsupported diagnostic. A default value is not proof that a nondefault required request was honored.

Context inheritance selects none, explicitly named artifacts/messages, or bounded authorized history. Full-history authoring is a request subject to policy and budget; it cannot copy live approval tokens, system credentials or private reasoning into a child. The installed receipt records actual context selection and exclusions.

## TeamDefinition

A top-level team pins member agent/subteam definitions, role cardinalities, one coordinator role, communication paths, task acceptance/assignment policy, model routing requirements and aggregate ceilings. Each subteam gets a separately identified TeamInstance linked to its parent task and root budget. Definition references MUST form a DAG. Activation MUST enforce the root graph's total members and maximum nesting, not reset limits at each child.

The first profile uses fixed coordinator membership; it may reactivate the coordinator's durable instance after recovery. It does not promise live leader election or cross-host takeover. Removing an active member requires an explicit drain or cancel decision; a UI deletion cannot orphan owned tasks.

## WorkflowDefinition

Workflows describe finite task dependencies, role requirements, outputs, joins and effect/retry classifications. No arbitrary executable expression language is embedded. A dependency is satisfied by the specified terminal success/artifact contract, not a delivery receipt. Failed required dependencies block successors; an explicit revised task graph with recorded revision can change that decision. Integration tasks in development workflows MUST depend on all planned implementation tasks.

Feedback-to-issue workflow creates a proposal first. The connector effect requires approval bound to exact destination and payload unless separately granted standing policy is explicitly supported; this profile's required acceptance story exercises human approval. A critic output is evidence for a human decision, never the approval itself.

## DeploymentBinding and package

Bindings resolve owner, workspace, UAR instance, definition locks, skill hashes, provider/model connection identities, storage capabilities and current policy revisions. Credential values stay in protected host stores; binding references are private installed state. Export MUST remove private identifiers as well as secrets when the receiving principal cannot resolve them, producing a template requiring fresh installation. It MUST NOT export executable approval or person-representation grants.

Package manifests bind example/source paths to exact byte hashes. Definition self-digests follow the canonicalization rule in schemas/README.md; they are distinct from a file's byte hash. Resolution MUST detect modified content under a reused identity/version. Unknown required extensions fail admission; optional unknown extensions are retained with an explicit unsupported disposition.

## Catalog and migration

Reuse UAR catalog revision/compare-and-swap semantics. Store canonical descriptor, source provenance and field conversion report together. Legacy artifact projections MAY remain for compatible consumers but MUST NOT become the sole authority when they drop semantics. A catalog replacement creates a revision; existing active bindings remain pinned until migration is explicitly applied. Atomic task ownership and inbox persistence belong to the runtime store, not the definition catalog.

Private RepresentationGrant is reserved for future authority-plane profiles. It must distinguish a role assistant, a simulation and disclosed representation of a named human. Neither a role title nor a portable document may authorize impersonation, spending, publishing or simulated human approval.
