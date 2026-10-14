# Spec Delta

## ADDED Requirements

### Requirement: Follow-up decisions and delivery remain governed and durable

Follow-ups SHALL derive only from authorized durable occurrences under current user-controlled rules. Decisions, suppressions and delivery state SHALL be durable; external dispatch uncertainty SHALL never be hidden by automatic retries.

#### Scenario: Policy suppresses follow-up

- **WHEN** an observed event matches a rule but consent, source grant or destination permission is revoked
- **THEN** no disclosure or effect occurs and an owner-visible scoped suppression reason is retained

#### Scenario: Delivery interrupted

- **WHEN** a committed follow-up is interrupted during delivery
- **THEN** the durable inbox item is recovered once logically, while uncertain external dispatch is held for reconciliation rather than retried blindly

#### Scenario: Overnight quiet hours

- **WHEN** a rule uses daily 22:00–07:00 quiet hours in its explicit IANA zone
- **THEN** 22:00 is suppressed and 07:00 is permitted, with overnight and repeated-hour membership evaluated at the candidate real instant and queued work rechecked after policy edits


