## Purpose

Keep trusted UAR coordination controls and paired-host effects within their correct admission authorities while preserving the existing durable execution lifecycle.

## ADDED Requirements

### Requirement: Prepared invocation v1 wire compatibility

PreparedToolInvocation SHALL serialize governancePolicyRevision for the v1 host parser. Canonical authority hashes SHALL retain their intentional expectedGovernancePolicyRevision key.

#### Scenario: Host validates a filesystem effect

- **WHEN** UAR submits a prepared v1 invocation to Boss
- **THEN** its governancePolicyRevision field satisfies the existing parser without changing authority digest semantics

### Requirement: Registered handler owns admission routing

Only a trusted registered handler explicitly marked as a UAR runtime control SHALL select local admission. Model inputs and deserialized invocation metadata SHALL NOT select ownership. Other native effects and mounted MCP effects SHALL retain existing host admission.

#### Scenario: Native peer operation

- **WHEN** an admitted model calls a registered team peer handler
- **THEN** UAR locally prepares, resolves, claims and settles it through the existing lifecycle with all current governance, approvals, budgets and claim revalidators

#### Scenario: Local claim interrupted

- **WHEN** cancellation interrupts an already claimed runtime control
- **THEN** its effects remain uncertain unless a terminal receipt establishes the result

### Requirement: Approval ownership is authoritative

Pending approvals SHALL expose captured admissionOwner as uar-runtime or paired-host. Missing ownership in older projections SHALL mean paired-host. Required descriptor approvals SHALL remain required.

#### Scenario: Local delegation approval

- **WHEN** a team delegation requires a human decision
- **THEN** the owner-scoped UAR waiter carries uar-runtime ownership and the normal approval identity without requiring a filesystem bridge admission record
