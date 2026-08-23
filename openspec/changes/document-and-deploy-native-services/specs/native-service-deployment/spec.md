## ADDED Requirements

### Requirement: macOS installed release is functionally verified
After code completion, the phase SHALL build and install the release binary and React bundle, load the LaunchAgent on port 1906, and observe health, readiness, UI/static assets, loopback-only listeners, provider/model visibility, genuine inference, persistence across one restart, database access, graceful shutdown, and required logging.

#### Scenario: Installed LaunchAgent is restarted
- **WHEN** the operator restarts the LaunchAgent after successful inference
- **THEN** it becomes ready again with configuration, provider visibility, database access, operational logging, and genuine inference intact
