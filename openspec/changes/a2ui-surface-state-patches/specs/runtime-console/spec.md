## MODIFIED Requirements

### Requirement: Validated A2UI Surface Replay
The runtime console SHALL reconstruct late-join A2UI surface state from ordered replay patches using the existing A2UI validator and reducer.

The replay log is a message and action log. The console SHALL obtain newer surface changes by re-reading `GET /api/uar/runs/{run_id}/a2ui/surface-replay`. It SHALL NOT merge live `STATE_DELTA` patches into its replayed surfaces, because their encoding is run state rather than messages.

Action entries (`add /a2ui/actions/<uuid>`) SHALL be skipped from surface reconstruction.

#### Scenario: A2UI replay succeeds
- **WHEN** `GET /api/uar/runs/{run_id}/a2ui/surface-replay` returns valid ordered state-patch operations
- **THEN** the console MUST reconstruct their A2UI v0.9.1 message envelopes in publish order
- **AND** it MUST pass every message through the existing A2UI validation and reduction path before exposing replayed surface metadata.

#### Scenario: Replay contains action entries
- **WHEN** the replay log contains `add /a2ui/actions/<uuid>` entries between valid surface entries
- **THEN** the console MUST skip the action entries without treating them as invalid
- **AND** it MUST reconstruct the surface entries as if the action entries were absent.

#### Scenario: Replay contains an invalid patch
- **WHEN** a replay item has an unknown operation, invalid surface path, invalid component graph, or executable content
- **THEN** the console MUST reject that item from rendered surface state
- **AND** it MUST expose an inspector error that identifies the rejected replay position without executing the payload.

#### Scenario: Replay and live events overlap
- **WHEN** a late-joining console has loaded replay history and newer surface changes are then published for the run
- **THEN** the console MUST obtain them by re-reading the replay log rather than by merging live state patches
- **AND** replay operations MUST retain publish order
- **AND** stable surface and event identities MUST prevent duplicate visible records.
