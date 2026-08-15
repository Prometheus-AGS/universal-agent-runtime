# skill-builtin-availability Specification

## Purpose
TBD - created by archiving change skill-builtins-on-embedded. Update Purpose after archive.
## Requirements
### Requirement: Built-in skills are available on every runtime profile
Built-in skills SHALL be discovered and registered on the embedded runtime as
well as the server runtime, so that a host embedding the runtime library obtains
the same built-in catalogue as a host running the server.

#### Scenario: Fresh embedded database gains built-ins
- **WHEN** the embedded runtime starts against a database that has never been written
- **THEN** the built-in skills are present in the registry and readable through the embedded skill service

#### Scenario: Built-ins are durable, not in-memory
- **WHEN** the embedded runtime registers built-ins and the process exits
- **THEN** a second embedded start against the same database finds those built-ins already persisted

#### Scenario: Re-registration does not duplicate
- **WHEN** the embedded runtime starts twice against the same database
- **THEN** each built-in skill appears exactly once in the registry

#### Scenario: Seeding switch is honoured
- **WHEN** the embedded runtime is built with default seeding disabled
- **THEN** built-in skills are not seeded, consistent with built-in agents and the default knowledge base

### Requirement: Built-in availability is asserted rather than assumed
The runtime SHALL carry a test that fails when built-in skills are absent from a
freshly-initialized embedded runtime.

#### Scenario: Negative control for the availability test
- **WHEN** the availability test runs against a build with the embedded registration removed
- **THEN** the test fails, demonstrating it is capable of detecting the defect it exists to catch
