## MODIFIED Requirements

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
