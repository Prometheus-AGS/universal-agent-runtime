# JWT hardening deterministic verification summary

Profile scope: `server-full`, plus the separately named `uar-jwt-proxy` package
where stated. These results transfer to no other profile.

- Startup secret: PASS. Required JWT authentication rejects the published
  fallback with a clear configuration error, including after optional Vault
  resolution. Explicit anonymous mode remains available and is documented as
  sharing one non-isolated identity.
- Registered claims: PASS. HS256 and JWKS use the same issuer, audience, and
  optional not-before policy. Matching HS256 claims authenticate; missing or
  mismatched claims and future `nbf` return 401.
- Issued-token continuity: PASS. API-key exchange and `uar-jwt-proxy` include
  configured issuer and audience claims and decode under those rules.
- Sidecar compatibility: PASS. The sidecar's three focused tests preserve its
  loopback anonymous default only when the operator expressed no JWT opinion.
- Focused suites: PASS. Security 37/0, configuration 20/0, config integration
  5/0, config manager 3/0, sidecar 3/0, and proxy 1/0.
- Tier 0: PASS within the recorded baseline. UAR and proxy checks exit 0.
  Scoped Clippy exits 0 with 572 existing warnings; this is not a warning-free
  result.
- Tier timing: full phase Tier 2 remains deferred until all changes in the
  active phase are complete.

Independent artifact critic: PASS. Independent artifact judge: PASS. All four
blocking constraints are satisfied for this change boundary.
