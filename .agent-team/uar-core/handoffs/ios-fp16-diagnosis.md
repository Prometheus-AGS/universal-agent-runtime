# S8 iOS FP16 source diagnosis — A15 bounded diagnostic

Phase: `uar-post-lifecycle-verification-readiness`, task 2.9, waypoint revision 517. Full-product verification gate CLOSED. This is source-only diagnosis of the observed S8 failure, not the scheduled RUST review. No Cargo, compiler, test, lint, format, build, packaging, or other verification command ran. Others share the checkout; only this handoff was written.

## Recommendation

A bounded same-version correction exists as a strong source-supported candidate: vendor the cached `gemm-common` **0.19.0** unchanged except adding `#[target_feature(enable = "fp16")]` to the four AArch64 vector assembly helpers in `src/simd.rs`: `vmulq_f16` (1949), `vaddq_f16` (1963), `vfmaq_f16` (1976), and `vfmaq_laneq_f16` (1987). Retain their existing `#[inline]` and unsafe signatures. Add explicit safety documentation that callers must establish FP16 support. Use the existing root `[patch.crates-io]` mechanism for that exact version and update the lockfile's source selection through an authorized later process. Preserve version 0.18.2 separately. Do not set global `+fp16`, alter CPU/deployment minimums, upgrade dependencies, or modify the Cargo registry.

This is a candidate correction, not a demonstrated successful build. Parent owns authorization, source ownership, spec/ledger recording, implementation, and any allowed rerun.

## Observed cause

Evidence: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-1/stderr.log:486–619` reports eleven `instruction requires: fullfp16` failures, all at the four vector helper bodies, including all eight lane cases. The final failure names `gemm-f16` compilation. Cached source at `/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gemm-common-0.19.0/src/simd.rs` confirms these helpers lack a target-feature attribute. Immediately preceding scalar helpers (`add_f16_fp16`, `fmaq_f16`, `multiply_f16_fp16`, lines 1906–1941) already have `#[target_feature(enable = "fp16")]`. The proposed patch makes vector helpers express the same precondition at their own code-generation boundary.

## Baseline-preservation argument from inspected source

- `gemm-f16-0.19.0/src/gemm.rs:800–832`: `init_gemm_fn` selects `neonfp16::gemm_basic` only after `feature_detected!("neon")` and `feature_detected!("fp16")`; otherwise the existing `neon::gemm_basic` or scalar path remains.
- `gemm-common-0.19.0/src/lib.rs:75–90`: with `std`, the AArch64 detection macro delegates to `std::arch::is_aarch64_feature_detected!`. Without `std`, the macro uses `cfg!(feature = ...)`; the cached manifest does not define a feature named `fp16`, so that branch does not positively assert FP16 on its own.
- `gemm-common-0.19.0/src/simd.rs:2051–2054`: `NeonFp16` has a private field, preventing ordinary external construction. Its two `MixedSimd::try_new` implementations (2091 and 2193) require both features before returning the token. Its `Simd::vectorize` (2069–2075) already enables `neon,fp16`.
- `gemm-f16-0.19.0/src/microkernel.rs:15–97`: direct vector-helper calls are inside the `neonfp16` module. Its generated microkernels explicitly request `neon,fp16`. `gemm-common-0.19.0/src/microkernel.rs:169` and related macro definitions apply requested target-feature attributes to generated kernels.
- Additional direct uses found in cached gemm 0.19 family source are the token-bearing `MixedSimd` methods in `gemm-common/src/simd.rs:2232,2278,2283`. No other direct helper calls were found among the inspected gemm-common/f16/f32/f64/c32/c64 0.19 sources.

Thus the patch enables FP16 only on functions whose bodies already unconditionally execute FP16 instructions; it does not change the selection predicates or promote baseline functions to FP16. Existing unsafe public callers outside the inspected gemm family remain responsible for the instruction precondition, just as they already are when directly executing these helper bodies. This diagnosis does not claim an exhaustive audit of every crate in the dependency graph.

## Exact prospective implementation scope

1. New checked-in source directory under the existing `vendor/crates/` convention containing cached gemm-common 0.19.0 source and license; four function attributes plus their safety comments are the only behavioral source delta.
2. Root `Cargo.toml` existing `[patch.crates-io]` entry selecting this vendored version; workspace exclusion if required by established local vendoring conventions.
3. `Cargo.lock` source selection for gemm-common 0.19.0, with version 0.18.2 retained; any separately active consumer workspace and lockfile need explicit source inspection before assuming the root patch applies.
4. A short patch provenance record describing observed eleven diagnostics, source base, four-function scope, and CPU-baseline rationale; required phase/spec records remain lead-owned.

Root `Cargo.toml:32` already patches esaxx and SurrealDB. Root lock contains both gemm 0.18.2 and 0.19.0; Candle 0.9.2 and 0.11.0 reference gemm 0.19.0. Root `.cargo/config.toml` has no target-feature addition. `versions.toml` search yielded no gemm/Candle override.

## Limits and uncomfortable scenario

The correction may expose a later independent S8 failure after these eleven errors are removed. Function-local attributes can also change inlining/performance. Neither success nor performance nor old-device runtime behavior is verified. No hardware capability has been inferred from the deployment target. Global `+fp16` is rejected because it would alter what the compilation baseline promises, rather than retaining runtime selection.

Only source reads (`sed`, `rg`, `cat`) and this documentation write were performed. One `rg` search included absent `apps/` and reported that missing path; no matching production helper use was printed from the existing `src`/`crates` paths. No source, registry, manifest, lockfile, config, or upstream resource was changed.

External issue URLs supplied by the parent are leads only, not authority for this conclusion: https://github.com/zed-industries/zed/issues/57770 ; https://github.com/huggingface/candle/issues/494 ; https://github.com/huggingface/candle/issues/1625 ; https://github.com/huggingface/candle/issues/2823 . The diagnosis relies on the exact cached 0.19.0 source and actual S8 diagnostic above; it makes no independent claim about the contents of those unread issues.
