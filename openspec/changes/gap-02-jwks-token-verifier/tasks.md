## 0. Read first

- [ ] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`
      before starting. Execution order across this phase is load-bearing.

## 1. TokenVerifier abstraction

- [ ] 1.1 Add `src/uar/security/verifier/mod.rs` defining `trait TokenVerifier`,
      `enum Presented { Jwks(String) }` with `SdJwtVp` / `DidAuth` reserved and
      documented as PID P4 (do not implement them), and `struct Principal`.
- [ ] 1.2 Implement the existing HS256 shared-secret path behind the trait with
      no behavioural change. Prove it by running the current middleware tests
      unchanged — they must pass before any JWKS code is added.

## 2. JWKS lane

- [ ] 2.1 Add `jwks_url`, `jwt_issuer`, `jwt_audience` to `SecurityConfig`
      (`src/config.rs`), all optional, following the existing
      `#[serde(default)]` pattern in that file.
- [ ] 2.2 Implement JWKS fetch with the existing `reqwest` client. Use
      `jsonwebtoken::jwk::JwkSet` and `DecodingKey::from_jwk`. **Add no new
      dependency** — if one appears necessary, stop (see 5.2).
- [ ] 2.3 Cache in `RwLock<HashMap<String, DecodingKey>>` with a refresh
      timestamp. Refresh at most once per unknown `kid` per request; a `kid`
      still absent after one refresh is a 401.
- [ ] 2.4 Enforce `iss` and `aud` via `Validation` when configured.

## 3. Enforce jwt_required

- [ ] 3.1 Replace the hardcoded `false` at `src/uar/security/middleware.rs:85`
      with `state.config.security.jwt_required`.
- [ ] 3.2 Run the `uar-sidecar` tests. They are expected to **pass**: the sidecar
      sets `jwt_required` explicitly (see `fix-sidecar-loopback-auth`, 5/6 done),
      and this change makes that setting effective. **If a sidecar test fails,
      stop and report — do not revert 3.1 and do not edit the sidecar.**

## 4. Proof

- [ ] 4.1 Unit tests: JWKS-signed token accepted; unknown `kid` refreshes once
      then 401; HS256 lane unchanged.
- [ ] 4.2 Unit tests: `jwt_required=true` rejects both absent and invalid tokens
      with 401; `jwt_required=false` still permits anonymous.
- [ ] 4.3 Unit tests: correct signature with wrong `aud` → 401; wrong `iss` → 401.
- [ ] 4.4 Fail-closed test: JWKS unreachable, no cached keys, `jwt_required=true`
      → 401.
- [ ] 4.5 **Negative control for 4.4.** Demonstrate the test fails when the
      closed branch is inverted. Record the command and its failing output in
      the verification record. An untested fail-closed assertion proves nothing.

## 5. Stop conditions

- [ ] 5.1 A task appears to require editing `docs/SPECIFICATION.md` → stop.
- [ ] 5.2 A new crate dependency appears necessary → stop and report; the
      analysis concluded none is needed and that conclusion is falsifiable.
- [ ] 5.3 A pre-existing failure unrelated to this change appears → stop and
      report rather than repairing it inside this change.
- [ ] 5.4 Closing this gap appears to require a tenant concept → stop. That is
      GAP-03's scope and it depends on this change, not the reverse.
