# Spec Delta

## ADDED Requirements

### Requirement: Effective settings require one Bearer admin credential

The pinned effective-settings read SHALL use the existing JWT verifier and an explicit admin principal claim. A missing or invalid Bearer credential SHALL return 401; a valid non-admin principal SHALL return 403. A valid admin Bearer credential alone SHALL be sufficient for the read, without `x-uar-admin-key` or another credential header. Installable local and shared profiles SHALL require JWT for private routes by default.

#### Scenario: Missing or invalid token

- **WHEN** the effective-settings path receives no Bearer token or an invalid token
- **THEN** it returns 401 and no effective settings body

#### Scenario: Non-admin token

- **WHEN** a valid authenticated non-admin principal requests effective settings
- **THEN** it returns 403 without exposing the aggregate

#### Scenario: Bearer admin alone

- **WHEN** a valid admin principal requests effective settings with only `Authorization: Bearer <token>`
- **THEN** it returns the selected secret-free aggregate without requiring a second credential header
