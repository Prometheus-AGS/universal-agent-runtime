# Scoped delegation grants

## Purpose

Permit a trusted launching host to delegate explicit UAR operations to a native
consumer with a short-lived principal/workspace credential that cannot acquire
the launching host's broader authority.

## ADDED Requirements

### Requirement: Only the launching host issues and revokes grants

Managed-local UAR SHALL issue grants through POST `/api/uar/delegation-grants`
and revoke them through DELETE `/api/uar/delegation-grants/{id}` only with
original launch-host authentication. Issuance MUST capture a non-anonymous
verified principal and explicit nonempty workspace/operation scope.

#### Scenario: Host issues a private grant
- **WHEN** the original authenticated host submits approved principal, workspaces and operations
- **THEN** UAR returns a fresh 256-bit bearer grant with ID, expiry, scope, instance and full-harness runtime epoch
- **AND** the response prevents caching and the native authority keeps credentials private in memory

#### Scenario: A grantee requests administrative credential authority
- **WHEN** a grant caller attempts grant issuance, renewal, or revocation
- **THEN** UAR refuses the request without granting original host authority

### Requirement: Grants expire and runtime replacement invalidates them

The grant authority SHALL retain grants only in private native memory. Grant
lifetime SHALL be at most 900 seconds measured by monotonic time. Expiry,
explicit revocation, and runtime replacement MUST prevent subsequent admission
with the old credential. Repeated host revocation SHALL be idempotent.

#### Scenario: Revoked or expired grant
- **WHEN** a caller presents a revoked or expired grant
- **THEN** UAR refuses the request before route execution

#### Scenario: New runtime generation
- **WHEN** the managed-local runtime restarts with a new full-harness epoch
- **THEN** prior grants cannot authenticate and the host must obtain a new grant

### Requirement: Grant rights are explicit method path and workspace selections

Grant operations SHALL be `discovery`, `model_read`, `model_completion`, and
`full_harness_delegation`, with the exact allowlist in the protocol document.
`model_completion` SHALL allow only POST `/api/chat/completion`. Completion and
full-harness operations MUST carry exactly one matching workspace header.
Unlisted methods, paths, operations, and workspaces MUST fail closed.

#### Scenario: A supported selected model call
- **WHEN** a grant includes model_completion and posts native completion with a matching workspace
- **THEN** the existing chosen-model completion contract executes with the captured principal

#### Scenario: Different workspace or path
- **WHEN** a grant request carries an absent, duplicate or different workspace header for completion/task operations, or calls an unlisted route/method
- **THEN** the guard refuses the request before route execution

### Requirement: Grants preserve identity and original host separation

The sidecar guard SHALL require its exact authority and no Origin for grant
requests, strip Authorization before inner layers, and derive identity from the
recorded verified principal. Caller principal headers MUST be rejected. Grants
MUST NOT install the host-authentication marker. External UAR SHALL retain JWT
verified subject/tenant identity without host-header override.

#### Scenario: Spoofed principal
- **WHEN** a grant caller supplies x-uar-principal naming another user
- **THEN** UAR rejects the request rather than replacing the recorded principal

#### Scenario: Browser or wrong authority
- **WHEN** a grant request carries Origin or names an authority outside the managed-local sidecar
- **THEN** the guard refuses it before comparing credentials

#### Scenario: External JWT remains authoritative
- **WHEN** an external JWT-authenticated caller tries to supply x-uar-principal
- **THEN** the header cannot override the JWT's verified identity

### Requirement: Existing native execution authorities remain effective

Full-harness grant calls SHALL retain existing ownership, admission digest,
revision, approval, Cedar, effect authorization, and cancellation contracts.
Grants MUST NOT confer owner bypass or original-host paired tool-admission
rights. Credential revocation SHALL NOT implicitly cancel already admitted work.

#### Scenario: Other principal owns a task
- **WHEN** a scoped grant reads or controls another principal's full-harness task
- **THEN** the existing task authority refuses access

#### Scenario: Paired-host tool admission
- **WHEN** a grant supplies host-only tool-admission authority in an admission request
- **THEN** the existing original-host requirement refuses it

#### Scenario: Previously admitted work continues until explicit cancellation
- **WHEN** a grant is revoked after a native task was admitted
- **THEN** new requests are refused and existing work remains governed by the native cancellation/effect authorities
