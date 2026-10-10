# API Key Authentication

The UAR uses a **Personal Access Token (PAT)** system for API authentication. Tokens are exchanged for short-lived JWTs for request authorization.

## Flow

```
Client                          UAR Auth API
  │                                  │
  │── POST /api/auth/keys ──────────▶│  (create PAT)
  │◀── { key_id, token } ────────────│
  │                                  │
  │── POST /api/auth/exchange ───────▶│  (exchange PAT → JWT)
  │   { token: "<pat>" }             │
  │◀── { access_token, expires_in } ─│
  │                                  │
  │── GET /api/uar/specs ────────────▶│  (use JWT)
  │   Authorization: Bearer <jwt>    │
  │◀── [ ... ] ──────────────────────│
```

## Creating a PAT

```bash
curl -X POST http://localhost:3928/api/auth/keys \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-ci-token",
    "description": "Used in CI pipeline"
  }'
```

Response:
```json
{
  "key_id": "key_abc123",
  "token": "uar_pat_xxxxxxxxxxxxxxxxxxxx",
  "name": "my-ci-token",
  "created_at": "2026-02-18T09:00:00Z"
}
```

> **Important:** The `token` value is only shown once. Store it securely.

## Exchanging for a JWT

```bash
curl -X POST http://localhost:3928/api/auth/exchange \
  -H "Content-Type: application/json" \
  -d '{ "token": "uar_pat_xxxxxxxxxxxxxxxxxxxx" }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Using the JWT

Include the JWT in the `Authorization` header for all API requests:

```bash
curl http://localhost:3928/api/uar/specs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Listing Keys

```bash
curl http://localhost:3928/api/auth/keys \
  -H "Authorization: Bearer <jwt>"
```

## Revoking a Key

```bash
curl -X DELETE http://localhost:3928/api/auth/keys/<key_id> \
  -H "Authorization: Bearer <jwt>"
```

## Identity profiles

The default `security.deployment_profile: trusted_local` preserves the existing
local launch contract. An authenticated launch principal remains installation
identity; it cannot replace a verified remote user or tenant.

Remote deployments set `deployment_profile: remote`, `jwt_required: true`,
`jwt_validate_nbf: true`, a nonempty `jwt_issuer` and `jwt_audience`, and an
explicit `jwt_algorithm`. Select HS256 for a deliberate shared signing secret,
or RS256 with the application-configured `jwks_url`. A token cannot choose the
algorithm. Missing or inconsistent remote policy disables protected admission.
No external identity provider or signing custody service is selected by UAR.

JWT verification requires expiry, validates not-before when present, and retains
60 seconds of clock leeway. Remote tokens also require a nonempty subject and
verified tenant. Caller headers and A2A parameters do not establish either.

`security.workspace_authorities` is an operator-owned list of exact
`issuer`, `subject`, `tenant_id`, and `workspace_id` records. Remote policy
requires at least one mapping; identities not in the list cannot enter protected
routes. The issuer must equal the configured verifier issuer. HTTP and gRPC
`x-uar-workspace-id` and A2A body `workspace_id` values select only workspaces
mapped to that verified identity. Cross-workspace denial precedes data lookup
or lifecycle mutation. Existing owner checks continue to apply inside services.

The fields participate in the derived configuration schema. Remote mode is a
strict opt-in: incomplete legacy tokens/configurations need explicit application
configuration; they are not silently assigned a tenant. Local configuration can
explicitly disable JWT using `security.jwt_required: false` where appropriate.

Identity profile source scenarios are authored with this change. HTTP, gRPC and
A2A runtime acceptance remains deferred to the complete production delivery gate.

## Key delegation and retained identity

Issuance requires authenticated user context. Requested roles must be held by
the caller and present in `security.api_key_delegable_roles` (default [user]).
Matching is case-sensitive. Omitted roles request [user] only when the caller
actually holds it; the service rejects the entire request before storage if
any role is outside either set. host-session and admin are always reserved and
are invalid allowlist configuration. No key carries installation or host proof.

Records now carry authority_version=1, issuing verifier issuer, verified subject
and optional local/required remote tenant. Direct X-API-Key authentication and
supported JWT exchange preserve that identity and the exact attenuated roles.
The bearer credential takes precedence when both credential types are supplied.
Exchange accepts a key in X-API-Key or api_key JSON body and performs its own
credential verification. Returned fields are token, token_type and expires_in.

Remote use rejects legacy records without versioned ownership/tenant metadata
or whose stored issuer differs from the active verifier. Reissue through the
verified owner's application flow; the runtime never guesses a tenant from
headers, current requester or workspace. Local HS256 exchange includes the
retained issuer and configured audience. External-JWKS exchange is explicitly
unsupported because this runtime cannot issue a token under that verifier.

