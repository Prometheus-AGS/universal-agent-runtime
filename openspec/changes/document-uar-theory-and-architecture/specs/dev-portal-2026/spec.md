## ADDED Requirements

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
