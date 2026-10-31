# JWT validation boundary

UAR validates externally issued, signed access JWTs. It does not issue tokens,
run an identity provider, or decide what an authenticated caller may do. A
successful validation means one thing: the presented credential is valid for
the trust profile the operator configured. Roles, scopes, tools, resources and
business permissions belong to agent code, the trusted host, or downstream
services.

Source of truth: `src/uar/security/verifier/mod.rs` (verifier),
`src/uar/security/middleware.rs` (HTTP ingress), `src/config.rs`
(`SecurityConfig`). Contract: GitHub issue #422 (J01-J10). Local identity:
GitHub issue #392.

## Supported access-token profile

One profile, selected by operator configuration under `security:`. Nothing in
the token selects or widens it.

| Key | Meaning |
| --- | --- |
| `deployment_profile` | `trusted_local` (default) or `remote`. |
| `jwt_required` | `true` (default): a request without a credential is refused. `false`: see "Requests without a credential". |
| `jwks_url` | When set, the JWKS verifier is used: asymmetric keys from this URL only. |
| `jwt_secret` | When `jwks_url` is absent, the HS256 shared-secret verifier is used with this secret. |
| `jwt_algorithm` | `HS256` or `RS256`. Optional locally, required in `remote`. Must agree with the verifier: `HS256` without `jwks_url`, `RS256` with it. |
| `jwt_issuer` | Expected `iss`. When set, `iss` is required and must match exactly. |
| `jwt_audience` | Expected `aud`. When set, `aud` is required and must contain this value. |
| `jwt_validate_nbf` | `true` (default): a present `nbf` is enforced. Required `true` in `remote`. |
| `local_loopback_owner` | `false` (default). Opt-in local-host owner for credential-less loopback callers; see "Local-host loopback identity". Ignored unless `trusted_local` and `jwt_required: false`. |

The `remote` profile fails closed unless `jwt_required`, `jwt_validate_nbf`,
`jwt_algorithm`, a non-empty `jwt_issuer`, a non-empty `jwt_audience`, a
deliberate signing secret (shared-secret mode) and exact
`workspace_authorities` are all configured. A missing or contradictory value
returns 503 for every request rather than admitting anyone.

### Algorithms and keys

- Shared-secret mode accepts HS256 only.
- JWKS mode accepts RS256 (RSA keys), ES256 (EC P-256) and ES384 (EC P-384).
  Each key is bound to one algorithm by its own `kty`/`crv` and declared `alg`,
  never by the token header. An ES256 token naming an RSA key, an RS256 token
  naming an EC key, HMAC on the JWKS path, `none`, and any other algorithm are
  rejected before signature verification.
- `jwt_algorithm: RS256` selects the JWKS verifier family. It does not narrow
  that family to RSA alone: a configured JWKS that publishes an EC key can
  verify ES256/ES384 tokens. Publish only the keys you intend to trust.
- `kid` is required on the JWKS path and only selects among keys fetched from
  the configured `jwks_url`. The header fields `jku`, `x5u`, `x5c` and `jwk` are
  never read. A token cannot point UAR at another key source, and an unknown
  `kid` never falls back to another key or issuer.
- JWKS fetches are bounded: one 5-second budget per verification, a minimum
  5-second interval between refreshes, a 60-second refresh target and a
  300-second hard age after which cached keys are refused.

### Token purpose (J08)

If the JWT header carries an explicit `typ`, it must be one of `JWT`,
`application/jwt`, `at+jwt` or `application/at+jwt`, compared
case-insensitively. Any other declared type, such as an ID-token, logout,
DPoP or security-event type, or an empty string, is rejected before a key is
fetched or a claim is read.

`application/jwt` is accepted because RFC 7515 §4.1.9 allows the
`application/` prefix to be omitted; it names the same type as `JWT`, just as
`application/at+jwt` names `at+jwt`.

A token without `typ` is accepted only under the exclusive profile above: the
exact issuer, the configured audience, and keys from the configured source.
This is the uncomfortable part. Most identity providers sign ID tokens with the
same keys and issuer as access tokens and label both `typ: JWT`. UAR cannot
tell them apart by type. The audience is the separation: configure
`jwt_audience` to a value your provider places only in access tokens for UAR
(an API identifier or resource URI), never a value it also uses as an OAuth
client ID, because ID tokens carry the client ID as their audience. Without
`jwt_audience` there is no purpose separation at all, which is why `remote`
requires it.

### Claims and time (J04-J06)

- `exp` is always required and must be a number.
- `iss` and `aud` are required and must be well-typed when `jwt_issuer` and
  `jwt_audience` are configured. `aud` may be a string or an array; an array is
  accepted when it contains the configured audience.
- `nbf` is optional. When present and `jwt_validate_nbf` is true it is enforced.
- Clock tolerance is a fixed 60 seconds. It is not configurable. With `now` in
  whole Unix seconds at verification time:
  - a token is expired, and rejected, only when `exp < now - 60`; `exp` equal
    to `now - 60` is still accepted;
  - a token is not yet valid, and rejected, only when `nbf > now + 60`; `nbf`
    equal to `now + 60` is still accepted.
  The J05 tests use 45-second (inside) and 75-second (outside) margins so they
  do not depend on which second the verifier reads.
- `sub`, `roles`, `tenant_id` and `name` are carried into the verified context
  as identity data. UAR does not interpret roles or scopes as permissions at
  this boundary.

## Status mapping

