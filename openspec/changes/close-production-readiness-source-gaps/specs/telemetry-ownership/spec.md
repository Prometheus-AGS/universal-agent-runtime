## ADDED Requirements

### Requirement: Respect telemetry host ownership
Telemetry SHALL coexist without replacing or panicking on hostglobals, expose honest export/scrape status, keep labels bounded and contentprivate, preserve correlation without duplicate accounting, and report bounded supportedflush/offline guarantees.

#### Scenario: A host recorder or subscriber already exists
- **WHEN** A host recorder or subscriber already exists
- **THEN** UAR reports hostownership instead of a healthy empty scrape or fictitious ownexport; runexecution remains usable
