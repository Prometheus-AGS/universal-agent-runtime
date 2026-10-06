## ADDED Requirements

### Requirement: Owner-scoped inert history snapshot

The API SHALL provide GET /api/uar/runs/{id}/events?after=<exclusive cursor> only to the same current run owner admitted by get_run_for_context. It SHALL read existing history without subscribing, cancelling or changing state and SHALL return404 for foreign/missing history.

#### Scenario: Capture native call arguments
- **WHEN** the run owner reads retained history
- **THEN** public tool call/result projections retain their SSE event semantics and exact call IDs/arguments/results

### Requirement: Versioned bounded cursor contract

The snapshot SHALL use version1, ordered monotonic event IDs and cursor, declared process-local-bounded retention and explicit retention-gap/cursor-ahead indication. It SHALL NOT promise durable replay after process shutdown.

#### Scenario: History prefix has expired
- **WHEN** retained IDs after the requested cursor omit an expected ID
- **THEN** gapReason is retention-gap and the response does not claim a complete replay
