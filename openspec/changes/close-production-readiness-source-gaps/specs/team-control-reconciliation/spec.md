## ADDED Requirements

### Requirement: Preserve team policy and uncertain liabilities
Team control advertisement SHALL reflect usable authoritativepolicy and typed recoverable versus terminal refusals. Reconciliation SHALL reuse authoritative owner/trustedhost usage/effect receipts, retain unknown liabilities and immutable settled outputs, and never replay effects or invent free usage.

#### Scenario: A cancelled attempt is reconciled repeatedly after restart
- **WHEN** A cancelled attempt is reconciled repeatedly after restart
- **THEN** Only identical authoritative evidence settles original IDs idempotently; unknown/conflicting/foreign evidence remains refused or reserved with explicit budget impact

#### Scenario: The installed Linux team recovery case observes a real restart
- **WHEN** The trusted host stops and starts the case-owned Linux amd64 runtime between the before and after observations
- **THEN** The case verifies a clean stop and a later start on the bound image, container and volume before comparing original team, attempt, accounting and effect identities
- **AND** Missing or mismatched fixture ownership fails before team admission, and the case exchanges only nonsecret selectors and lifecycle observations with the host
