## ADDED Requirements

### Requirement: Scheduled Audit Trigger Independence

Security-audit CI checks SHALL run on their own dedicated trigger (schedule and/or manual dispatch) rather than being nested only inside an unrelated pipeline whose trigger condition may rarely or never fire, and a scheduled audit MUST ignore only advisories with a documented disposition so it fails on genuinely new findings.

#### Scenario: A security-relevant check is nested inside a rarely-firing trigger

- **Given** a repository's documentation claims a security audit step runs
  as part of another workflow (e.g. a release pipeline)
- **When** that other workflow's own trigger condition (e.g. a version-tag
  push) has never actually fired
- **Then** a dedicated workflow with its own schedule and/or
  `workflow_dispatch` trigger MUST be added, independent of the other
  workflow's trigger, and the documentation MUST be corrected to describe
  the actual trigger

#### Scenario: A scheduled audit ignores advisories

- **Given** a scheduled audit job ignores one or more advisory IDs to avoid
  permanent failure on already-triaged findings
- **When** the ignore list is defined
- **Then** each ignored ID MUST correspond to a disposition already
  documented (fixed-but-still-listed, mitigated, or accepted-risk) in the
  project's dependency-management documentation, so an advisory outside
  that list still fails the job
