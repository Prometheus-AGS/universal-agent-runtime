## ADDED Requirements

### Requirement: Session prompt caching is an authoritative tri-state control
Session Configuration SHALL expose prompt caching through a shadcn-style Inherit, On, and Off control backed by the canonical and draft session entities. The editor SHALL display the effective inherited value and source returned by the runtime and SHALL persist the selected override through the typed session configuration contract.

#### Scenario: The session inherits prompt caching
- **WHEN** the saved session override is null
- **THEN** the control selects Inherit and displays the authoritative effective On or Off value and source

#### Scenario: The operator changes the session override
- **WHEN** the operator selects On, Off, or Inherit and saves
- **THEN** the draft submits true, false, or null respectively through the session domain action
- **AND** reopening the editor shows the persisted selection

#### Scenario: Effective state cannot be loaded
- **WHEN** the effective prompt-caching endpoint fails
- **THEN** the editor reports an actionable unavailable state and does not invent an inherited value

### Requirement: Session configuration accepts absent legacy agent configuration
The Session Configuration frontend SHALL treat both an empty 204 response and the legacy 404 response as absence, initialize a complete draft for the owner, and preserve non-absence errors as actionable failures.

#### Scenario: The backend returns 204
- **WHEN** Session Configuration loads before the first save
- **THEN** a complete new draft is initialized without displaying a not-found error

#### Scenario: A legacy backend returns 404
- **WHEN** Session Configuration loads against a compatible older runtime
- **THEN** the response is treated as absent configuration

#### Scenario: Another load error occurs
- **WHEN** the backend returns a status other than the accepted absence statuses
- **THEN** the error remains visible and the frontend does not silently initialize authoritative state
