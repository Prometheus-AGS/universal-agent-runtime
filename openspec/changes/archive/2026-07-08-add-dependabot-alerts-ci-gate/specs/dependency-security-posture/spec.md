## ADDED Requirements

### Requirement: Dependabot Alert Feed Checked in CI

`security-audit.yml` SHALL include a job that checks GitHub's Dependabot
alert feed (`GET /repos/{owner}/{repo}/dependabot/alerts`) as a
complement to `cargo audit`/`npm audit`/`pnpm audit`, since those tools'
advisory databases have been shown to lag GitHub's own GHSA database.
The job SHALL fail when an **open** alert's GHSA ID is not already
present in the workflow's disclosed-advisory allowlist, and SHALL fail
with a specific, actionable error (not a silent pass or generic error)
when the credential used lacks sufficient permission to call the
endpoint.

#### Scenario: A genuinely new, undisclosed alert appears

- **Given** GitHub's Dependabot alert feed reports an **open** alert
  whose GHSA ID is not in the workflow's disclosed-advisory allowlist
- **When** the `dependabot-alerts-gate` job runs
- **Then** the job MUST fail, surfacing the GHSA ID, severity, and
  affected package so it can be triaged and disclosed in
  `docs/DEPENDENCY_MANAGEMENT.md`

#### Scenario: Only already-disclosed alerts are open

- **Given** every **open** Dependabot alert's GHSA ID is already present
  in the workflow's disclosed-advisory allowlist
- **When** the `dependabot-alerts-gate` job runs
- **Then** the job MUST pass

#### Scenario: The credential lacks permission to read alerts

- **Given** the token used by the job does not have sufficient scope
  (`security_events` for a classic PAT, or "Dependabot alerts: Read" for
  a fine-grained token) to call the Dependabot alerts endpoint
- **When** the API call returns 401 or 403
- **Then** the job MUST fail with a message identifying the likely cause
  (insufficient token scope) rather than passing silently or failing
  with an unexplained generic error
