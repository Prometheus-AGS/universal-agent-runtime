## Why

UAR's primary product workflows are currently split across configuration, API,
skills, and troubleshooting pages, leaving operators without one source-grounded
path from provider setup to genuine inference, agent execution, skill activation,
knowledge retrieval, and memory behavior. The architecture spine is now stable,
so these shipped workflows need current public guides before README reconciliation
and final portal certification can be truthful.

## What Changes

- Publish end-to-end provider and model guides that distinguish catalog metadata,
  configured execution support, default routing, credentials, local providers,
  and profile-specific inference boundaries.
- Document genuine inference through the packaged API and operator interface,
  including observable success, failure, and realtime-state behavior without
  presenting examples, mocks, fixtures, or recorded responses as model evidence.
- Publish agent creation and execution guidance across the supported API,
  operator interface, and embedded boundary, linking execution semantics back to
  the current architecture authority.
- Replace fragmented skill guidance with current built-in, configured, and
  API-created provenance; global, agent, and conversation activation precedence;
  scoped governance; live effect; restart behavior; and tombstone/restore safety.
- Publish knowledge ingestion, document processing, retrieval, citation/context
  use, and memory guides that distinguish durable product state, retrieved model
  context, opt-in agent memory, and the memory MCP boundary.
- Add bounded local documentation controls for required routes, classified
  provenance, profile limits, packaged UI/API coverage, genuine-inference
  language, skill-safety distinctions, and knowledge-versus-memory boundaries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Require source-grounded product workflow guides for
  providers, models, genuine inference, agents, skills, knowledge, and memory,
  with packaged UI/API paths, realtime behavior, provenance, and explicit
  runtime-profile and evidence limits.

## Impact

- **Documentation:** new or reconciled guides under `website/docs/providers/`,
  `website/docs/agents/`, `website/docs/skills/`, `website/docs/knowledge/`, and
  `website/docs/memory/`; existing fragmented pages; route/category metadata;
  publication manifests; and bounded local source validators.
- **Runtime UX:** no React behavior changes; the guides map the shipped provider,
  model, agent, skill, knowledge, memory, and chat surfaces into complete operator
  workflows and describe which state is expected to update live.
- **Provider compatibility:** no provider, model, credential, or routing behavior
  changes; documentation must distinguish catalog discovery from configured and
  genuinely observed execution support and state profile-specific limits.
- **Realtime state:** no SSE, entity-graph, or normalized-event changes; guides
  document current live updates and identify reloadable resource authority rather
  than treating event delivery as durability.
- **Dependencies and APIs:** no dependency, public API, storage schema, runtime
  profile, or deployment workflow changes.
- **KBD:** transition this registered change through Execute only after bounded
  source evidence passes, then advance the exact next command to
  `document-security-tenancy-governance-and-operations`.
