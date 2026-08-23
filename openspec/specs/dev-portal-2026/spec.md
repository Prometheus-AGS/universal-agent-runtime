# dev-portal-2026 Specification

## Purpose

Provide a single, hosted developer portal that combines narrative documentation, API references generated from source, and architecture decision records for the Universal Agent Runtime project.

## Requirements

### Requirement: Hosted developer portal
The project SHALL publish one branded Docusaurus site to GitHub Pages that combines complete narrative documentation, architecture and testing history, architecture decision records, and generated API references that are actually produced and staged by the accepted publication path.

#### Scenario: Docusaurus IA
- **WHEN** a visitor navigates to the documentation site
- **THEN** the information architecture exposes the runtime theory, architecture, configuration, profiles, providers and models, agents, skills, knowledge and memory, APIs and SDKs, protocols, tenancy and governance, security, operations, testing history, decisions, deployment, and contributing sections required by the route manifest

#### Scenario: API reference hosting
- **WHEN** the documentation site is built
- **THEN** every advertised generated API reference is staged beneath the portal and linked from canonical navigation
- **AND** a language whose reference artifact is not actually produced is described without a false hosted-reference claim

### Requirement: Prose quality
The project SHALL provide a deterministic local command that runs the UAR-specific prose rules and fails when the required validator is unavailable or reports a violation. The deployment workflow SHALL consume locally accepted documentation and SHALL NOT own prose linting.

#### Scenario: Style violation
- **WHEN** a contributor runs the documented local prose command against a style violation
- **THEN** the command exits non-zero and identifies the violation

#### Scenario: Prose validator is unavailable
- **WHEN** the local prose command cannot locate its required validator
- **THEN** the command exits non-zero instead of treating the check as passed

### Requirement: Architecture decisions are documented

The project SHALL maintain architecture decision records (ADRs) that document the major grade-A decisions.

#### Scenario: ADR lookup

- **WHEN** a developer opens `docs/adr/`
- **THEN** they find a template, an index, and at least 10 ADRs covering the grade-A decisions

### Requirement: Docusaurus information architecture
The project SHALL organize the developer portal around the frozen product route
inventory plus current architecture, workflow, security, operations,
configuration, SDK, deployment, history, and contributing authorities. Every
required product route SHALL resolve to one Docusaurus document ID without
changing the frozen route contract in a content lane.

#### Scenario: Architecture section
- **WHEN** a visitor opens the architecture section
- **THEN** they see the UAR purpose, trust boundaries, execution lifecycle, state/events, profiles, protocols, and delegation limits

#### Scenario: SDK sections
- **WHEN** a visitor opens a Rust, Python, or TypeScript SDK guide
- **THEN** they see source-supported SDK behavior and separate local-reference, hosted-reference, and registry-publication status

#### Scenario: Contributing section
- **WHEN** a visitor opens the contributing section
- **THEN** they see contribution guidance, the license split, commit conventions, and local verification policy

#### Scenario: History sections
- **WHEN** a visitor opens architecture or testing history
- **THEN** they see dated, source-linked synthesis that distinguishes current authority from superseded methods and designs

#### Scenario: Frozen product routes
- **WHEN** the route manifest is validated
- **THEN** chat, A2UI artifact, Runtime Console, runs, approvals, protocols, providers, credentials, models, skills, agents, tools, authentication, knowledge, memory, compiler, settings, A2UI testing, MCP health, cost, and About document IDs MUST all exist

#### Scenario: Compatibility page scope
- **WHEN** a product route reuses a broader current guide
- **THEN** its page MUST remain a concise profile-bounded entry point and link to the deeper authority rather than duplicate it

### Requirement: Vale prose linting

The project SHALL run a UAR-specific prose linter against the documentation.

#### Scenario: Lint command

- **WHEN** a contributor runs `pnpm docs:lint`
- **THEN** Vale executes using `.vale.ini` and the UAR style rules

### Requirement: Architecture decision records

The project SHALL publish ADRs that capture the grade-A decisions.

#### Scenario: ADR template

