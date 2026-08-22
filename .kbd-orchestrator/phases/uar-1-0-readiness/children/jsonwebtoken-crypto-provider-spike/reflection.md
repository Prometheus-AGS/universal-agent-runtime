# Reflection — jsonwebtoken crypto provider spike

Date: 2026-08-13
Scope: `server-full` on the current aarch64 macOS host; no result transfers to another profile or target.

## Delta between plan and delivery

The parent A0 proposal preferred RustCrypto because its packages appeared in `Cargo.lock`. The spike disproved the load-bearing inference: lockfile presence did not mean active build-graph presence. Measured feature simulation kept AWS-LC at the 918-package baseline and raised RustCrypto to 940 active normal/build packages. The delivered decision therefore selects AWS-LC, not the proposal's preliminary RustCrypto preference.

Research also began with the RustCrypto RSA timing advisory as a prominent discriminator. Independent review forced a narrower claim: repository search found no current RS/PS private-key signing path, so the advisory is secondary. The decisive evidence is the zero-versus-22 active-package delta, identical required algorithm coverage, and upstream support for the native targets.

The plan initially under-specified closure. Two isolated plan reviews blocked it because re-evaluation triggers, exact commands, receipt paths, and negative-control semantics were not executable acceptance criteria. Those defects were corrected after the two-round cap and disclosed rather than represented as independently re-vetted.

## Goal results

- Compare RustCrypto and AWS-LC using current official/source, graph, platform, security, and measured evidence: MET. Evidence is in `analysis.md` and `research-evidence.md`. Performance remains explicitly unknown and unused.
- Record one binding decision with rationale, rejected alternative, risks, and re-evaluation triggers: MET. `decision.md` binds `jsonwebtoken` 11 to `aws_lc_rs`; `handoff-out.md` carries it to A0.
- Return the exact parent configuration and verification commands without implementing inside the child: MET. `handoff-out.md` contains the manifest entry, Tier 0/focused Tier 1/exclusivity/strict-validation commands, wrong-secret prerequisite, and provider-disabled negative-control semantics. The implementation-surface diff is empty.

## Root causes

1. The proposal treated resolver inventory (`Cargo.lock`) as build activation. Cargo feature decisions require an active-graph comparison, not a lockfile search.
2. The initial assessment and plan used category labels such as “measured evidence” and “negative control” before defining executable commands and pass/fail semantics.
3. KBD's nested-child helper compares a basename with a fully qualified runtime phase ID, causing a false canonical-state block. Typed runtime commands remained authoritative, but the helper defect added manual coordination work.
4. The first task transition attempted pending → complete; canonical runtime correctly rejected it. The ledger was repaired through pending → in-progress → complete.

## Corrective actions carried forward

- Parent A0 uses `jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }` and stops if an actual new package appears.
- Parent A0 asserts exactly one provider through the feature tree; future workspace feature changes repeat that assertion.
- Negative controls always state the precondition, expected failure, and same-command post-fix pass.
- Security advisories are tied to observed code paths; unexercised provider capability is labeled conservative/secondary.
- KBD child work records helper/runtime inconsistencies instead of following an incorrect remediation that would activate the wrong phase.

## Artifact quality

- Manual validation: PASS, recorded in `qa-validation.md`; the KBD integration explicitly permits manual checks for trivial low-risk artifacts.
- Cross-model assessment review: PASS with warnings corrected.
- Cross-model analyze and decision reviews: two PASS rounds; warning resolutions and scope cuts recorded.
- Cross-model plan review: BLOCK in both permitted rounds; all critical criteria were corrected after the cap and remain explicitly un-re-vetted in `unresolved-review-findings.md`.
- No implementation QA claim is made because the child changed no implementation surface.

## Remaining risks

- The chosen manifest feature is unconditional. This phase does not certify embedded/mobile, cross-target UAR builds, or any non-`server-full` graph.
- AWS-LC retains native C/assembly/FFI and requires a C/C++ compiler for ordinary non-FIPS builds.
- Cargo feature unification can reintroduce the both-provider panic if another workspace member enables `rust_crypto` for `jsonwebtoken 11`.
- The plan's final corrections were not independently reviewed after the two-round cap; parent A0 must treat `handoff-out.md` commands as requirements and record observed output.

## Next work

Return to `uar-1-0-readiness` and execute A0 `fix-jwt-crypto-provider` first. Do not start A1 until A0's round trip, wrong-secret rejection, provider-disabled negative control, unchanged middleware tests, exactly-one-provider tree check, Tier 0 commands, strict OpenSpec validation, verification record, and commit all succeed.
