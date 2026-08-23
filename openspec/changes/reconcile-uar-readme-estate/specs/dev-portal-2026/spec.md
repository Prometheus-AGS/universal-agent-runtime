## MODIFIED Requirements

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

#### Scenario: Frozen product routes
- **WHEN** the route manifest is validated
- **THEN** chat, A2UI artifact, Runtime Console, runs, approvals, protocols, providers, credentials, models, skills, agents, tools, authentication, knowledge, memory, compiler, settings, A2UI testing, MCP health, cost, and About document IDs MUST all exist

#### Scenario: Compatibility page scope
- **WHEN** a product route reuses a broader current guide
- **THEN** its page MUST remain a concise profile-bounded entry point and link to the deeper authority rather than duplicate it
