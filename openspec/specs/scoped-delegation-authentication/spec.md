# scoped-delegation-authentication Specification

## Purpose
Preserve authenticated credential ownership when an installation host delegates scoped execution, while ensuring delegated callers do not acquire installation-level authority.

## Requirements

### Requirement: Delegation retains authenticated identity without host privilege
Scoped grants SHALL retain the complete credential principal captured at issuance and SHALL remove installation/service host authority. Grant identity SHALL never be inferred from a caller-supplied subject or owner string.

#### Scenario: Local host registers a delegated context
- **WHEN** an authenticated local host issues a workspace-scoped grant and registers its matching delegated host context
- **THEN** caller and grant resolve to the same complete verified credential principal
- **AND** delegated requests have no host authority
- **AND** existing scope, expiry, epoch, definition and binding checks remain mandatory
