## MODIFIED Requirements

### Requirement: Every run records a redacted turn manifest
The runtime SHALL record, for each turn and dispatch attempt, a manifest of fragment IDs, hashes, counts, budgets, provenance, selected skills/tools and warnings; store it in run context; and emit it as an additive artifact. It SHALL include actual destination, profile/template/counting revisions, override provenance, count certainty, reserves, compaction disposition and predicted/actual usage when available. It SHALL NOT include prompt bodies, credentials, hidden reasoning or raw retrieved content. Unavailable actual usage SHALL be labeled unknown, never zero.

#### Scenario: Manifest is emitted alongside the effective policy artifact
- **WHEN** a run starts
- **THEN** clients receive the existing effective-policy artifact and a turn_manifest artifact without fragment body text

#### Scenario: Failover changes preparation
- **WHEN** a request changes destination
- **THEN** a correlated attempt manifest records the new profile, template, budget and outcome while preserving redaction

## ADDED Requirements

### Requirement: Templates resolve for the exact destination
Every supported destination SHALL resolve a versioned profile keyed by provider, endpoint kind and exact model/revision or reviewed alias, with template hash and counting revision. Host policy SHALL constrain descriptor/operator overrides, then exact profiles, then explicitly verified compatible family profiles. Generic rendering SHALL be eligible only when required endpoint capabilities and limits are known. Unsupported compatibility SHALL produce an explicit outcome.

#### Scenario: Descriptor override reaches production
- **WHEN** a permitted descriptor override selects a compatible template for an exact model
- **THEN** the final outbound request uses that template and records which override won

#### Scenario: Ambiguous family match
- **WHEN** a model name merely contains a known family string but no verified compatible profile applies
- **THEN** that substring does not authorize family settings; preparation uses a validated generic contract or returns unsupported

### Requirement: Templates preserve authority and endpoint semantics
Templates SHALL declare required slots and supported roles, preserve fixed section order and protected historical ordering, and escape data without elevating retrieved instructions. Prompt layout, chat serialization/control tokens and provider settings SHALL be separate contracts. Only host-owned raw-inference formatting SHALL emit model chat control tokens; hosted APIs SHALL use their documented message schema. Templates SHALL be data-only and SHALL NOT execute model-supplied template programs or filesystem includes.

#### Scenario: Missing required slot
- **WHEN** a template omits policy, governed tools, required evidence or another required slot, or cannot represent a protected role
- **THEN** preparation fails explicitly before dispatch

#### Scenario: Different endpoint for the same family
- **WHEN** a hosted-chat endpoint and a raw-inference endpoint use related models
- **THEN** each captured request follows its own serialization contract without duplicated control tokens or transplanted API fields

### Requirement: Every attempt is prepared from canonical content
Initial calls, iterations, retries, failovers, graph steps and resumed runs SHALL derive from immutable canonical content and current authorization, resolving the actual destination before rendering, settings selection and final protected-context budgeting. Supported output/reasoning/structured-output/cache settings SHALL be backed by exact endpoint/model evidence and request fixtures. The enforced output limit SHALL match the budget reservation. An already rendered or reduced source request SHALL NOT be reused as canonical input.

#### Scenario: All entry paths use the contract
- **WHEN** each supported production entry path invokes a provider, including graph and resumed runs
- **THEN** a final driver-boundary fixture captures the complete request and verifies selected template/settings, required sections and final input/output bounds

#### Scenario: Failover after compaction
- **WHEN** the first destination used a prose summary and a retry selects another destination
- **THEN** the retry reauthorizes and prepares from canonical data under the new profile without compressing protected data or reexecuting completed tools

#### Scenario: Opaque continuity incompatibility
- **WHEN** signed or opaque provider continuity data cannot be represented by a fallback endpoint
- **THEN** transparent failover to that endpoint is ineligible; the runtime neither rewrites protected continuity nor synthesizes equivalent hidden reasoning

#### Scenario: Cache compatibility
- **WHEN** destination, template or rendered content changes
- **THEN** request cache identity changes accordingly, and revoked or incompatible cached context cannot be reused
