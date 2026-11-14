## ADDED Requirements

### Requirement: Final A2UI 1.0 surfaces and actions

UAR SHALL accept and publish final A2UI 1.0 messages for approved catalogs and the supported surface lifecycle. UAR SHALL retain legacy v0.9 and v0.9.1 history and preserve existing action authority and receipt safeguards.

#### Scenario: An inline final 1.0 surface accepts a governed action

- **WHEN** a validated createSurface carries approved components and initial data
- **THEN** UAR makes the data available before binding the components
- **AND** an action from the current declared component follows the existing trusted-host admission path
- **AND** a retained identical interaction reconciles the original receipt

#### Scenario: An unsupported function or release-candidate envelope is submitted

- **WHEN** a caller submits a remote function or the v1.0 release-candidate dialect
- **THEN** UAR refuses it before dispatch and advertises neither capability
