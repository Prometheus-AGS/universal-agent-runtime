# Convergence Decisions

## 2026-08-14 — Accept iteration 1

All A0 blocking constraints pass with deterministic evidence. The first
provider-conflict control exposed an incorrect interpretation of
`OnceLock::set`: its error returns the attempted provider, not the installed
provider. The corrected wrapper records only a successful UAR-owned RustCrypto
installation and compares that recorded pointer on repeated calls; a prior
foreign or panic-provider initialization returns `ProviderConflict`.

The Android check initially exposed missing NDK/OpenSSL environment setup. A
temporary vendored Android OpenSSL sysroot allowed the unchanged full
`embedded-mobile` graph to pass, so no unrelated dependency repair entered A0.
Proceed to the KBD/OpenSpec completion handoff without running Tier 2.

## 2026-08-14 — Supersede iteration 1 provider-identity claim

Independent review invalidated iteration 1's pointer-comparison rationale:
`install_default()` returns the attempted provider, not the installed one, and
the installed provider getter is private. The operator selected the smaller
safe contract: UAR owns first installation. Any process provider installed
before UAR—including RustCrypto—returns a structured conflict. Iteration 2
implements that boundary at the shared server-startup funnel and guarded JWT
operations.

## 2026-08-14 — Terminate iteration 2

The independent artifact-validator confirmed all four blocking constraints,
and the isolated artifact-critic returned PASS with no findings. Persist
iteration 2 and terminate refinement as converged. Tier 2 remains deferred to
completion of all six phase changes.
