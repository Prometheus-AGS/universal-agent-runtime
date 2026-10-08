# Spec Delta

## Purpose

Allow a trusted paired host to provide private run resources to a scoped delegation while retaining host approval authority and native UAR execution ownership.

## ADDED Requirements

### Requirement: Exact private delegated context

The runtime SHALL accept context registration only from its authenticated host, bind it to a live verified delegation grant, workspace, runtime epoch and immutable catalog-resolved binding, and disclose only safe context identity.

#### Scenario: Ordinary bound delegation
- **WHEN** the exact grant admits a bound task using its registered opaque context reference
- **THEN** UAR resolves the current bound agent and injects the original private resources without exposing them to the delegated caller or persisted records.

#### Scenario: Context substitution or expired authority
- **WHEN** a caller substitutes grant, workspace, binding, definition or resources, or original authority expires or is revoked
- **THEN** admission or protected effect claim refuses with an actionable stable code and does not adopt replacement authority.

### Requirement: Host decision precedes native approval resolution

For a context-bound task the runtime SHALL coordinate the exact live pending issuer/challenge/admission/run with the private host and require its recorded decision before resolving the existing UAR approval waiter.

#### Scenario: Approved or denied effect
- **WHEN** the host validates and records the exact human decision and returns its matching acknowledgement
- **THEN** UAR rechecks the original lease and pending challenge, resolves its existing waiter and retains existing effect/budget settlement ownership.

#### Scenario: Failed callback or pending cancellation
- **WHEN** host coordination fails, its echo differs, or cancellation removes the pending challenge
- **THEN** UAR does not resolve a different waiter or dispatch an unauthorized effect.

### Requirement: Admission retry and existing callers

The opaque context reference SHALL participate in the existing admission digest and preserve original admission identities and behavior for callers without a context.

#### Scenario: Repeated exact admission
- **WHEN** an exact admission is retried
- **THEN** the existing retained admission response is returned without a second execution or replacement context.
