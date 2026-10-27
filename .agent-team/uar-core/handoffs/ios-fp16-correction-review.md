# iOS FP16 correction — independent source review

2026-10-25. Active phase: `uar-post-lifecycle-verification-readiness`, task 2.9. Source inspection authorized for the S8 correction under A15; full product verification gate remains CLOSED. No author reasoning or generation history was supplied. Other writers share this checkout; this reviewer changed only this handoff.

## Artifact echoed before verdict

The cached-original versus vendor recursive source diff contains exactly four hunks in `vendor/patches/gemm-common-0.19.0/src/simd.rs`. Each adds:

```rust
/// # Safety
/// The caller must establish that the CPU supports FP16 instructions.
#[target_feature(enable = "fp16")]
```

These precede `vmulq_f16` at line 1953, `vaddq_f16` at 1971, `vfmaq_f16` at 1988, and `vfmaq_laneq_f16` at 2003. Existing assembly bodies and inline attributes are unchanged. The last hunk also adds an empty documentation separator.

Root `Cargo.toml` adds the workspace exclusion and:

```toml
gemm-common = { path = "vendor/patches/gemm-common-0.19.0" }
```

Root `Cargo.lock` retains `gemm-common` version `0.19.0` and its dependency list, removing only that package's registry source/checksum. Other existing root manifest/lock changes belong to the shared checkout and are outside this correction review.

## Verdict

PASS for source acceptability. No actionable defects found in the reviewed correction. This verdict does not establish compilation, runtime behavior, performance, or device coverage.

All four helpers remain unsafe and AArch64-only. Their new per-function instruction requirements match the already annotated scalar FP16 helpers. No global target-feature or CPU baseline change appears in this correction.

Callers were traced in both sources. The common crate calls `vfmaq_f16`, `vmulq_f16`, and `vaddq_f16` through `MixedSimd<f16, f16, f16, f16> for NeonFp16`; its private token is constructed only after the existing NEON and FP16 feature checks. `vfmaq_laneq_f16` has no common-crate caller. The cached f16 crate's `mul`, `add`, `mul_add`, and `mul_add_lane` wrappers are unsafe; its microkernel invocations request `neon,fp16`, and the common `microkernel!` macro applies that target feature to generated entries. The f16 dispatch retains `fp16` detection, the NEON fallback, and the scalar fallback. Its FP16 `gemm_basic` additionally constructs the checked `NeonFp16` token. No new call path bypasses those existing guards.

The normalized vendor manifest remains edition 2021, package/version `gemm-common`/`0.19.0`, with unchanged features/dependencies. Root patch placement and workspace exclusion follow existing repository patch conventions. The recursive comparison found no extra or missing files relative to the cached original. Copied registry metadata (`.cargo-ok`, `.cargo_vcs_info.json`), nested lockfile, original manifest, README and MIT license remain identical; comparable existing lifecycle patches also retain this metadata. The VCS metadata names upstream source revision `86102c5b712737978371ac9ef7a11982f686d7bc`, not a claim that the patched tree is pristine.

## Inspection and limits

Source operations only: `cat`, `sed`, `rg`, `git diff --staged --stat`, scoped `git diff`, `git diff --name-only`, scoped `git status --short`, and `diff -ru` against the supplied cached original. Recursive diff output consisted solely of the four helper hunks. No tests, Cargo, lint, formatting, compiler probes, builds, packaging checks, or verification helpers ran.

Files inspected: authoritative current waypoint; root `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, relevant `versions.toml` search; vendor `Cargo.toml`, `src/simd.rs` (full changed helpers and relevant token/trait blocks), `src/lib.rs`, `src/microkernel.rs` (full `microkernel!` block), metadata/license and complete vendor file inventory; cached original gemm-common directory via recursive diff; cached `gemm-f16-0.19.0/Cargo.toml`, `src/microkernel.rs` FP16 module, and `src/gemm.rs` dispatch/FP16 entry. Consulted Rust workspace, Rust best practices/documentation, and review verification protocol skills. Repository gotchas were consulted; a large read was truncated and no claim depends on unseen output.

Only this review document was added. No production code, guards, dependency pins, or unrelated files were changed by this reviewer. Compile resolution, LLVM assembler acceptance on the failing iOS toolchain, installed product behavior, and non-FP16 hardware fallback execution remain unverified pending the lead's exact authorized gate.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE — source-only PASS; runtime/device acceptance remains unverified.
