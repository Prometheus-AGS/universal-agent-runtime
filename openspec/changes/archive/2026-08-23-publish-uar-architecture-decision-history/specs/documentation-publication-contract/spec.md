## ADDED Requirements

### Requirement: Reviewed historical synthesis

Public architecture history SHALL be synthesized from classified repository
records. Raw Prometheus logs, KBD event payloads, session transcripts,
machine-local paths, credentials, and unreviewed wiki copies MUST NOT be
published directly.

#### Scenario: Reader follows a decision to evidence

- **WHEN** a reader examines a selected architecture decision
- **THEN** the public record identifies its date, disposition, reviewed source class, and present authority without exposing raw private content

#### Scenario: Raw history enters public output

- **WHEN** a public history page copies a private record or cites an unreviewed wiki file as authority
- **THEN** local publication controls exit non-zero and identify the unsafe source

### Requirement: Supersession remains visible

The public history SHALL preserve material reversals and SHALL identify the
replacement for every selected superseded position. The record MUST include the
licensing, frontend, visual-authority, JWT-provider, verification-location,
inference-evidence, and placeholder-publication corrections.

#### Scenario: Old guidance conflicts with current authority

- **WHEN** a retained decision is no longer current
- **THEN** the history labels it superseded, names the replacement, and links the reader to current authority

#### Scenario: A correction is omitted

- **WHEN** one of the required correction records or its replacement is missing
- **THEN** local history validation exits non-zero
