## ADDED Requirements

### Requirement: Explicit negotiation with legacy compatibility
The host SHALL accept optional client rendering support and a requested auto, text, A2UI or hybrid mode. Absence of both fields SHALL preserve current client behavior. Support-only requests SHALL use auto; mode-only requests SHALL have no asserted rendering support. A2UI SHALL request surface-first output with a brief accessible text summary; hybrid SHALL request substantive text and surface output. Explicit text or unsupported UI output SHALL prevent surface publication and preserve a readable text path with a recorded reason.

#### Scenario: Legacy client
- **WHEN** a client omits rendering negotiation
- **THEN** its existing A2UI and text behavior remains available under existing governance

#### Scenario: Unsupported surface mode
- **WHEN** a client requests A2UI or hybrid without compatible rendering support
- **THEN** the effective mode is text and a fallback reason is recorded

### Requirement: Host-owned frozen selection and truthful provenance
The trusted host SHALL freeze validated template content together with eligible identities/revisions at admission and govern rendering against that snapshot. Edits, disablement and deletion SHALL affect subsequent admissions, not mutate admitted snapshots. Agents SHALL NOT mutate templates or select another owner or run through render-tool arguments. Selection provenance SHALL distinguish requested mode, effective mode and actual template publication; it SHALL NOT claim client display without evidence.

#### Scenario: Template changes during a run
- **WHEN** a selected template is edited after the run snapshot is assembled
- **THEN** that run renders only the frozen revision and records its identity/revision

#### Scenario: Text ceiling applies to legacy tools
- **WHEN** a negotiated text-only run attempts a2ui_render
- **THEN** the host prevents surface publication rather than bypassing the output ceiling

#### Scenario: Non-tool publication paths
- **WHEN** an explicit text-only run reaches host policy artifact emission, direct surface-message submission or delegated output
- **THEN** the same host output ceiling prevents surface publication on every path

#### Scenario: Template disabled or deleted after admission
- **WHEN** a template is disabled or deleted after a run captures its validated content and revision
- **THEN** that admitted run retains the captured content while new admissions cannot select the template

#### Scenario: Selection without publication
- **WHEN** auto mode exposes eligible templates but no surface is published
- **THEN** run events do not claim a rendered template or client display
