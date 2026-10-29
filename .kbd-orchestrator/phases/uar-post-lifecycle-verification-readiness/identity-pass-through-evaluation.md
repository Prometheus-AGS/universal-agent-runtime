# Evaluation — pass-through JWT identity

2026-10-28. Read-only evaluation, for an operator decision. Nothing changed and
nothing ran.

**Evidence:**

- A security-owner survey of UAR source (identity and security modules, cited
  inline).
- The MCP security best-practices specification, fetched 2026-10-28:
  <https://modelcontextprotocol.io/specification/draft/basic/security_best_practices>.

## The proposal (operator, 2026-10-28)

1. Accept JWTs presented on any interface (AG-UI, A2A, OpenAI, Anthropic, and
   others).
2. Do nothing with them. No authentication, no roles, no validation.
3. Hold them and forward them. Agents and tools carry out the UAR owner's
   intent.
4. Treat each distinct `sub` as a different user.
5. Offer hooks so the owner of a UAR node can add validation if they want.

## What UAR does today

| Topic | Today | Source |
|---|---|---|
| Validation | HTTP middleware verifies bearer JWTs: HS256 with a shared secret, or RS256/ES256/ES384 via JWKS (`kid` required). `exp` is required, and `iss`/`aud` are checked if configured. With `jwt_required=false`, a missing or invalid token becomes `anonymous`, not an error. gRPC always requires and verifies a bearer. | `security/middleware.rs:195-310`; `security/verifier/mod.rs:131-183, 260-329`; `api/a2a/grpc.rs:51-140` |
| Partition key | Never `sub` alone. **Key A** (kind, configured issuer, tenant, sub) covers sessions, runs, AG-UI, ACP, KBs, memory, approvals and chat. **Key B** (tenant, sub) covers user settings, teams, MCP-bound runs and prompt caching. The issuer comes from configuration, not the token. | `security/claims.rs:164-242`; `actor/messages.rs:126-145` |
| Roles | `admin` gates effective settings. `operator`, `admin` or `host-session` gates observers and agent reconciliation. Roles also gate API-key delegation and `skill_required_roles`. | `effective_settings.rs:43`; `observers.rs:251`; `agent_instances.rs:330`; `api_keys/policy.rs:82-87`; `manager.rs:1709` |
| Host and peer authority from claims | `HostAuthority` comes from a token matching `trusted_host_principals`, and unlocks memory admin, credentials, migration and supervisor. `uar_instance_id` gates A2A delegation contracts. Shared provider credentials require a verified issuer, tenant and subject. | `claims.rs:121-130`; `routes.rs:1733`; `docs/operations/principal-provider-credentials.md:39-42` |
| Forwarding | **None.** The inbound token is captured only to scrub it from errors, events, history and checkpoints. Outbound credentials come from configuration. | `credential_capture.rs:1-29`; `manager.rs:6281-6306`; `mcp/runtime.rs:106-125` |
| Local tests | The identity proxy strips the client header and mints an HS256 token on every request, signed with UAR's secret, with `sub` from `PROXY_JWT_SUB` and default roles `admin,user`. | `tools/uar-jwt-proxy/src/main.rs:44-69, 343-394` |

**UAR already treats each subject as a different user, scoped by issuer and
tenant.** Two proxies with different `PROXY_JWT_SUB` values against one UAR
instance already give two users. Today's E09 blocker is only the second proxy
process, not UAR itself.

## Assessment

### Part 1 — sub as the user, no validation inside UAR: sound only with a declared trust boundary

**What is right about it.** UAR should not be an identity provider. The
**trusted front** pattern is common and sound: a gateway (Flint Gate,
Oathkeeper, an Envoy filter) validates the token, and the service trusts what
reaches it. It would:

- remove the San Saba blocker that HS256 shared secrets don't fit RS256
  identity;
- remove signing secrets from local test infrastructure, since the proxy would
  no longer need UAR's secret;
- make multiple users cheap.

**What is wrong with it unqualified.** If UAR never validates, then `sub` is
whatever the caller writes:

- Anyone who can reach UAR can forge a token with someone else's `sub` and
  read or write that user's sessions, memory, KBs, settings and teams.
- Isolation then holds only while every request passes through a validating
  front.
- The repository is public, and a default that is unsafe when exposed will be
  deployed exposed by someone.

**The proposal needs one addition: UAR must know where validation happens.**
The node's configuration declares one of three things:

| Declaration | Meaning |
|---|---|
| `trusted-front` | A named upstream validates. UAR binds loopback or a private network only. |
| A validation hook | A JWKS, an issuer and audience, or an owner plugin, which runs before identity is used. |
| `single-user` | Every request is the same owner. |

UAR still does no authentication of its own. It only refuses an unsafe
combination: multiple users, no declared validator, and a public bind. That is
compatible with "we do not provide authentication", because the hook is the
owner's choice and UAR ships no identity provider.

