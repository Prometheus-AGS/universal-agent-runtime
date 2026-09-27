# Compatibility and fixed legacy Agent Markdown headings

Canonical draft.2 collaboration documents are JSON. UAR-AGENT-MD is accepted only as a legacy AgentDefinition input adapter. Import retains the original bytes, complete parsed IR, source identity, original heading spelling, authored presence, and any explicit rename mapping.

## Exact v1.1 headings

The only accepted top-level heading is:

1. `# Agent: <name>`

The fixed fifteen v1.1 section headings are:

1. `## Metadata`
2. `## Identity`
3. `## UI (A2UI)`
4. `## Capabilities`
5. `## Skills`
6. `## Tools`
7. `## MCP Servers`
8. `## Knowledge Base`
9. `## Memory Model`
10. `## A2A Contracts`
11. `## Governance`
12. `## Budgets & Constraints`
13. `## Execution Model`
14. `## Observability`
15. `## Deployment Profiles`

## Exact v2 additions

Agent Spec v2 adds exactly these five headings:

1. `## Model Requirements`
2. `## Prompt Dialect`
3. `## RAG Configuration`
4. `## Context Strategy`
5. `## API Harness`

Omission and an explicitly authored default are distinct. `authoredFields` records that distinction. An accepted compatibility alias may normalize to one of the headings above, but the source spelling remains in the source record and conversion report.

## No collaboration Markdown dialect

The importer MUST NOT interpret `# Team:`, `# Workflow:`, `# Deployment Binding:`, or `# Representation Grant:` as canonical documents. Those values are JSON documents selected by the draft.2 discriminator and their exact `kind`.

## Field preservation and binding

The canonical SkillRef superset is `id`, `version`, `digest`, `required`, `config`, `entrypoint`, and `requiredTools`. The v2 fields normalize to `modelRequirements`, `promptDialect`, `ragConfiguration`, `contextStrategy`, and `apiHarness`, each with `{ required, value }`.

Conversion storage is not execution support. Every field receives a JSON-pointer diagnostic. `required-unsupported` blocks private binding and activation. `optional-unsupported` preserves the exact value for export. A successful EffectiveBindingReceipt identifies the enforcing runtime component and immutable resolutions.

## Draft.1 migration

Draft.1 input remains readable. Migration records its original profile, ID, version, digest, optional revision, normalized draft.2 identity, and field diagnostics. It does not modify draft.1 bytes or publish the normalized value under the old digest.

## Native and downgrade export

An export target declares its profile or harness and supported semantics. Export refuses any required field the target cannot represent. Optional unsupported content remains in the portable report when the target format can carry it. A native agent file never becomes a UAR TeamInstance, task, approval, or authority grant.