- **WHEN** a contributor proposes a new architectural decision
- **THEN** they use the template in `docs/adr/0001-record-architecture-decisions.md`

#### Scenario: Grade-A decisions documented

- **WHEN** a reviewer inspects `docs/adr/`
- **THEN** they find at least 10 ADRs covering license, coverage, error handling, configuration, supply chain, SDKs, RAG, A2UI vendoring, A2UI renderer, and docs/visual regression

### Requirement: GitHub Pages deployment workflow
The project SHALL use exactly one GitHub Actions workflow to build, package, deploy, and validate the complete documentation artifact on accepted changes to `main`. The workflow SHALL perform deployment execution and deployed-artifact validation only; routine development verification SHALL run locally before publication. The accepted artifact SHALL be assembled from the frozen npm-managed Docusaurus build and real generated reference outputs, and SHALL fail rather than publish a placeholder when a declared reference cannot be generated or staged.

#### Scenario: Docs deployment
- **WHEN** an accepted documentation change reaches `main`
- **THEN** the sole Pages workflow installs the pinned npm dependencies, builds the Docusaurus site, stages generated references, uploads one Pages artifact, deploys it, and checks the deployed root and representative deep routes

#### Scenario: API reference wiring
- **WHEN** the Pages artifact is assembled
- **THEN** generated Rust and TypeScript references are staged under their declared portal paths
- **AND** Python reference navigation is published only when a corresponding generated artifact is staged
- **AND** a missing declared generated reference stops publication instead of creating placeholder content

#### Scenario: Competing publisher
- **WHEN** another workflow attempts to upload or deploy the GitHub Pages artifact
- **THEN** local workflow-policy validation fails before the change is accepted

#### Scenario: Routine verification in Actions
- **WHEN** the Pages workflow contains prose linting, unit tests, integration tests, conformance tests, or local accessibility checks
- **THEN** local workflow-policy validation fails and identifies the prohibited step

#### Scenario: Package-manager mismatch
- **WHEN** the frozen Docusaurus build is installed and invoked through npm
- **THEN** every site build subcommand also uses the npm-managed contract and does not require pnpm, yarn, or bun

#### Scenario: Deployed route validation
- **WHEN** the Pages deployment reports its public URL
- **THEN** deployment validation requests the portal root plus representative narrative, Rust-reference, and TypeScript-reference routes
- **AND** any missing or non-successful route fails the deployment job

### Requirement: Portal presents the shipped UAR identity
The documentation portal SHALL use the same UAR mark, wordmark, ember/cyan palette, surface hierarchy, typography roles, and Flat 2.0 interaction language as the shipped React application. It SHALL contain no stock Docusaurus identity, tutorial copy, sample illustration, or unrelated social-card asset.

#### Scenario: Reader opens the portal
- **WHEN** a reader opens the homepage or a documentation route
- **THEN** the UAR identity is visible in navigation and page presentation
- **AND** the page contains no stock Docusaurus product identity

#### Scenario: Flat 2.0 regions render
- **WHEN** navigation, hero, cards, sidebars, code blocks, or callouts distinguish adjacent regions
- **THEN** they use filled surface steps and spacing rather than decorative borders, separator lines, gradients, or shadows

### Requirement: Homepage orients readers to the product
The portal homepage SHALL explain the runtime's purpose, agent/host trust boundary, supported protocol and product surfaces, profile limits, and direct next steps into concepts, guides, reference, and operations.

#### Scenario: New reader chooses a path
- **WHEN** a reader reaches the homepage without prior UAR knowledge
- **THEN** they can identify what UAR does, what it does not claim across profiles, and which primary documentation path matches their goal

### Requirement: Portal interaction remains accessible across presentation modes
The portal SHALL preserve semantic navigation, visible keyboard focus, zoom and reflow, touch targets, heading hierarchy, light/dark/system themes, readable code and Mermaid output, and reduced-motion behavior at the responsive sizes certified by the final local gate.

#### Scenario: Keyboard navigation
- **WHEN** a reader navigates interactive portal controls without a pointer
- **THEN** focus order follows document order and every focused control has a visible UAR-token focus indicator

