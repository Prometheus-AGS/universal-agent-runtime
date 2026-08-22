# Decision log — jsonwebtoken crypto provider spike

## 2026-08-13 — JWT-CRYPTO-001

- Status: binding for parent A0, subject only to the falsifiers in `decision.md`.
- Decision: retain `jsonwebtoken` 11.0.0 and enable only `aws_lc_rs` for `server-full`.
- Adopted candidate: `cand-001`.
- Rejected candidates: `cand-002` (`rust_crypto`) and `cand-003` (manual/custom provider).
- Rationale: same JWT algorithm coverage and all release targets supported; measured feature simulation keeps AWS-LC at the 918-package baseline while RustCrypto resolves 940 active normal/build packages. The locked AWS-LC version is beyond inspected patched advisory ranges. RustCrypto RSA's unresolved timing advisory is secondary because no current RS/PS signing path was observed. No performance or FIPS claim was used.
- Parent configuration: `jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }`.
- Evidence: `analysis.md`, `library-candidates.json`, observed focused baseline output recorded in `assessment.md`.
- Uncomfortable consequence: standardization retains native C/assembly and FFI in the authentication dependency surface. This is accepted because that surface already exists under `server-full`; it is not evidence that AWS-LC is universally safer or preferable outside this profile.

## 2026-08-13 — JWT-CRYPTO-002 supersedes JWT-CRYPTO-001

- Status: binding for parent A0.
- Decision: pin `jsonwebtoken` exactly at 11.0.0 and enable only `rust_crypto` across UAR-owned workspace packages.
- Supersedes: JWT-CRYPTO-001 (`aws_lc_rs`).
- New evidence: a workspace feature tree observed the root AWS-LC feature and proxy RustCrypto feature unified on the same package, which selects the crate's panic provider. Isolated RustCrypto checks completed for iOS and Android.
- Safety mechanism: central workspace dependency plus UAR-owned first RustCrypto installation at shared server startup and before JWT operations. UAR caches only its own successful installation; any earlier provider, including RustCrypto, is a structured conflict because the public v11 API exposes no installed-provider identity.
- Rationale and falsifiers: `re-evaluation-2026-08-13.md`.
- Uncomfortable consequence: RustCrypto activates more pure-Rust packages in `server-full` and retains an unpatched RSA private-operation advisory. The advisory is not exercised by current HS256 signing or planned public-key verification, and private RSA/PS signing remains outside this phase.
