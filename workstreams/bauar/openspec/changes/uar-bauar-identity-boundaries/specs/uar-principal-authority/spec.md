# Spec Delta

## Purpose

Preserve verified user, tenant and service authority across UAR authentication, key delegation and run admission in separately declared local and remote deployments.

## ADDED Requirements

### Requirement: Explicit deployment identity profile
UAR SHALL distinguish trusted-local launch from remote multi-user admission. Remote mode MUST require configured issuer, audience, allowed algorithm, expiry and not-before validation and a verified subject and tenant for tenant-scoped operations. Unconfigured remote authentication MUST fail closed.

#### Scenario: Remote policy is incomplete
- **WHEN** remote mode lacks any required verifier policy or trusted tenant mapping
- **THEN** protected admission is unavailable with a non-secret configuration diagnostic and no anonymous or installation-principal fallback.

#### Scenario: Local launch remains distinct
- **WHEN** an explicitly trusted-local host presents a valid installation/launch credential
- **THEN** local admission preserves its installation provenance without treating that credential as a remote user's tenant identity.

### Requirement: JWT validation at admission
UAR SHALL validate JWT signature, allowed algorithm, issuer, audience, required expiry and not-before before admitting work. The initial profile SHALL retain the existing 60-second clock leeway. Caller headers and model arguments MUST NOT supply verified identity.

#### Scenario: Negative JWT matrix
- **WHEN** tokens have a wrong or missing issuer/audience, missing expiry, disallowed algorithm, invalid signature, expiry more than 60 seconds in the past, or not-before more than 60 seconds in the future
- **THEN** authentication rejects them before admission, credential resolution or effects.

#### Scenario: Valid tenant and workspace authority
- **WHEN** a valid tenant A token selects a workspace or run belonging to tenant B without a trusted authorization mapping
- **THEN** access is denied without B's data or lifecycle mutation, while A's authorized workspace remains usable.

### Requirement: Bounded signing-key freshness
Remote JWKS validation SHALL use monotonic age since the last successful complete refresh, with a 300-second hard maximum and 60-second refresh target. Failed fetches MUST NOT reset age. At the hard deadline, unresolved refresh MUST deny new authentication. Refresh work SHALL be single-flight per issuer, bounded by a five-second timeout and at least five seconds between attempts.

#### Scenario: Removed or replaced key
- **WHEN** the issuer removes a key or replaces its material under the same key identifier
- **THEN** new authentications cease accepting the old key no later than 300 seconds after the last successful refresh, including when refresh fails; a successful refresh atomically removes superseded material.

#### Scenario: Refresh storm or outage
- **WHEN** concurrent requests present unknown keys or reach refresh time during an outage
- **THEN** at most one issuer refresh runs concurrently, attempts respect the minimum interval, authentication using an unexpired last-good key remains bounded by the hard deadline, and no unknown key is accepted.

### Requirement: Attenuated API-key authority
New API keys SHALL retain their verified issuer subject and tenant. Their roles MUST be a subset of the caller's delegable roles and the service's delegation allowlist. Reserved host, installation and administrative provenance MUST NOT be created through ordinary key issuance.

#### Scenario: Ordinary caller requests reserved authority
- **WHEN** an ordinary authenticated caller requests a host-session or otherwise nondelegable role
- **THEN** issuance rejects the request with no key created and no partially elevated credential returned.

#### Scenario: Tenant survives both supported key paths
- **WHEN** a scoped key is used directly or exchanged through a supported exchange mode
- **THEN** the resulting principal preserves the same verified tenant and attenuated authority; neither path synthesizes tenant from request fields.

#### Scenario: Legacy or incompatible exchange mode
- **WHEN** a stored key lacks required remote ownership/tenant metadata, or local token issuance cannot satisfy the active remote verifier
- **THEN** remote use or exchange is rejected with a reissue or unsupported-mode diagnostic; missing metadata is not inferred from the current requester.

### Requirement: Delegable role policy is explicit
The service SHALL expose security.api_key_delegable_roles, defaulting to [user]. Requested roles MUST be a subset of both verified caller roles and that allowlist. host-session and admin SHALL be reserved and forbidden in the allowlist. Installation/host provenance MUST never derive from any key role. Invalid configuration MUST fail closed.

#### Scenario: Concrete persisted attenuation
- **WHEN** verified caller roles are [user, reader], configured allowlist is [user], and a key requests [user]
- **THEN** the stored roles and both resulting principal paths equal [user], retain verified subject/tenant, and contain no host or administrative provenance; requesting reader, host-session or admin is denied without creating a record.

#### Scenario: Default or empty caller authority
- **WHEN** requested roles are omitted
- **THEN** [user] is used only if permitted by both caller roles and configured allowlist; a caller without that verified role receives a denial rather than a manufactured role.

### Requirement: Owner-authorized key management
Key listing and revocation SHALL require verified identity and apply owner-plus-tenant or explicitly authorized administrative policy. Revoked/expired keys MUST fail direct use and exchange. Exchanged token lifetime MUST NOT exceed the lesser of one hour and remaining key lifetime.

#### Scenario: Cross-owner revoke
- **WHEN** user B attempts to list or revoke user A's key without scoped administrative authority
- **THEN** B receives no key metadata and cannot revoke A's key; authorized owner or administrator revocation succeeds.

#### Scenario: Revocation window is visible
- **WHEN** a key is revoked after a JWT has already been issued
- **THEN** direct key use and new exchange fail, and the documented independently issued JWT validity window remains bounded by its expiry unless the chosen deployment provides earlier revocation enforcement.

### Requirement: Host authority cannot be self-minted
Host grant admission SHALL require typed provenance established by an authenticated launch or explicitly configured host/service authority. A user-controlled role, agent identifier or header MUST NOT independently establish host trust.

#### Scenario: Forged host role with valid downstream token
- **WHEN** a normal principal presents a minted host role, a registered destination and a downstream-valid resource credential
- **THEN** UAR denies host-grant admission before effects; an independently authenticated intended host with equivalent permitted grant succeeds.
