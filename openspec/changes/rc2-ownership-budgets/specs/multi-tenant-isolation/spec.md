## ADDED Requirements

### Requirement: Compiler records are scoped by verified owner
Compiler specs, compile reports and authoring sessions SHALL record the storage key of the verified credential principal that created them. Every list, read, compile, update, delete and cancel SHALL be filtered by that owner on both the SurrealDB and PostgreSQL stores. A record owned by another principal SHALL answer exactly as an absent record does (404).

#### Scenario: A second owner cannot reach the first owner's compiler records
- **WHEN** the local owner and a proxied owner each create a compiler spec and an authoring session on the same installation
- **THEN** each owner lists and reads only its own spec and session, and gets 404 when it reads, compiles, deletes or cancels the other owner's records, on both route aliases and on both stores.

### Requirement: Compiler requests need a verified owner
A compiler spec, report or session request without a verified credential principal SHALL be refused with 401 and code `verified_compiler_owner_required`. Records stored before owner scoping SHALL stay unchanged in storage and SHALL be visible only to the installation-local `local-host` principal.

#### Scenario: No verified owner creates nothing
- **WHEN** `security.local_loopback_owner` is off and a loopback caller without credentials creates a compiler spec or an authoring session
- **THEN** the request is refused with 401.