The only backend in this release is InMemoryApiKeyStorage. It retains Argon2
hashes, not raw keys, and keys disappear on restart. No database or data
migration is performed. These are source-authored contracts pending the
combined delivery acceptance gate; runtime acceptance remains deferred.

## JWKS freshness

A configured issuer/endpoint pair shares one refresh lane. Key sets refresh
after 60 seconds or when an unknown key identifier is requested, with at least
five seconds between attempts. Concurrent requests join the existing attempt.
The complete set and its monotonic success time are published together; removed
keys and replacement material under an existing identifier replace the old set.

Failed fetches do not renew age. A last-good known key is usable only before
300 seconds since the last successful complete refresh. At that deadline, new
authentication fails closed until a successful refresh. Unknown keys never
receive stale fallback. A five-second total production-verifier budget covers
cache construction, registry/refresh waiting and network work. Timeout and
fetch diagnostics omit endpoint, key identifier, response body and token values.
These cooperative deadlines cannot preempt synchronous CPU work or scheduler
starvation; late completion is rejected rather than certified timely.

Source scenarios include key removal, actual different RSA material under the
same key identifier, outage/hard age, concurrent unknown keys, retry spacing
and delayed fetch. They have not been run; acceptance also needs the protected
router, body-delay/cancellation/log-sentinel scenarios and final build receipt.

## Management authority and token lifetime

Issuance, listing and revocation require an internal verified identity proof.
The proof is constructed only after JWT or stored-key credential validation;
it is not deserializable and must still match the context's subject, issuer,
tenant and roles. Anonymous, named synthetic and mismatched contexts cannot
create or manage keys. Authenticated local installation provenance alone is
not user key-management authority.

Owners may see and revoke only records matching their issuer, subject and
tenant. `security.api_key_admin_principals` defaults to an empty list of exact
issuer/subject/tenant triples. A matching administrator may manage other owners
only within that same issuer and tenant. An admin role alone grants no such
power. Denied listing reveals no foreign metadata; denied revocation returns
the same not-found result as an absent key and leaves the record unchanged.

An exchanged JWT expires no later than the lesser of 3600 seconds, configured
service TTL and remaining key lifetime. The exchange response returns the
actual expires_in. A key at its expiry is invalid for both direct use and new
exchange. Revocation disables those two paths immediately, while already-issued
self-contained JWTs retain their expiry and the verifier's configured 60-second
clock leeway. No immediate retroactive JWT revocation is promised.

Owner/tenant denial, exact scoped administration, synthetic-context denial,
remaining lifetime and revocation source scenarios are authored but unexecuted.
The final production acceptance gate must verify these through real routes.

## Host provenance and privileged credential cutover

Host grant admission consumes an internal, non-deserializable HostAuthority.
Roles such as host-session or uar:mcp:delegate, instance identifiers and request
headers cannot create it. Local SidecarGuard launch authentication produces a
private marker, consumed with the admitted local principal assertion. This
installation proof is separate from verified user/tenant identity and does not
permit API-key issuance or management by itself. A valid launch assertion is
handled before ordinary JWT resolution because the guard already consumed its
launch bearer; remote mode still refuses installation assertions.

security.trusted_host_principals defaults to []. Each configured entry contains
exact nonempty issuer, subject, tenant_id and host_id strings. Its issuer must
equal the enforced verifier issuer. Only a verified JWT with an issuer-controlled
signed uar_credential_kind claim equal to issuer can activate that mapping.
The resulting host_id must still be in the destination's trusted_hosts; destination,
scope, lease, owner and renewal checks remain applicable. No remote identity
provider, service, server preset or credential custodian is supplied by default.
These fields participate in the derived configuration schema.

New API-key exchanges sign uar_credential_kind=api_key; direct keys carry the
same delegated kind internally. Both remain excluded from service-host authority
and configured cross-owner API-key administrator authority, even when their
subject matches those mappings. Owner management and attenuated user roles
remain available. Cross-owner administrator mappings also require the explicit
signed issuer kind in addition to the exact configured identity triple.

This is a deliberate privileged credential cutover: absent or unknown kind is
unclassified, including legacy exchanged JWTs. Omission cannot establish that a
token was issued independently of a key. Ordinary admission and owner management
remain compatible, but privileged service/admin credentials must be reissued by
the application's trusted issuer with the issuer-controlled signed marker before
those mappings can work. Do not copy caller-supplied kind values into issuer
claims. No unsigned header, body value or legacy-token omission is a substitute.

Source scenarios pair denied forged-role, delegated and legacy principals with
a registered destination and an independently accepted downstream credential.
They also cover exact service admission, real launch-guard proof, tampered proof
binding and preserved lease/scope denial. They are authored only; no scenario,
compiler or runtime acceptance gate has run for this change.
