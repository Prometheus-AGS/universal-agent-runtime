# provider-diagnostic-status Specification

## Purpose
TBD - created by archiving change moonshot-provider-status. Update Purpose after archive.
## Requirements
### Requirement: Provider Credential-Blocked Status

The provider catalog SHALL classify auth-required providers that are not configured as `credential-blocked`.

#### Scenario: Moonshot lacks a configured credential

- **Given** Moonshot is present in the model catalog with an auth environment variable
- **And** Moonshot is not present as an enabled configured provider
- **When** the providers catalog response is generated
- **Then** the Moonshot provider summary MUST include `status` set to `credential-blocked`
- **And** the provider summary MUST include a status detail explaining that configuration requires the declared credential source

#### Scenario: Provider is configured

- **Given** a provider is present as an enabled configured provider
- **When** the providers catalog response is generated
- **Then** the provider summary MUST include `status` set to `configured`

### Requirement: Provider Status UI

The providers UI SHALL display provider diagnostic status without exposing secrets.

#### Scenario: Credential-blocked provider is selected

- **Given** an operator opens the providers page
- **And** selects an auth-required provider that is not configured
- **When** the provider detail panel renders
- **Then** the UI MUST show that the provider is credential-blocked
- **And** the UI MUST NOT show any API key value

#### Scenario: Configured provider is selected

- **Given** an operator selects a configured provider
- **When** the provider detail panel renders
- **Then** the UI MUST continue to show the provider as configured

### Requirement: Secret-Safe Provider Verification

Provider compatibility hardening SHALL avoid committing provider credentials or live secret material.

#### Scenario: Live credential unavailable

- **Given** a provider cannot be tested with a safe runtime credential
- **When** the validation-hardening phase closes provider compatibility status
- **Then** the phase evidence MUST classify the provider as credential-blocked rather than recording a live secret

