## MODIFIED Requirements

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

### Requirement: Docusaurus information architecture
The project SHALL organize the developer portal according to the authoritative route manifest and SHALL expose generated API references, current guides, decision history, and testing methodology through consistent primary and contextual navigation.

#### Scenario: Architecture section
- **WHEN** a visitor opens the architecture category
- **THEN** they can reach the runtime theory, execution boundaries, profiles, persistence, protocols, and current decision authority

#### Scenario: SDK sections
- **WHEN** a visitor opens the Rust, Python, or TypeScript SDK section
- **THEN** they see the supported SDK behavior, version and profile boundaries, runnable guidance, and only those generated references actually staged by the portal build

#### Scenario: Contributing section
- **WHEN** a visitor opens the contributing section
- **THEN** they see contribution conventions, local documentation verification, license boundaries, and the process for changing public claims

#### Scenario: History sections
- **WHEN** a visitor opens architecture or testing history
- **THEN** they see dated, source-linked synthesis that distinguishes current authority from superseded methods and designs

### Requirement: GitHub Pages deployment workflow
The project SHALL use exactly one GitHub Actions workflow to build, package, deploy, and validate the complete documentation artifact on accepted changes to `main`. The workflow SHALL perform deployment execution and deployed-artifact validation only; routine development verification SHALL run locally before publication.

#### Scenario: Docs deployment
- **WHEN** an accepted documentation change reaches `main`
- **THEN** the sole Pages workflow builds the pinned Docusaurus site, stages generated references, uploads one Pages artifact, deploys it, and checks the deployed root and representative deep routes

#### Scenario: API reference wiring
- **WHEN** the Pages artifact is assembled
- **THEN** generated Rust and TypeScript references are staged under their declared portal paths
- **AND** Python reference navigation is published only when a corresponding generated artifact is staged

#### Scenario: Competing publisher
- **WHEN** another workflow attempts to upload or deploy the GitHub Pages artifact
- **THEN** local workflow-policy validation fails before the change is accepted

#### Scenario: Routine verification in Actions
- **WHEN** the Pages workflow contains prose linting, unit tests, integration tests, conformance tests, or local accessibility checks
- **THEN** local workflow-policy validation fails and identifies the prohibited step
