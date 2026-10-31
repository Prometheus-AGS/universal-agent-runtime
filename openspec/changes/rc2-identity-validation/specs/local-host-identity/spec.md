## ADDED Requirements

### Requirement: A loopback caller without credentials is the local host
When `security.local_loopback_owner` is enabled on the TrustedLocal deployment profile with optional JWT, UAR SHALL give a request that has no credentials and a loopback TCP peer address the `local-host` principal. Its sessions, memory, knowledge bases and tools SHALL therefore share one owner. A non-loopback request without credentials SHALL keep the existing anonymous behavior. No request header SHALL influence the peer-address decision.

#### Scenario: Local chat and memory share one owner
- **WHEN** `security.local_loopback_owner` is true and a loopback caller without credentials saves a memory record and chats in a session
- **THEN** the same caller can read that memory record and lists that session
- **AND** an owner supplied by the identity proxy sees neither.

#### Scenario: Off by default
- **WHEN** `security.local_loopback_owner` is not set and a loopback caller without credentials saves a memory record
- **THEN** the request is refused as anonymous, as before.