#### Scenario: Reduced motion preference
- **WHEN** the reader enables reduced motion
- **THEN** nonessential animation is removed or reduced without hiding content or state

#### Scenario: Narrow viewport
- **WHEN** the portal is viewed at a supported mobile width or browser zoom level
- **THEN** navigation, copy, code, tables, and calls to action remain reachable without page-level horizontal scrolling

### Requirement: Portal search is local and deterministic
The portal SHALL build a local search index from accepted public documentation and SHALL NOT require a hosted search or analytics service to discover content.

#### Scenario: Reader searches documentation
- **WHEN** a reader submits a query for indexed public content
- **THEN** matching documentation routes are returned from the locally built index
- **AND** private-synthesis-only or excluded sources are absent from results

#### Scenario: Search index cannot be produced
- **WHEN** the production documentation build cannot generate the configured local index
- **THEN** the build fails instead of publishing a portal with a falsely advertised search control

### Requirement: Source-grounded architecture narrative

The developer portal SHALL provide a public architecture section whose
present-tense claims are traceable to current repository source, canonical
OpenSpec requirements, or observed product behavior. The section SHALL explain
the problem UAR solves, its runtime theory, capability inversion, the agent and
trusted-host boundary, turn and execution lifecycle, normalized event flow,
persistence, delegation, and protocol boundaries without presenting planned or
historical behavior as delivered.

#### Scenario: Reader follows the conceptual spine

- **WHEN** a reader enters the architecture section
- **THEN** the portal provides a navigable path from UAR's purpose through its trust boundary, execution lifecycle, protocols, state, and delegation model

#### Scenario: Architecture claim lacks current authority

- **WHEN** a present-tense architecture claim cannot be traced to current source, canonical specification, or observed behavior
- **THEN** the portal omits the claim or labels it explicitly as historical, planned, uncertain, or unsupported

### Requirement: Capability inversion boundary

The architecture section SHALL state that agent kernels propose actions and do
not own mutation authority, while trusted host capabilities authorize, execute,
persist, and observe side effects. It SHALL distinguish model output and tool
intent from a completed, host-authorized effect.

#### Scenario: Reader evaluates an agent-requested side effect

- **WHEN** a reader examines how a model-selected tool can change external state
- **THEN** the portal shows the request crossing identity, policy, capability, execution, and event boundaries before the effect is represented as completed

#### Scenario: Agent-only state is compared with durable state

- **WHEN** a reader compares an agent's conversational context with runtime-owned state
- **THEN** the portal identifies runtime events and configured persistence as inspectable authorities and does not describe agent-only memory as durable business state

### Requirement: Profile-bounded architecture claims

The architecture section SHALL document `server-full`, `minimal`, and
`embedded-mobile` as separate capability and evidence profiles. Every claim that
does not apply uniformly SHALL state its profile, and evidence for one profile
MUST NOT be presented as evidence for another.

#### Scenario: Governance capability is described

- **WHEN** the portal describes governance or server composition
- **THEN** it identifies `server-full` as the governing profile and states any exclusions or reduced behavior for `minimal` and `embedded-mobile`

#### Scenario: Embedded behavior is described

- **WHEN** the portal describes embedded or offline execution
- **THEN** it states the embedded-mobile boundary and does not transfer server-full persistence, protocol, or operational claims to it

### Requirement: Stable architecture routes and diagrams

The portal SHALL publish the architecture conceptual spine at stable documented
routes, include accessible text surrounding every Mermaid diagram, and expose
those routes through architecture navigation. Local documentation validation
SHALL fail when a required route, profile limit, provenance reference, or
diagram explanation is missing.

#### Scenario: Architecture navigation is rendered

- **WHEN** a reader opens the Architecture category
- **THEN** the category exposes the purpose, trust-boundary, execution-lifecycle, state-and-events, profiles, protocols, and delegation guides

#### Scenario: Diagram is unavailable

- **WHEN** Mermaid does not render or a reader does not consume graphics
- **THEN** the surrounding prose still communicates the diagram's nodes, direction, authority boundaries, and profile limits

