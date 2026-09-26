# Change: UAR collaboration package catalog

## Why

UAR can compile and register only one legacy agent at a time. The approved collaboration profile requires lossless multi-agent and team definitions, atomic installation, exact immutable references, and private deployment binding before durable team execution can begin.

## What changes

- Compile and validate canonical AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, and DeploymentBinding documents.
- Add atomic package preflight/install with command idempotency, catalog revisions, conversion diagnostics, and exact retrieval.
- Preserve the full source descriptor and emit a compatibility AgentArtifact projection for ordinary single-agent use.
- Add private owner/workspace-scoped deployment binding preflight and revisioned persistence.
- Expose REST and MCP administration and advertise package support without claiming team execution.

## Non-goals

- TeamInstance execution, task scheduling, messaging, or AG-UI/A2A team endpoints.
- The Boss sidecar, settings, or release changes.

## Impact

Additive UAR compiler/catalog/API work. Existing Markdown compilation, `/api/agents`, and ordinary agent runs remain supported.
