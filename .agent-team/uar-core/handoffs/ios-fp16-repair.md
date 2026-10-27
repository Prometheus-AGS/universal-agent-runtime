# S8 iOS FP16 repair — source-only handoff

Phase: `uar-post-lifecycle-verification-readiness`, task 2.9; inspected waypoint revision 517, plan revision 141. Parent owns A15 repair012 accounting, current source identity, phase ledger/spec records, any inventory, and any gate. Full-product verification gate remains **CLOSED**. No Cargo, lock generator, compiler, test, lint, format, build, packaging, source-identity/inventory command, review gate, or external action was run. Earlier product/source010 passes are historical after these edits.

## Files and exact edits

- `Cargo.toml`: added `"vendor/patches/gemm-common-0.19.0"` to existing workspace exclusions; added `gemm-common = { path = "vendor/patches/gemm-common-0.19.0" }` under existing `[patch.crates-io]` with a CPU-baseline rationale comment. Existing patches remain in place.
- `Cargo.lock`: in the `[[package]]` whose name is `gemm-common` and version is `0.19.0`, removed only `source = "registry+https://github.com/rust-lang/crates.io-index"` and `checksum = "88027625910cc9b1085aaaa1c4bc46bb3a36aad323452b33c25b5e4e7c8e2a3e"` to select the local path. No resolver was run. The 0.18.2 package and all dependency lists, versions, and other checksums were preserved by the targeted edit.
- `vendor/patches/gemm-common-0.19.0/**`: copied the entire cached directory `/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gemm-common-0.19.0` with `shutil.copytree`, retaining original license/source/manifests/provenance files. Only `src/simd.rs` was edited within the copy: immediately before the existing `#[inline]` on `vmulq_f16`, `vaddq_f16`, `vfmaq_f16`, and `vfmaq_laneq_f16`, added `#[target_feature(enable = "fp16")]` plus a `# Safety` doc stating that callers must establish CPU FP16 support. The existing inline annotations, unsafe signatures, assembly, and dispatch are unchanged. No registry content was edited.
- `.agent-team/uar-core/handoffs/ios-fp16-repair.md`: this record.

Exact per-helper before/after insertion (four occurrences, each anchored to its named unsafe function): before, `#[inline]` directly precedes the function; after, the FP16 safety doc and `#[target_feature(enable = "fp16")]` precede the retained `#[inline]`. No runtime guard was added. The attributes address the observed eleven `instruction requires: fullfp16` diagnostics, rather than a hypothetical failure.

## Source basis and baseline preservation

Read `.agent-team/uar-core/handoffs/ios-fp16-diagnosis.md` for source-only diagnosis and references. Actual S8 evidence: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/independent-diagnostic-sweep-001/S8/attempt-1/stderr.log:486–619`. Cached `src/simd.rs` originally placed these vector helpers at lines 1949, 1963, 1976, and 1987; immediately preceding scalar helpers already enabled FP16 locally.

The analyst records `gemm-f16-0.19.0/src/gemm.rs:800–832` selecting FP16 only after both neon and fp16 detection, preserving neon/scalar fallback. Cached `gemm-common/src/lib.rs:75–90` supplies detection; `NeonFp16` construction and vectorize/microkernel contracts are detailed in the diagnosis handoff. This implementation does not edit those selection paths, global CPU flags, deployment targets, or package versions. It declares the instruction precondition on the four existing unsafe assembly bodies. These references are source reasoning, not execution evidence or a full dependency audit.

## Commands and outcome

All commands classified as source inspection or authorized implementation/documentation writing. Reads used `cat`, `sed`, and `rg`; the initial `rg --files -g AGENTS.md vendor .agent-team` was instruction discovery, not a source-identity inventory. Implementation used a Python file-copy/text-edit script; exit 0 with no output. This handoff used a shell heredoc. No verification was executed or delegated; no acceptance credit is claimed.

## Remaining limits

Compilation, lockfile acceptance under `--locked`, S8 success, old-device runtime behavior, and performance remain unverified. The repair may expose another independent S8 error; this assignment authorizes one correction only, so no second fix loop was attempted. No additional production behavior or unrelated source change was introduced. Parent must incorporate the local path into the authoritative source identity before any explicitly approved A15 rerun.