#### Scenario: Required architecture evidence is removed

- **WHEN** a required architecture route, profile statement, provenance source, or text explanation is removed
- **THEN** the local documentation controls exit non-zero and identify the missing contract element

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

### Requirement: Security and operations guide set

The developer portal SHALL publish source-grounded guides for authentication,
credentials, tenant isolation, governance, approvals, the runtime console, run
operations, observability, realtime state, cost controls, and recovery and
shutdown. The guides SHALL link the shipped operator surfaces to their API or
process boundaries and SHALL identify unsupported or process-local behavior.

#### Scenario: Operator follows the operating contract
- **WHEN** an operator follows the security and operations guides
- **THEN** the portal identifies how to configure and observe each delivered control, its fail-closed or permissive behavior, and the profile and durability limits that apply

#### Scenario: Required operating guide is absent
- **WHEN** a required guide or compatibility route is removed
- **THEN** local documentation validation exits non-zero and identifies the missing document or route

### Requirement: Authentication and credential boundaries

The authentication guide SHALL distinguish required and optional JWT behavior,
HS256 and RS256/JWKS verification, issuer/audience/not-before validation,
RustCrypto process-provider ownership, API-key creation and exchange, and
unauthenticated probe exceptions. The credential guide SHALL distinguish API
keys from provider credentials, document write-only plaintext and masked reads,
state the credential resolution order, and explain the encryption-key and
single-tenant fallback boundaries without publishing usable secrets.

#### Scenario: Authentication verification fails
- **WHEN** a required token is absent, has an invalid signature or registered claim, uses an unsupported JWKS algorithm, has an unknown key after refresh, or cannot be verified because the JWKS endpoint is unavailable with no cached key
- **THEN** the guide states the request is rejected and does not describe an unverified claim as an identity or tenant boundary

#### Scenario: Provider credential service is unavailable
- **WHEN** the credential encryption key is absent or invalid
- **THEN** the guide distinguishes disabled per-user credential storage from the operator environment/configuration fallback and does not claim encrypted multi-tenant storage is active

### Requirement: Tenant, governance, and approval limits

The portal SHALL document tenant identity as a value created only from a
verified credential and SHALL enumerate the current subsystems that consume it
without claiming universal tenant partitioning. It SHALL distinguish Cedar
allow/deny decisions from risk-based human approval, document profile-specific
governance behavior and policy-load fallback, and state that denial, rejection,
channel closure, cancellation, and approval timeout do not authorize tool
execution.

#### Scenario: Tenant claim is unverified
- **WHEN** a request supplies a tenant-like string outside successful credential verification
- **THEN** the guide states that UAR does not construct a trusted tenant identity from that value

#### Scenario: Governance is unavailable or permissive
- **WHEN** a profile omits Cedar or the server falls back to a permissive policy set after a policy-load error
- **THEN** the guide states that Cedar denial is not provided by that condition and does not present the runtime as fail closed by default

#### Scenario: Approval is not completed
- **WHEN** a human rejects a tool call, its approval channel closes, its run is cancelled, or five minutes elapse without approval
- **THEN** the guide states that the tool call is rejected and that a Cedar denial cannot be overridden by approval

### Requirement: Observability, realtime, and cost evidence limits

The operations guides SHALL document health/readiness probes, Prometheus
metrics, structured logs, optional OTLP export, runtime-console entity state,
run inspection and cancellation, multiplexed realtime updates, reconnect
behavior, usage-derived cost estimates, and budget alerts. They MUST distinguish
provider/model observations from UAR-owned signals, live or process-local state
from durable history, and estimated cost from billing authority.

#### Scenario: Operator inspects live state
- **WHEN** the runtime console receives run, approval, provider, route, cost, or protocol updates
- **THEN** the guide identifies the SSE/entity-graph path and states which state must be reloaded or is lost when the process or browser session ends

#### Scenario: Operator interprets cost
- **WHEN** a run reports usage and an estimated cost or crosses an in-process budget threshold
- **THEN** the guide labels the amount as an estimate, identifies the process/session scope, and directs billing reconciliation to the provider rather than treating the dashboard as an invoice