| Situation | HTTP status | Context |
| --- | --- | --- |
| Valid credential | request proceeds | verified context only |
| Bearer present but malformed, unsigned, tampered, expired, not yet valid, wrong issuer/audience/type, unknown `kid` against a fresh key set, disallowed algorithm | 401 | none |
| Configured JWKS unreachable, failing, timed out, invalid key set, or past hard age | 503 | none |
| Identity policy misconfigured or contradictory | 503 | none |
| Process JWT crypto-provider conflict | 500 | none |
| No credential, `jwt_required: true` | 401 | none |
| No credential, `jwt_required: false` (default `local_loopback_owner: false`) | request proceeds | anonymous |
| No credential, `jwt_required: false`, `trusted_local`, `local_loopback_owner: true`, loopback peer, loopback `Host`/`Origin` | request proceeds | local-host owner |

An unknown `kid` is 401, not 503, when the configured JWKS was fetched
successfully and simply does not contain it: the key source is available and
the credential names a key it does not publish. Only failure to obtain the key
set (unreachable, error status, timeout, invalid set, past hard age) is 503.
When a refresh has failed, an unknown `kid` reports that failure and is 503.

These mappings hold regardless of `jwt_required`: a presented bearer that does
not verify is never downgraded to anonymous, and an unavailable key source is
never reported as success or as a bad credential. Equivalent failures map to
the same status on every attempt. Response bodies carry no error text. Logs
carry the failure category only; tokens, signatures, secrets and key material
are never logged, and the admitted bearer is registered with the run's secret
scrubber so it cannot reach model prompts or events.

The gRPC A2A ingress (`src/uar/api/a2a/grpc.rs`) uses the same verifier but
its own mapping: provider conflict is `INTERNAL`, identity policy is
`UNAVAILABLE`, and every other failure, including JWKS unavailability, is
`UNAUTHENTICATED`. It has no anonymous or loopback path.

## Requests without a credential

With `jwt_required: false`, a request that presents no `Authorization` and no
`x-api-key` header is admitted without a verified credential. What identity it
gets depends on where the connection came from.

### Local-host loopback identity (#392, opt-in)

This behavior is off by default. With `security.local_loopback_owner: false`
(or the key absent), every credential-less request keeps the anonymous context,
exactly as before #392. Enable it only for a single-user desktop or host
install where UAR is reached directly, never through a proxy.

With `local_loopback_owner: true`, `deployment_profile: trusted_local` and
`jwt_required: false`, a credential-less request becomes the single local owner
(`issuer_kind: local-host`, issuer `urn:uar:local:host`, no tenant, subject
`local-user`) when all of these hold:

- the accepted TCP peer address is loopback: `127.0.0.0/8`, `::1`, or
  IPv4-mapped `::ffff:127.0.0.0/104`. The address comes from the socket only.
  `X-Forwarded-For`, `X-Real-IP`, `Forwarded` and every other header are
  ignored for this decision;
- the `Host` header (or HTTP/2 authority) is a loopback name: `localhost`, a
  loopback IPv4 address, or `[::1]`, with any port;
- there is no `Origin` header, or it is an `http`/`https` loopback origin.

This is the same principal an authenticated sidecar launch produces, so
conversation history, memory, knowledge bases and tool credentials all resolve
to one owner key. It carries no host authority and no roles: host
administration (memory administration, supervisor, credential migration,
remote MCP grants, accounting) stays unavailable, and it cannot create, list or
revoke API keys. Stored provider credentials for this owner require
`security.provider_credentials.installation_id`, as for every local owner.

Every other credential-less request, including any non-loopback peer, keeps the
existing anonymous context. The flag is ignored under `remote` (which requires
a JWT) and when `jwt_required: true`.

### The trusted-local assumption

`trusted_local` asserts that every process able to open a loopback connection
to UAR acts for its one local owner. That includes other user accounts on the
same machine, local malware, and anything that forwards traffic to loopback.

The scenario that hurts, and the reason the flag defaults off: a reverse
proxy, tunnel (`cloudflared`, `ngrok`), SSH port forward or service-mesh
sidecar on the same host or pod connects to UAR over loopback. Every request
it forwards arrives from a loopback peer; Istio's sidecar, for example,
forwards inbound traffic from `127.0.0.6`. The `Host` header is client data:
any remote client can send `Host: localhost`, and many proxies pass it through
or rewrite it to the upstream's loopback name. With the flag on, a forwarded
credential-less request then becomes the local owner and can read that owner's
history, memory and knowledge. UAR cannot see past the proxy and deliberately
does not trust forwarding headers. The `Host`/`Origin` checks stop browser
pages (DNS rebinding, cross-origin requests); they do not stop a proxy or a
non-browser client.

Never enable `local_loopback_owner` behind any proxy, tunnel or mesh. Anything
reachable from another machine must use `deployment_profile: remote`, or
`jwt_required: true` with a proxy that always presents a verified bearer.

Data note: while the flag is off, credential-less loopback callers store
conversation history under the shared `anonymous` owner. Turning the flag on
does not migrate that history to the `local-host` owner; it stays in storage
but is no longer visible to those callers. Turning the flag off again makes it
visible again and hides the `local-host` owner's data.

## What UAR does not do

- No IAM, authorization or policy decisions from token claims at this boundary.
- No token issuance, exchange, refresh or revocation service. (The separate
  API-key exchange signs short-lived UAR tokens for stored keys; it is not part
  of this validation contract.)
- No issuer discovery and no trust derived from the token: no `jku`, `x5u`,
  `x5c`, embedded `jwk` or issuer metadata fetch.
- No DID, verifiable-credential or SD-JWT verification.
- No configurable clock tolerance or multi-profile negotiation.
- No forwarding of the bearer to new destinations.
