## ADDED Requirements

### Requirement: End-to-end product workflow guides

The developer portal SHALL publish source-grounded guides at
`/docs/providers/configuration`, `/docs/providers/models`,
`/docs/agents/overview`, `/docs/skills/overview`,
`/docs/knowledge/overview`, and `/docs/memory/overview`. Together the guides
SHALL take a reader from provider and model configuration through inference,
agent execution, skill activation, knowledge retrieval, and memory behavior
using the supported packaged UI, API, and embedded boundaries that apply to
each workflow.

#### Scenario: Operator follows the packaged workflow

- **WHEN** an operator starts from provider configuration and follows the product guides
- **THEN** the portal identifies the supported configuration, model, agent, skill, knowledge, memory, and inference steps and the packaged surface used for each step

#### Scenario: Required workflow route is absent

- **WHEN** a required product guide or navigation entry is removed
- **THEN** local documentation validation exits non-zero and identifies the missing route or guide

### Requirement: Provider, model, and inference evidence boundaries

The provider and model guides SHALL distinguish provider-catalog discovery from
configured execution support, explain default and explicit `provider/model`
selection, identify credential and local-provider boundaries, and state profile
limits. A guide MUST NOT present a mock, stub, fixture, recorded response,
example response, catalog entry, or configuration save as evidence that genuine
model inference completed.

#### Scenario: Reader configures a provider and model

- **WHEN** a reader follows the provider and model setup workflow
- **THEN** the portal distinguishes saving configuration, resolving a model route, provider availability, and completing genuine inference as separate observable outcomes

#### Scenario: Inference is represented as verified

- **WHEN** a guide labels an inference path as observed or verified
- **THEN** it identifies the provider and model, the packaged UAR boundary traversed, and the returned genuine model output without transferring that evidence to another provider, model, route, or runtime profile

#### Scenario: Synthetic response is shown

- **WHEN** a guide uses an example, fixture, recorded response, or model double
- **THEN** it labels that material as illustrative or non-certifying and does not describe it as inference evidence

### Requirement: Agent creation and execution guidance

The portal SHALL document how to create, configure, select, run, and inspect a
basic agent through the supported server API and operator interface, and SHALL
describe the separate host-supplied boundary for `embedded-mobile`. The guide
SHALL connect agent configuration to effective model, skill, knowledge,
session, and runtime-event behavior without implying that the agent kernel owns
host mutation authority.

#### Scenario: Operator creates and runs an agent

- **WHEN** an operator follows the agent guide using the packaged server interface
- **THEN** the portal identifies how to create the agent, select its effective model and supported context, submit a run or chat request, and inspect the resulting output and runtime state

#### Scenario: Embedded reader follows the agent guide

- **WHEN** a reader selects the `embedded-mobile` path
- **THEN** the portal describes host-injected inference and persistence boundaries and does not direct the reader to server-only HTTP, admin, governance, or transport behavior as though it were embedded behavior

### Requirement: Skill lifecycle, scope, and reconciliation safety

The skill guide SHALL distinguish built-in, configuration-provisioned, and
API-created skill provenance; explain global, agent, and conversation scope
precedence; describe live activation and restart persistence; and state the
configuration-reconciliation tombstone and restore contract. It SHALL state
that reconciliation never hard-deletes a skill and MUST NOT tombstone built-in
or API-created skills.

#### Scenario: Reader evaluates effective skill activation

- **WHEN** a reader configures overlapping global, agent, and conversation skill states
- **THEN** the guide identifies conversation scope as most specific, then explicit durable agent scope, then global scope, and explains when the effective change becomes visible to execution

#### Scenario: Configuration-provisioned skill is removed

- **WHEN** a configured skill disappears from operator configuration
- **THEN** the guide describes tombstoning, operator-directed restoration, and preservation of its scoped configuration rather than hard deletion

#### Scenario: Built-in or API-created skill is reconciled

- **WHEN** reconciliation examines a built-in or API-created skill that is absent from operator configuration
- **THEN** the guide states that the skill is not tombstoned by configuration reconciliation

### Requirement: Knowledge and memory boundaries

The portal SHALL document knowledge-base creation, document ingestion and
processing, retrieval, attachment to agent execution, citation or retrieved
context visibility, and genuine model-backed use. It SHALL separately document
opt-in agent memory, auto-capture and context injection where supported, and the
memory MCP boundary. The guides MUST distinguish durable knowledge and memory
records from selected model context, live events, and process-local buffers.

#### Scenario: Reader verifies knowledge-backed inference

- **WHEN** a reader follows the knowledge workflow from an ingested document to an agent response
- **THEN** the guide identifies the durable knowledge resource, completed processing, retrieval evidence, how retrieved context enters execution, and the genuine model output that uses that knowledge

#### Scenario: Reader compares knowledge and memory

- **WHEN** a reader compares a knowledge base with agent memory
- **THEN** the portal explains their separate configuration, storage, retrieval, and lifecycle boundaries and does not use either term as a synonym for model context or live event history

### Requirement: Product workflow provenance, profiles, and realtime limits

Every product workflow guide SHALL identify its classified source record and
current public authority, state whether it applies to `server-full`, `minimal`,
or `embedded-mobile`, and distinguish reloadable resource state from realtime
event delivery. Local documentation controls SHALL fail when provenance,
profile limits, packaged UI/API coverage, inference-evidence language,
skill-safety distinctions, or knowledge-versus-memory boundaries are absent.

#### Scenario: Live update is described

- **WHEN** a guide describes provider, agent, skill, knowledge, or memory state updating live
- **THEN** it identifies the current realtime surface and the reloadable resource or configured persistence authority without describing event delivery itself as durability

#### Scenario: Product claim lacks provenance or profile scope

- **WHEN** a required source record, current authority, or runtime-profile limit is removed from a guide
- **THEN** local documentation validation exits non-zero and identifies the missing contract element

#### Scenario: Safety distinction is removed

- **WHEN** a guide no longer distinguishes genuine inference evidence, skill reconciliation safety, or knowledge from memory
- **THEN** an isolated negative control observes the documentation validator reject that guide before the complete source fixture passes