### Requirement: Shutdown, persistence, and recovery boundaries

The portal SHALL explain SIGINT/SIGTERM handling, run cancellation, listener
drain, registered cleanup, the configured hard deadline, emitted shutdown
outcomes, persistence-provider ownership, cold backup/restore, and embedded or
offline host responsibility. It SHALL NOT claim that an HTTP cancellation token
alone terminates every runtime resource or that a successful source check proves
backup restoration.

#### Scenario: Graceful deadline expires
- **WHEN** an HTTP/SSE request or registered cleanup remains held until the configured shutdown deadline
- **THEN** the guide states that the process exits at the deadline without the graceful-complete outcome rather than waiting indefinitely

#### Scenario: Operator restores persistent state
- **WHEN** an operator follows a local or remote database restore procedure
- **THEN** the guide requires a stopped/cold boundary where applicable, identifies the database owner and configured location, and requires functional read-back instead of treating archive creation as restore proof

### Requirement: Security and operations provenance gate

Every security and operations guide SHALL identify classified source records,
current authorities, applicable runtime profiles, state ownership, and an
explicit limitation. Local controls SHALL reject missing boundaries, unsafe
credential material, universal tenant or governance claims, missing
realtime-versus-durable distinctions, and missing shutdown or recovery limits
before accepting the complete source fixture.

#### Scenario: Security claim exceeds current authority
- **WHEN** a guide claims blanket tenant isolation, universal fail-closed Cedar enforcement, durable live history, authoritative billing, or cross-profile parity not supported by its authorities
- **THEN** local documentation validation exits non-zero and identifies the unsupported claim class

#### Scenario: Public guide contains unsafe material
- **WHEN** a guide contains raw credentials, private keys, machine-local paths, raw private-history excerpts, or internal event/session payloads
- **THEN** local documentation validation exits non-zero before the complete fixture passes

### Requirement: API and protocol reference boundaries

The portal SHALL document the current UAR HTTP, SSE, OpenAI-compatible,
Anthropic-compatible, AG-UI, A2UI, MCP, and A2A entrances and outputs from
current source. It SHALL distinguish adapter compatibility from complete
upstream protocol parity and identify the runtime profiles in which each
network surface is present.

#### Scenario: Developer chooses an interface
- **WHEN** a developer compares UAR protocol surfaces
- **THEN** the portal identifies the route or transport, request and event boundary, authentication expectation, applicable profile, generated-reference location, and known compatibility limit

#### Scenario: Reference exceeds current source
- **WHEN** a guide names an endpoint, event mode, protocol version, or generated reference that is not present in a classified current authority
- **THEN** local documentation validation exits non-zero and identifies the unsupported claim

### Requirement: Tool discovery and execution boundaries

The portal SHALL document native tools, MCP-discovered tools, the public tool
catalog and execution routes, tool-name normalization, capability and approval
boundaries, health visibility, and the local-only JWT proxy. It MUST NOT
describe a discovered tool as automatically authorized or the development proxy
as a production authentication gateway.

#### Scenario: Developer executes a tool
- **WHEN** a developer follows the tool guide
- **THEN** the portal explains discovery, normalized identity, schema and capability checks, governance or approval outcomes, execution events, and the profile limits that apply

#### Scenario: Tool boundary is overstated
- **WHEN** a guide treats discovery as authorization, bypasses the trusted host execution boundary, or recommends the JWT proxy for production
- **THEN** local documentation validation exits non-zero before accepting the complete source fixture

### Requirement: SDK source and publication status

The portal SHALL document the Rust, Python, and TypeScript SDK source packages,
their supported client or embedded modes, examples, local build or reference
commands, and their actual hosted-reference and registry-publication status.
The presence of package metadata or source version `1.0.0` MUST NOT be
represented as proof that a registry artifact is available.

#### Scenario: Developer selects an SDK
- **WHEN** a developer opens an SDK guide
- **THEN** the portal identifies the repository package, supported boundary, local installation or source command, examples, generated-reference availability, and profile or transport limits

