## ADDED Requirements

### Requirement: A published documentation site exists

The project SHALL provide a documentation site, built from source and deployed
by CI to GitHub Pages, covering at minimum: introduction, installation,
configuration reference, a backup/restore runbook, an upgrade guide,
troubleshooting, and an API reference.

#### Scenario: Docs site builds in CI

- **When** the docs deploy workflow runs on a push touching `website/**`
- **Then** the Docusaurus build MUST succeed and the site MUST be published to
  GitHub Pages

#### Scenario: Operational runbooks are present

- **Given** a self-hosting operator
- **When** they open the docs site
- **Then** they MUST find a configuration reference (env vars + `UAR_*__*`
  convention), a backup/restore runbook for the embedded datastore, and a
  troubleshooting page covering the common boot failures
