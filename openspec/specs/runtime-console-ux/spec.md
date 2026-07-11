# runtime-console-ux Specification

## Purpose
TBD - created by archiving change resolve-runtime-protocols-page-facade. Update Purpose after archive.
## Requirements
### Requirement: Runtime Console Panels Are Wired To Live Backend Emission

Every Runtime Console panel that represents a defined runtime entity type SHALL
be wired to a live backend source, so it renders real operational data rather
than a "not yet wired" disclosure. This covers `RuntimeProviderHealth`,
`RuntimeMemoryEvent`, `RuntimeArtifact`, `RuntimeAgUiEvent`,
`RuntimeModelRouteDecision`, and `RuntimeA2uiSurface`, in addition to the
already-wired `RuntimeRun`, `RuntimeRunStep`, `RuntimeToolCall`, and
`RuntimeApproval` types. A panel MAY show a neutral empty state only when its
source exists but no matching activity has occurred; a permanent "backend has
no emission path" disclosure is no longer acceptable for these types.

#### Scenario: A wired panel receives live data

- **Given** a Runtime Console panel for a defined runtime entity type
- **When** its backing source produces data — the backend emits the
  corresponding `runtime.*` frame, the frontend routes the corresponding
  `agui.*` frame into the entity graph, or the console's REST feed polls the
  backing endpoint (provider health, resolve-model, a2ui/schemas)
- **Then** the panel MUST render the resulting entity live, without a manual
  refresh, and MUST NOT show a "not yet wired" disclosure

#### Scenario: A wired panel is simply quiet

- **Given** a wired Runtime Console panel whose source exists
- **When** no matching runtime activity has occurred
- **Then** the panel MUST show a neutral empty state (e.g. "No … observed
  yet"), distinct from a "not yet wired to backend data" disclosure

### Requirement: Operators Can Inspect a Specific Run's Detail

The Runtime Console SHALL let an operator select any listed run and view that specific run's detail, rather than always displaying a fixed run regardless of selection.

#### Scenario: Inspecting a run from the Runs page list

- **Given** the Runs page lists multiple runs
- **When** the operator clicks "Inspect" on a run row
- **Then** the Run Detail column MUST show that run's steps, not always the first run in the list

#### Scenario: Inspecting a run from the Cockpit's Live Runs panel

- **Given** the operator is on the Cockpit page, which has no run-detail column
- **When** the operator clicks "Inspect" on a run row
- **Then** the system MUST navigate to the Runs page with that run preselected

