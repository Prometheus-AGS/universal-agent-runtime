## ADDED Requirements

### Requirement: Kreuzberg-Pinned Advisory Mitigation

When a `cargo audit` advisory is confirmed reachable through the pinned `kreuzberg` dependency and no upstream release fixes it, the system SHALL bound the blast radius via explicit resource limits rather than leaving the vulnerable code path unconstrained, and the disposition MUST be recorded in `docs/DEPENDENCY_MANAGEMENT.md`.

#### Scenario: Reachable advisory with no upstream fix

- **Given** `cargo audit` reports `RUSTSEC-2026-0187` (lopdf stack overflow)
  and `RUSTSEC-2026-0194` (quick-xml quadratic attribute DoS) as reachable
  via kreuzberg's document-processing path
- **When** no kreuzberg tag through the latest available release fixes both
  advisories
- **Then** `KreuzbergConfig` MUST expose `max_input_bytes` and
  `extraction_timeout_secs` limits wired into the document-processing entry
  points, and the disposition MUST be documented with a reachability trace

#### Scenario: Advisory confirmed not reachable

- **Given** `cargo audit` reports an advisory for a dependency pulled in by
  kreuzberg
- **When** source inspection finds no call site exercising the vulnerable
  API
- **Then** the advisory MUST be disclosed as not reachable in
  `docs/DEPENDENCY_MANAGEMENT.md` rather than silently left unaddressed
