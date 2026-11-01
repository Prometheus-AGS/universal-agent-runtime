## Why

Three issues remain on the identity and provider-stream boundaries for 1.4.0-rc.2:

- **#422:** UAR validates externally issued access JWTs but lacks parts of a frozen contract. A JWKS outage falls back to anonymous when JWT is optional, an explicit token `typ` is not checked, and several J01–J10 checks have no named test.
- **#392:** with optional JWT, an invalid bearer silently becomes anonymous. A local caller without credentials has no stable owner, so memory and other verified-owner features refuse it.
- **#380:** the native Anthropic SSE decoder lacks the exhaustive byte-partition acceptance test.

## What Changes

- **Invalid bearer:** a bearer that is present but fails verification gets 401, whatever `jwt_required` says.
- **JWKS outage:** a configured JWKS fetch failure or timeout gets 503 and never anonymous access.
- **Token type:** an explicit JWT `typ` must be `JWT` or `at+jwt`.
- **Local caller:** with the new opt-in `security.local_loopback_owner` (default false), on the TrustedLocal profile with optional JWT, a no-credential caller on loopback gets the existing `local-host` principal. It is opt-in because a same-pod proxy or mesh sidecar (for example Istio, which forwards from 127.0.0.6) would otherwise make remote callers the local owner. Non-loopback no-credential callers stay anonymous as before, and the Remote profile is unchanged.
- **Peer address:** the server records the TCP peer address (`ConnectInfo`) for the loopback check. No forwarded header is trusted.
- **Docs:** new `docs/security/jwt-validation.md` covers the supported profile, the time tolerance, the status mapping and the local identity assumption.
- **Tests:** named cargo tests for J01–J10, and an Anthropic SSE byte-partition test.
- **BDD:** `local-identity.feature` for #392.

## Impact

- Code: `src/uar/security/middleware.rs`, `src/uar/security/verifier/mod.rs` and the serve calls in `src/server.rs`.
- Tests: `src/llm/anthropic_sse.rs`.
- Behavior change: clients that send a malformed or expired bearer to a JWT-optional UAR now get 401 instead of anonymous access.
