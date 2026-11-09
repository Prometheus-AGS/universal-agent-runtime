## ADDED Requirements

### Requirement: Constrain native component loading
Component compiler/runtime/target/features SHALL agree for claimedAOT. Discovered untrustednativecache SHALL never be unsafe-deserialized; only protected provenance or safelycompiledplaincomponent policy may produce executable code, preserving governedtool boundaries.

#### Scenario: A user directory contains malformed or incompatible native cache
- **WHEN** A user directory contains malformed or incompatible native cache
- **THEN** Loading refuses actionably or safely uses declared plaincomponent source, never executing arbitrary nativebytes

#### Scenario: Reference component fixture identity remains exact
- **WHEN** the plain reference component is used for isolated capability behavior coverage
- **THEN** its SHA-256 byte digest encoded with two lowercase hexadecimal characters per byte equals the unchanged recorded fixture digest, while the runtime registration/execution and complete isolated-capability output assertions remain required
