## ADDED Requirements

### Requirement: Private Vulnerability Reporting Policy
The repository SHALL publish a security policy naming a private reporting
channel, acknowledgement/triage time targets, and supported versions.

#### Scenario: Researcher finds the policy
- **WHEN** a security researcher opens SECURITY.md
- **THEN** it names GitHub private vulnerability reporting as the channel,
  states acknowledgement and triage targets, and lists supported versions

### Requirement: Support Expectations Are Published
The repository SHALL publish support channels and response expectations.

#### Scenario: Customer evaluates support
- **WHEN** a prospective deployer opens SUPPORT.md
- **THEN** it states where to ask questions, what response to expect, and how
  commercial support differs

### Requirement: Licensing Clarity For Self-Hosters
The repository SHALL publish a plain-language explanation of AGPL obligations
for self-hosting customers and when the commercial license applies.

#### Scenario: Enterprise legal review
- **WHEN** an evaluator opens docs/LICENSING.md from the README
- **THEN** it states that unmodified self-hosting imposes no source
  publication duty, what network copyleft does require, and the commercial
  licensing contact path
