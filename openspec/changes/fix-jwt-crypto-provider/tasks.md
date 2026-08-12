## 0. Read first

- [ ] 0.1 Read `.kbd-orchestrator/phases/uar-1-0-readiness/EXECUTION-CONTRACT.md`.
- [ ] 0.2 This change runs FIRST, before `gap-02-jwks-token-verifier`.

## 1. Enable a provider

- [ ] 1.1 `Cargo.toml:393` → `jsonwebtoken = { version = "11.0.0", features = ["rust_crypto"] }`.
      Prefer `rust_crypto`: its deps already resolve in the tree, so no new
      transitive crates are added. If `aws_lc_rs` is chosen instead, say why in
      the verification record.
- [ ] 1.2 `cargo check --locked --no-default-features --features server-full` — clean.

## 2. Prove it executes

- [ ] 2.1 Test: sign a token, verify it through the runtime's verification path,
      assert the subject round-trips. **A compile check does not count** — the
      defect is a runtime panic in a function pointer.
- [ ] 2.2 Test: a token signed with a different secret is rejected as an error,
      not a panic.
- [ ] 2.3 **Negative control.** Revert 1.1 in a scratch build and show 2.1 fails.
      Record the command and its failing output.

## 3. Unblock A1

- [ ] 3.1 Run the pre-existing HS256 middleware tests unchanged. They must pass.
      This is `gap-02` task 1.2's precondition — met, not waived.

## 4. Stop conditions

- [ ] 4.1 The fix appears to require source changes beyond `Cargo.toml` and
      tests → stop and report. It should not.
- [ ] 4.2 Enabling the feature pulls in NEW transitive crates → stop and report
      which; the analysis predicted none.
- [ ] 4.3 A pre-existing unrelated failure appears → stop and report.
