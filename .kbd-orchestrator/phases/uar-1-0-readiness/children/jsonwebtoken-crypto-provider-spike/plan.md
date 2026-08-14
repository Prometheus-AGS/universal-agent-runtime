PLAN: uar-1-0-readiness/jsonwebtoken-crypto-provider-spike
Project: universal-agent-runtime
Date: 2026-08-13
OpenSpec available: YES — reuse existing `fix-jwt-crypto-provider`; create no child change
Changes to implement: 1

## Change list

1. `handoff-jwt-provider-decision`: Close the contained spike and return one binding provider decision to parent A0.
   - Scope: child KBD artifacts only
   - Depends on: completed assess and analyze handoffs
   - Library: `cand-001`
   - Recommended agent: Codex
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: HIGH — unblocks authentication repair without guessing at the crypto backend
   - Details: Preserve `decision.md`, `analysis.md`, `research-evidence.md`, `unresolved-review-findings.md`, and the concrete receipts under `review/{assess,analyze,decision,plan}/`. Write `handoff-out.md` naming the manifest entry, commands below, negative control, exactly-one-provider assertion, stop condition for any new package, risks/rejected alternatives/re-evaluation triggers, and the `server-full` reporting limit. Do not edit Cargo, source, tests, or the existing OpenSpec change from this child.

## Execution order

Round 1: `handoff-jwt-provider-decision`

## Acceptance criteria

- `handoff-out.md` names `jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }` as binding for parent A0.
- It states that Cargo's intended caret semantics permit compatible 11.x releases while `Cargo.lock` pins the observed build to 11.0.0; future lock/version updates must repeat the provider/advisory checks.
- It records that baseline/AWS-LC resolve 918 active normal/build packages and RustCrypto resolves 940, with the exact evidence path.
- It does not claim performance, FIPS, cross-target UAR, non-`server-full`, or runtime-wide correctness.
- It lists the exact A0 commands in the next section.
- It states that any new package, both provider features active, or an unrelated pre-existing failure triggers the parent contract's stop-and-report behavior.
- It carries the rationale, rejected RustCrypto/manual-provider alternatives, native/FFI and profile-scope risks, and the explicit re-evaluation triggers from `decision.md`: provider feature unification, a new package, a relevant unpatched advisory or version update, loss of AWS-LC graph/platform support, a real no-std/WASM/C-free/HSM/KMS/FIPS requirement, or a patched RustCrypto RSA release.
- It explicitly instructs parent A0 to add `test_resolve_user_context_rejects_token_signed_with_wrong_secret` before running the focused suite and requires rejection as an error, never a panic.
- It defines the negative-control precondition and two outcomes: with the provider absent before A0 (or removed in a scratch checkout), the exact valid-token test must fail with the missing-provider panic; after A0 enables AWS-LC, the identical test must pass.
- The child exits to `uar-1-0-readiness` with exact next work `/kbd-execute uar-1-0-readiness`, beginning A0.

## Commands to run

No `/opsx:new` command. Parent change already exists:

```text
openspec/changes/fix-jwt-crypto-provider
```

Parent A0 commands to preserve verbatim in `handoff-out.md`:

```bash
git diff -- Cargo.toml Cargo.lock
cargo check --locked --no-default-features --features server-full
cargo clippy -p universal-agent-runtime
cargo tree --locked --no-default-features --features server-full \
  -e features -i jsonwebtoken@11.0.0
cargo test --locked --no-default-features --features server-full \
  --lib test_resolve_user_context -- --nocapture
openspec validate fix-jwt-crypto-provider --strict
```

Provider-disabled scratch negative control:

```bash
cargo test --locked --no-default-features --features server-full \
  --lib uar::security::middleware::tests::test_resolve_user_context_valid_token \
  -- --exact --nocapture
```

Run the negative-control command against the unmodified provider-less baseline before applying A0, or in a scratch checkout where A0's `aws_lc_rs` feature is removed. It must fail with the observed `Could not automatically determine the process-level CryptoProvider` panic. After A0 enables AWS-LC, the identical test must pass. A passing provider-disabled control is invalid evidence.

Parent A0 must add `test_resolve_user_context_rejects_token_signed_with_wrong_secret` before the focused suite; the test must return an authentication error and must not panic. The tree output must show `aws_lc_rs` active and `rust_crypto` absent.

The clippy command intentionally remains `cargo clippy -p universal-agent-runtime`, verbatim from the readiness execution contract and user instruction. It is package-scoped because `--all-targets` is blocked by vendored-submodule pedantic errors; this child does not silently replace the contract command with a different feature invocation.

## Trade-off and scope cut

The choice keeps native C/assembly/FFI already present in `server-full`. The child deliberately does not certify embedded/mobile or cross-target builds and does not add CI, because those claims and files are outside both the execution contract and child scope.

PLAN COMPLETE
