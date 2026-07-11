## MODIFIED Requirements

### Requirement: Runtime Console Panels Are Wired To Live Backend Emission

Every Runtime Console panel that represents a defined runtime entity type SHALL
be wired to a live backend emission path, so it renders real operational data
rather than a "not yet wired" disclosure. This covers `RuntimeProviderHealth`,
`RuntimeMemoryEvent`, `RuntimeArtifact`, `RuntimeAgUiEvent`,
`RuntimeModelRouteDecision`, and `RuntimeA2uiSurface`, in addition to the
already-wired `RuntimeRun`, `RuntimeRunStep`, `RuntimeToolCall`, and
`RuntimeApproval` types. A panel MAY show an empty state only when its backing
emission path exists but no matching activity has occurred; a permanent
"backend has no emission path" disclosure is no longer acceptable for these
types.

#### Scenario: A wired panel receives live emission

- **Given** a Runtime Console panel for a defined runtime entity type
  (provider health, memory event, artifact, AG-UI event, model route decision,
  or A2UI surface)
- **When** the backend emits the corresponding `runtime.*` frame (or the
  frontend routes the corresponding `agui.*` frame into the entity graph)
- **Then** the panel MUST render the resulting entity live, without a manual
  refresh, and MUST NOT show a "not yet wired" disclosure

#### Scenario: A wired panel is simply quiet

- **Given** a wired Runtime Console panel whose emission path exists
- **When** no matching runtime activity has occurred
- **Then** the panel MUST show a neutral empty state (e.g. "No … observed
  yet"), distinct from a "not yet wired to backend data" disclosure

#### Scenario: Execution timeline rows carry human-readable detail

- **Given** the Execution Timeline / Run Detail panels render `RuntimeRunStep`
  entities
- **When** the backend emits `runtime.step` frames
- **Then** each frame MUST carry the step's `title`, `kind`, and `summary` so
  rows render non-blank
