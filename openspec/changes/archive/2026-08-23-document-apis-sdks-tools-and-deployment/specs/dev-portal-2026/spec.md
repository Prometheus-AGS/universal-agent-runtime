## ADDED Requirements

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
