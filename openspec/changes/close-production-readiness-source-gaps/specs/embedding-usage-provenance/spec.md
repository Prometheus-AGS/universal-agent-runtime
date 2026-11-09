## ADDED Requirements

### Requirement: Preserve embedding accounting and remote batch identity
Embedding APIs SHALL preserve known provider counts independently and distinguish unavailable counts from measuredzero; remote index sets SHALL be complete and unique, and provider internals SHALL remain private.

#### Scenario: A pinned backend reports mixed usage across chunks
- **WHEN** A pinned backend reports mixed usage across chunks
- **THEN** Only fully reported counters are available; malformed index/results refuse without partial success or credential disclosure
