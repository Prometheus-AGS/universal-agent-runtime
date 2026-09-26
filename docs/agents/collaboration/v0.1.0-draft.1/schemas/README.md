# Proposed document schemas

Status: **official Draft Specification 0.1.0-draft.1**, design approved; runtime conformance is not claimed. These JSON Schema 2020-12 files are documentation artifacts, outside runtime generation/import paths. Their HTTPS `$id` values are stable identifiers for offline registration, not a claim that a schema hosting service is deployed. Resolve the complete local schema set without network retrieval.

| Schema | Instance |
|---|---|
| agent-definition.schema.json | Portable AgentDefinition |
| team-definition.schema.json | Portable TeamDefinition with immutable agent/subteam references |
| workflow-definition.schema.json | Portable bounded workflow DAG |
| package-manifest.schema.json | Portable package inventory and exact dependency lock |
| deployment-binding.schema.json | Private installed binding; never a portable agent package |
| runtime-snapshot.schema.json | Authorized snapshot projection, not a portable definition |
| custom-event.schema.json | Full AG-UI CUSTOM wrapper for negotiated team extension events |
| surface-action.schema.json | Proposed UAR action envelope, distinct from standard A2UI messages |
| trace-row.schema.json | Illustrative fixture wrapper; never a wire message |
| common.schema.json | Shared definitions and immutable references |

Portable profile version is `0.1.0-draft.1`. Event/action/snapshot envelope version is `1`; the event name namespace is `uar.team.v1`. These version domains must not be conflated with definition semantic versions or mutable record revisions. Protocol shapes are further constrained by [the protocol documents](../protocols/ag-ui.md).

## Digests and immutable resolution

For every definition, binding and package, compute `contentDigest = sha256(RFC8785(document with its top-level contentDigest omitted))`, encoded as lowercase hex with a `sha256:` prefix. The omission applies only to the top-level self digest, never nested reference digests. Canonical documents reject duplicate keys, nonfinite numbers and invalid Unicode; strings are not silently normalized. Example documents use ASCII keys and integer numeric values, making their canonical form unambiguous.

`PackageManifest.files[].byteDigest` hashes the exact checked-in UTF-8 file bytes, including formatting and final newline. `definition.digest` hashes canonical content as above. A manifest does not list itself, so there is no recursive digest. Manifest `contentDigest` covers its inventory and lock. The supplied examples have computed digests; they are not fabricated provider or skill artifact hashes.

The lock MUST close every member, permitted child and allowed-workflow reference, resolve each ID/version/digest consistently, and preserve one immutable content object per resolved triple. A package consumer MUST verify both byte digest and canonical content digest before registration. Do not resolve a mutable latest tag at activation. These structural schemas do not themselves compute digests.

## Required semantic checks

A valid JSON shape alone never authorizes activation. The importer/registry must also perform these checks before advertising runtime conformance:

1. Validate input/output contracts against the JSON Schema 2020-12 metaschema. Unbounded evaluation, arbitrary executable expressions and external fetches are not enabled by a contract.
2. Every required capability and every extension marked `required:true` must be understood and supported end to end. Unknown required semantics prevent activation with field-specific diagnostics. Unknown optional extensions are preserved verbatim and reported as unsupported; they may be ignored for execution, never silently deleted during export. Namespace ownership must be declared by an installed adapter, not inferred from its spelling.
3. Every team role is unique; cardinality min <= max; coordinatorRole exists and resolves to an agent member. Communication edges refer only to declared roles. Workflow roles resolve in the team invoking that workflow; workflow task IDs are unique, dependency references exist, and the dependency graph is acyclic. Input mapping is a constrained reference syntax, not an expression evaluator.
4. Definition and nested-team graphs are acyclic and within ancestor/global ceilings. Compute member cardinality over the entire root team graph, including each subteam instance and its members. Child limits and budgets never enlarge an ancestor ceiling; definition requests grant no resource authority. The draft defaults are four turns/team, eight globally, sixteen members/root graph, depth three, and one thousand queued tasks/team. The explicit schema ceilings reflect this draft profile; increases need a profile/configuration policy decision and cannot be achieved by changing a portable document alone.
5. Workflow non-idempotent writes require an exact-effect operator approval at effect time. `approval:none` never exempts an actual tool boundary from current authorization. An approval-request step's completion is only an intent, never approval evidence. Retries preserve effect identity; unknown outcomes enter reconciliation. `maxAttempts` counts admitted execution attempts, not message delivery duplicates.
6. Runtime snapshot revisions/cursors are monotonic within their scope. Task/attempt/member/parent references resolve; task assignment epoch agrees with the current attempt; one member has at most one active mutating turn; dependencies govern readiness. JSON Schema cannot prove these relational or temporal constraints. Terminal tasks do not reopen; an explicit follow-up is a new task, while a nonterminal task can acquire another attempt after reconciliation.
7. Secret values, executable approvals, authority tokens and private representation grants MUST NOT appear anywhere in portable documents, including legacySections, sourceDescriptor, skill config, free text, contracts or optional extensions. Inspect content at import/export; arbitrary JSON extensions make a syntactic key blacklist insufficient. Installed bindings contain private references only. Serialized snapshots and public events are separately authorized/redacted projections, never full persistence dumps.
8. Imported legacy definitions retain `sourceDescriptor`, the full original compiler IR, unchanged. In particular retain `model_requirements`, `prompt_dialect`, `rag_configuration`, `context_strategy`, `api_harness`, and every skill's ID/version/required/config. `legacySections` preserves original authoring sections if distinct from the compiler IR. Do not manufacture support by copying fields: conversion diagnostics distinguish exact, translated, optional-unsupported and required-unsupported, and required behavior must actually be available before activation.

## Message receipts and action authority

`accepted` means a durable inbox commit; `delivered` means available to the intended recipient; `processed` means the consuming turn acknowledged application; `rejected` is a delivery/processing refusal. None means the assigned task succeeded. Task completion is a separate terminal task transition. A duplicate message ID returns its existing receipt without a second activation. Recipient sequences are independent of team event cursors.

Actions carry command identity, surface identity and expected revision. Optional team/task/member fields are untrusted routing hints. The server derives the authoritative team/task/member/run/catalog mapping from the surface registry and authenticates the actor before dispatch. The action payload is not an approval token. `surface.message` includes the exact `uar.a2ui/1` profile and `urn:uar:a2ui:catalog:1`; validate its inner message against that pinned profile, not upstream experimental 1.0.

## Private executive representation extension

An executive-role definition may include the OPTIONAL extension `uar.executive.role` describing an office, advisory function, requested capability and declaration that it cannot impersonate or approve for a human. It contains no named-human grant. A private authority-plane RepresentationGrant is stored separately and minimally relates grant ID, represented principal, grantee instance, organization, permitted action/resource scope, consent evidence reference, issuer, validity interval, revocation revision and restrictions. No grant example carries actual identity, signature, token, consent evidence or secret. Installed bindings may refer to such grants through `representationGrantRefs`; fresh effect authorization resolves current validity and revocation. This draft defines the boundary, not first-release implementation or a portable grant format.

The uncomfortable limitation: well-formed schemas cannot establish transaction durability, authorization correctness, sensible routing, or successful recovery. Those are future integration conformance obligations.
