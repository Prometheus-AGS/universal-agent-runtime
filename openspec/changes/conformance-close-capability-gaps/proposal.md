# conformance-close-capability-gaps

Phase: `uar-spec-conformance-2026-08` (change C-03 + C-04)

## Why

`docs/SPECIFICATION.md` declares 27 capabilities. The matrix covers 19.
**Eight have no test at all: C-16, C-18, C-19, C-21, C-24, C-25, C-26, C-27.**

That is not a random tail. It is the newest and most security-sensitive work:

- **C-21 is tenant isolation** — a security property with zero coverage.
- **C-25, C-26, C-27** are node DID, DID resolution / VC verification, and the
  credential wallet. The `frf-did` and `frf-wallet` crates were built 2026-08-07
  with 37 unit tests between them and have **never been exercised through the
  runtime**.
- **C-24** is the peer mesh.

Separately, the evidence labels overstate what was exercised. Thirteen cases
carry `l3_`, but against a stub whose fixtures the test author wrote, "did my
code parse my own canned output" is L2-wired, not L3-exercised. And
`shape_only_` and `absent_` are undefined relative to the L-scale, so the claim
that results are "honestly labelled" cannot currently be verified.

**Coverage is not conformance.** An exit criterion of "27 of 27 have a case" is
satisfied by 27 tests that assert nothing meaningful. Adversarial review
(MiniMax-M3) raised exactly this as CRITICAL, and the judge (Kimi k3) upheld it:
a route-not-found discriminator proves routing, which is L1, and for
security-sensitive capabilities an L1 result is barely better than no test.

## What Changes

- Define the evidence-label taxonomy completely, then relabel existing cases to
  match what they actually exercise.
- Add cases for the eight uncovered capabilities, each meeting a **stated target
  evidence level** rather than merely existing.
- Publish an exclusion, with its reason named, for any capability that cannot be
  exercised by this harness.

## Impact

- Affected specs: `spec-conformance-measurement`
- Affected code: `tests/integration/live/capability_cases.rs` (extend),
  possibly `tests/integration/live/harness.rs` (multi-tenant boot for C-21)
- Depends on: `conformance-baseline-gate` — do not extend a miscalibrated
  instrument
- Risk: medium. C-21 may require harness changes to boot two tenants; if that
  proves structural, C-21 becomes a published exclusion rather than a weak pass.
