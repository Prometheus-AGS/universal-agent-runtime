# Spec Delta

## ADDED Requirements

### Requirement: API-key authority survives restart and revocation is current

Production API-key authentication and management SHALL persist hashed key material, authority metadata, expiry and revocation, and enforce current authority across processes and restarts.

#### Scenario: Cross-process revocation

- **WHEN** one runtime revokes a key and another receives a new request using it
- **THEN** the second denies admission using durable current authority

#### Scenario: Durable storage outage

- **WHEN** production key storage cannot establish current validity
- **THEN** authentication and management fail explicitly without an in-memory or stale-authority fallback