#### Scenario: Python hosted reference is claimed without staging
- **WHEN** the portal claims a hosted generated Python reference but the Pages artifact contract does not stage one
- **THEN** local documentation validation exits non-zero and identifies the unsupported publication claim

### Requirement: Configuration authority and safe operation

The portal SHALL document configuration discovery and precedence, the
structured environment naming convention, settings that are startup-only or
runtime-managed, secret handling, provider/model selection, persistence and
feature requirements, schema inspection, and reload limits from current source.
Examples MUST use placeholders and MUST NOT imply that an unsafe local setting
provides an authenticated or tenant-isolated deployment.

#### Scenario: Operator resolves a setting
- **WHEN** a setting is present in more than one supported source
- **THEN** the portal identifies which value wins, whether a restart or supported reload is required, and which process or profile consumes it

#### Scenario: Unsafe configuration is presented as production-ready
- **WHEN** a guide exposes usable secret material, disables authentication on a non-local listener, or omits a required persistence or feature boundary
- **THEN** local documentation validation exits non-zero and identifies the unsafe example or missing limit

### Requirement: Installation, deployment, and upgrade contract

The portal SHALL document source builds, pinned container deployment, Docker
Compose, Kubernetes/Helm, persistence ownership, health and readiness checks,
profile and platform support, release artifacts, backup prerequisites, upgrade,
rollback, and the deployment-only GitHub Pages workflow boundary. It SHALL NOT
transfer evidence between profiles or claim that a tag, package manifest, image
name, or successful documentation build proves an artifact was published or a
runtime deployment is healthy.

#### Scenario: Operator deploys a supported server profile
- **WHEN** an operator follows an installation or deployment path
- **THEN** the portal names the artifact or source boundary, immutable pinning method, required secrets and storage, exposed ports, applicable profile, and functional health/readiness checks

#### Scenario: Publication or profile claim lacks evidence
- **WHEN** a guide treats local metadata as registry availability, transfers server-full behavior to minimal or embedded-mobile, or places routine development verification in GitHub Actions
- **THEN** local documentation validation exits non-zero and identifies the unsupported claim or policy violation

### Requirement: Architecture decision history routes

The portal SHALL publish an architecture-history overview, complete retained ADR
index, dated timeline, correction ledger, and process-provenance guide. Current
product guides SHALL remain authoritative for present behavior.

#### Scenario: Reader enters the history section

- **WHEN** a reader opens the History category
- **THEN** the reader can navigate the five architecture-history guides and distinguish current, superseded, and historical records

#### Scenario: Accepted decision is mistaken for delivered behavior

- **WHEN** an ADR or plan is accepted but delivery is not established by current source or specification
- **THEN** the portal describes the record as intent or history rather than a delivered product claim

### Requirement: Public testing methodology

The portal SHALL publish testing history, an evidence taxonomy, negative-control
practice, and local verification timing. It SHALL preserve the move from
coverage/synthetic emphasis to bounded genuine-model functional acceptance
without claiming earlier methods were useless or later evidence was broader than
its recorded server-full scope.

#### Scenario: Reader chooses a verification method

- **WHEN** a reader needs evidence for a code, inference, resilience, or deployment claim
- **THEN** the portal identifies the narrowest applicable evidence class, timing boundary, and non-claims

#### Scenario: Routine test is placed in Actions

- **WHEN** documentation proposes unit, integration, conformance, lint, format, type, or routine docs verification in GitHub Actions
- **THEN** local policy validation exits non-zero because Actions are deployment-only

### Requirement: Canonical public portal metadata

The sole Pages deployment workflow SHALL publish and validate the complete
Docusaurus artifact. After the canonical URL is observed working, the repository
homepage and root README SHALL point to that URL.

#### Scenario: Reader enters from the repository

- **WHEN** a reader uses the repository homepage field or README documentation link
- **THEN** the link opens the observed branded portal and representative deep links remain reachable

#### Scenario: Actions workflow performs routine testing

- **WHEN** the Pages workflow contains unit, integration, conformance, lint, format, type, or other routine development checks
- **THEN** the local GitHub Actions policy gate fails before publication
