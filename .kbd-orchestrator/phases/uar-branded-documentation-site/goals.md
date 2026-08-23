# Goals

- Rewrite every README.md and all documentation surfaces so they accurately cover the complete supported Universal Agent Runtime functionality with consistent navigation and no stale claims.
- Create a branded Docusaurus documentation site whose visual identity, design tokens, interaction language, and component styling match the shipped React 19 application.
- Explain the runtime theory, why UAR exists, its architecture, profiles, protocols, APIs, SDKs, tools, skills, knowledge, tenancy, security, operations, and supported deployment boundaries.
- Review the complete .prometheus Karpathy logs and append-only history, then turn the observed architecture and design decisions into traceable documentation without inventing rationale.
- Document the testing methodology history, including failures, negative controls, evidence limits, local-only verification, removal of synthetic soak claims, and adoption of real-model functional integration.
- Add a deployment-only GitHub Actions workflow that builds and publishes the Docusaurus site to GitHub Pages without running unit, integration, lint, conformance, or other routine development tests.
- Validate the deployed GitHub Pages site and place its public URL in the repository README/documentation navigation and the GitHub repository homepage field.
- Audit all documentation against current main, versions.toml, OpenSpec, and KBD history so obsolete or contradictory material is corrected rather than copied forward.
