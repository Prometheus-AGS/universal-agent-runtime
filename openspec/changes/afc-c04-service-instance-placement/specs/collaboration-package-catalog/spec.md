# collaboration-package-catalog Specification Delta

## MODIFIED Requirements

### Requirement: Deployment bindings target the live runtime

An installed deployment binding MUST target the live configured UAR instance,
MUST require only capabilities implemented by that instance, and MUST retain
credential and connection identifiers solely as opaque references. Its effective
receipt MUST record the admitted service binding and MUST be revalidated before
activation and protected tool claims.

#### Scenario: Binding targets another runtime

- **WHEN** `runtimeInstanceId` differs from the live configured instance
- **THEN** preflight records a required-unsupported diagnostic
- **AND** activation is not admitted.

#### Scenario: Required capability is absent

- **WHEN** a binding requires a capability the live runtime does not advertise
- **THEN** preflight identifies the unsupported capability
- **AND** activation is not admitted.

#### Scenario: Runtime changes after binding installation

- **WHEN** the effective receipt no longer matches the live instance at run or tool admission
- **THEN** UAR refuses activation rather than using the stale receipt.

