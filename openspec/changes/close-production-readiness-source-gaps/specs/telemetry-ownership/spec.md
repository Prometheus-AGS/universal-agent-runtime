## ADDED Requirements

### Requirement: Respect telemetry host ownership
Telemetry SHALL coexist without replacing or panicking on hostglobals, expose honest export/scrape status, keep labels bounded and contentprivate, preserve correlation without duplicate accounting, and report bounded supportedflush/offline guarantees.

#### Scenario: A host recorder or subscriber already exists
- **WHEN** A host recorder or subscriber already exists
- **THEN** UAR reports hostownership instead of a healthy empty scrape or fictitious ownexport; runexecution remains usable

#### Scenario: Distinct run requests share one bounded metric route
- **WHEN** the mounted runtime receives requests for distinct run IDs with query parameters
- **THEN** its Prometheus sample reports the quoted route template `/api/uar/runs/{id}`, counts both requests, and omits the raw IDs and query values
