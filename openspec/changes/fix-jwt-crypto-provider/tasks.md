## 0. Read first

- [x] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [x] 0.2 This change runs FIRST, before `gap-02-jwks-token-verifier`.

## 1. Standardize the workspace provider

- [x] 1.1 Define `jsonwebtoken = { version = "=11.0.0", default-features = false, features = ["rust_crypto"] }` under `[workspace.dependencies]`; make the runtime and `uar-jwt-proxy` inherit it.
- [x] 1.2 Regenerate `Cargo.lock` and prove the workspace graph activates `rust_crypto` but not `aws_lc_rs` for `jsonwebtoken` 11.0.0.

## 2. Fail closed at the process boundary

- [x] 2.1 Add a crate-private runtime wrapper that explicitly installs `rust_crypto::DEFAULT_PROVIDER` at the shared server-startup funnel and before every encode/decode. Cache UAR's successful first installation for idempotent reuse; return a structured error if any provider was initialized before UAR.
- [x] 2.2 Route middleware verification and API-key JWT issuance through the wrapper. Provider conflict maps to HTTP 500 in middleware and a contextual service error in API-key exchange.
- [x] 2.3 Initialize RustCrypto in `uar-jwt-proxy` before its first token is minted; provider conflict exits startup with an error.

## 3. Prove execution and negative controls

- [x] 3.1 Test idempotent UAR-owned RustCrypto initialization, HS256 round-trip through the runtime path, wrong-secret rejection, API-key exchange, and proxy token minting.
- [x] 3.2 Record the provider-disabled round-trip failure and the pre-fix workspace tree with both providers active.
- [x] 3.3 In isolated scratch processes, preinstall AWS-LC and RustCrypto separately and demonstrate that the UAR guard returns the structured conflict error for either prior owner.

## 4. Verification and handoff

- [x] 4.1 Tier 0 passes for the `server-full` profile, and `cargo check --locked -p uar-jwt-proxy` passes.
- [x] 4.2 Full `embedded-mobile` library checks pass for `aarch64-apple-ios` and `aarch64-linux-android`; report each target separately.
- [x] 4.3 `openspec validate fix-jwt-crypto-provider --strict` and the artifact-refiner validation gate pass.
- [x] 4.4 Write `verification.md`, transition A0 complete through canonical KBD state, and leave A1 pending as the exact next change.

## 5. Stop conditions

- [ ] 5.1 The change requires a new direct crate or a new package not already locked by the workspace → stop and report.
- [ ] 5.2 A UAR JWT encode/decode call cannot be routed through the guard within the permitted surface → stop and report.
- [ ] 5.3 A release-target check or focused test exposes a pre-existing unrelated failure → stop and report; do not repair it in A0.
