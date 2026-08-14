# Artifact-refiner verification summary — `fix-jwt-crypto-provider`

- Schema inputs: manifest and constraints are present for `direct:content` validation.
- Files: this receipt exists and is non-empty.
- Dependency constraint: the locked reverse feature tree shows RustCrypto and no AWS-LC feature for `jsonwebtoken 11.0.0`.
- Runtime constraint: the final security slice passed 25 tests and the proxy passed 2 tests.
- Fail-closed controls: AWS-LC-first and RustCrypto-first both returned the
  structured provider conflict; deliberately false foreign-provider acceptance
  and wrong-secret acceptance assertions failed as required.
- Profile constraints: final server-full Tier 0 passed; iOS and Android embedded-mobile checks passed separately.
- Provider ownership: the operator selected UAR-owned first installation after
  review proved that the public v11 API cannot identify an earlier provider.
  The shared server-startup funnel acquires RustCrypto before routes/readiness.
- Spec constraint: the amended first-owner OpenSpec requirement passes strict
  validation.
- Tier boundary: phase-level Tier 2 was not run.

Status: **PASS.** `jsonwebtoken` 11 keeps the
installed process provider behind a crate-private getter. UAR therefore claims
first ownership rather than provider identity: any earlier initialization,
including RustCrypto, fails closed. Literal Android, provider-disabled,
prior-provider, and wrong-secret commands and outputs are retained under
`openspec/changes/fix-jwt-crypto-provider/evidence/`.

Independent artifact-validator: all four blocking constraints pass.
Independent artifact-critic: PASS with no findings.

Detailed commands and observed outputs are in
`openspec/changes/fix-jwt-crypto-provider/verification.md`.
