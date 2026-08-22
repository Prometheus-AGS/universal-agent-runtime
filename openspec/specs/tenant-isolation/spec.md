# tenant-isolation Specification

## Purpose
TBD - created by archiving change gap-03-a2a-tenant-partitioning. Update Purpose after archive.
## Requirements
### Requirement: Tenant identity is distinct from user identity and is derived only from verified claims
The runtime SHALL represent tenant identity as a claim distinct from the token
subject, and SHALL populate it only from a credential that has passed
verification. A tenant identity SHALL NOT be accepted from a request body, query
parameter, or header.

#### Scenario: Tenant derived from a verified token
- **WHEN** a request presents a verified token carrying a tenant claim
- **THEN** the resulting principal carries that tenant, and it is distinct from the subject

#### Scenario: Client-supplied tenant is ignored
- **WHEN** a request supplies a tenant identifier in its body, query string, or a header
- **THEN** the supplied value is ignored and only the verified claim is used

#### Scenario: Unverified token yields no tenant
- **WHEN** a token fails verification
- **THEN** no tenant identity is established and the request is rejected per the jwt-hardening rules

### Requirement: A2A task records are addressable only within their tenant
Both the task map and the context index SHALL be partitioned by tenant. A caller
SHALL NOT read, mutate, or cancel a task belonging to another tenant, including
when the caller supplies a correct task id or context id.

#### Scenario: Cross-tenant read by task id is denied
- **WHEN** tenant A creates a task and tenant B requests it by its exact task id
- **THEN** the response denies access and contains no field of tenant A's task

#### Scenario: Cross-tenant read by context id is denied
- **WHEN** tenant A creates a task with a context id and tenant B looks up that same context id
- **THEN** no task is returned, and tenant B does not join tenant A's context

#### Scenario: Cross-tenant cancel is denied
- **WHEN** tenant B attempts to cancel a task belonging to tenant A
- **THEN** the cancel does not take effect and tenant A's task remains in its prior state

#### Scenario: Same-tenant access is unaffected
- **WHEN** a caller accesses a task belonging to its own tenant
- **THEN** behaviour is unchanged from before partitioning

### Requirement: Absent tenant identity fails closed on tenant-scoped surfaces
When a request reaches a tenant-scoped surface without an established tenant
identity and JWT verification is required, the runtime SHALL refuse the request
rather than serving an unpartitioned view.

#### Scenario: No tenant claim with verification required
- **WHEN** `security.jwt_required` is true and a verified token carries no tenant claim
- **THEN** A2A task surfaces reject the request rather than falling back to a shared partition

#### Scenario: Negative control for cross-tenant denial
- **WHEN** the cross-tenant denial test is run against a build where the partition key is ignored
- **THEN** the test fails, demonstrating the assertion is capable of failing