**Partition by (`iss`, `sub`), not `sub` alone.** Subjects are unique only
within an issuer. Two honest identity providers can both issue `sub=123`, and
`sub` alone would merge their users. Tenant stays optional, for San Saba-style
isolation. Under no validation these are still asserted values. The key only
keeps honest issuers apart.

**What has to change, because it depends on verified claims:**

| Area | Change |
|---|---|
| Roles (settings, observers, reconciliation, API-key delegation, `skill_required_roles`) | Removed, per the proposal. These features move to authorities that don't come from the JWT. `X-UAR-Admin-Key` and the sidecar launch token already exist. Otherwise the owner hook decides. |
| `HostAuthority` from `trusted_host_principals` | Must not be grantable by an unverified token. Use launch-bound host authority only. |
| A2A `uar_instance_id` delegation | Must not trust an unverified claim. Peer authority comes from peer configuration. |
| Shared provider credentials | Disabled unless a validation hook is configured. Otherwise any asserted tenant can spend shared provider keys. |
| Existing data keys (`v1:k:…`) | A new key derivation orphans existing rows unless they are mapped or migrated. This is a decision (Q4). |
| The remote profile (`validate_identity_policy`) | Becomes a "validated" profile beside the new pass-through profile, rather than being relaxed. |
| gRPC | Same rules as HTTP. It still needs a bearer in metadata to read `iss`/`sub`. |

### Part 2 — forwarding the presented JWT to agents and tools: not recommended as a default

- **The MCP specification forbids it.** "Token passthrough is explicitly
  forbidden in the authorization specification." It also says "MCP servers
  MUST NOT accept any tokens that were not explicitly issued for the MCP
  server" (MCP security best practices, "Token Passthrough"). Standards-
  compliant MCP servers will reject a forwarded user token. Accepting servers
  create a **confused deputy**.
- **It hands out a replayable credential.** Any tool or peer that receives the
  token can replay it against the user's other services, within its lifetime.
  Tool descriptions are untrusted input.
- **It contradicts current hardening.** UAR deliberately forwards nothing today
  and scrubs the token from all output.
- **Lifetimes don't line up.** Teams, scheduled runs and action continuations
  outlive a token's `exp`.

**Recommendation:**

- Hold the token in memory for the request only. Never persist it, and keep
  the scrubber.
- Forward **only** to destinations the owner opts in, per MCP server or A2A
  peer. Prefer an owner hook that performs **token exchange** (RFC 8693) to
  mint an audience-bound token for that destination.
- A continuation uses the token of the request that resumes it, never a
  stored one.

### Effect on the blocked scenarios

- **E09 (owner isolation).** It runs on one instance either way, because UAR
  already separates subjects. It needs a second user identity from external
  infrastructure. Under pass-through, the identity proxy no longer needs a
  signing secret. It could map a non-secret selector header (for example
  `x-uar-test-subject: b`) to a `sub`, which fits the rule that tests carry no
  credentials, only non-secret selectors. A second proxy process also works.
- **I15-I18 (gRPC).** These still need a gRPC-capable proxy (or a selector
  equivalent), because tests can't place a bearer in gRPC metadata.
- **San Saba G12.**
  - RS256 incompatibility: resolved (UAR stops verifying, or a JWKS hook
    verifies).
  - `tenant_id` in claims: already read today.
  - A2A tasks partitioned by tenant: still needs checking.

## Uncomfortable points

- **UAR alone would provide no isolation.** With no declared validator,
  isolation depends entirely on deployment.
- **Forwarding fails with compliant tools.** Agents and tools "implementing the
  owner's intent" with a forwarded token will mostly be refused by MCP servers
  that follow the specification. The owner's intent will usually need token
  exchange, which is work in the owner hook, not none.
- **Removing roles removes features** unless a replacement authority is
  chosen: the admin settings view, observers, reconciliation, API-key
  delegation and skill role gates.

## Decisions for the operator

| Id | Question | Recommended |
|---|---|---|
| Q1 | Adopt pass-through identity (no validation and no roles inside UAR) with a mandatory trust declaration: `trusted-front`, a validation hook, or `single-user`? | Yes, with the declaration |
| Q2 | Partition key: `(iss, sub)`, `(iss, tenant, sub)`, or `sub` alone? | `(iss, sub)`, with tenant optional |
| Q3 | Forwarding: opt-in per destination plus an exchange hook, or always forward? | Opt-in plus exchange hook; never persist |
| Q4 | Existing data: migrate `v1:k` rows to the new key, or start fresh? | Map existing rows; decide in Plan |
| Q5 | Replacement authority for removed roles: admin key, host launch token, or owner hook? | Admin key and launch token for host features; owner hook for the rest |
| Q6 | Test identities: a non-secret subject-selector header on the local identity proxy, or one proxy per user? | Selector header (local proxy only) |

If adopted, this becomes its own KBD child phase, since it touches every
interface, the partition keys and stored data. The A2UI work does not depend
on it, but San Saba (G12), E09 and I15-I18 do.
