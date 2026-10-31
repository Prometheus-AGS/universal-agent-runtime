# #462 phase-exit receipt — incomplete

Source base: 964e816854411cd12b3c85e5e0f11f03b0bf4892 plus recorded implementation inventory. No committed-candidate Rust PASS and no release artifact produced.

| Evidence | Result | Meaning |
|---|---|---|
| `python3 /tmp/uar476-safeguard-controls.py` | exit 0; 5/5 controls pass | Synthetic fake Cargo/rustc: profile sequence, dirty refusal, command failure exit42, warning refusal, changed-source refusal. No Rust compilation. Script and full outputs retained in evidence. |
| `PATH=/usr/local/bin:$PATH python3 /tmp/uar476-hook-controls.py` | exit 1; 6 positive selectors passed, README negative oracle initially failed | Installed Lefthook 2.2.2 ran 3 fake compiles for root/nested Rust/manifest/toolchain/config/script selections. README made no Cargo call; forced `--command` returns exit1 when no job matches. Initial test expected exit0 incorrectly. |
| `PATH=/usr/local/bin:$PATH python3 /tmp/uar476-hook-readme-control.py` | exit 0; corrected README negative oracle passed | Same source/hook; require exit1 plus `no matching staged files` and zero Cargo calls. Only affected control rerun. Initial receipt retained. |
| Artifact-only Rust review | no actionable findings | One duplicate-field deletion; four history/provenance/tenant assertions preserved. Source review, not compile/runtime evidence. |
| Artifact-only script/hook review | no actionable findings | Source review only; no runtime/cross-family certification. |
| `PATH=/usr/local/bin:$PATH pnpm audit --prod --audit-level=high` | exit 1; 21 findings (1 critical,9 high,4 moderate,7 low) | Required pre-commit audit. Existing dependency findings triaged separately; no lockfile/pin edits. |
| Three real library harness profiles | NOT RUN | Another chat's Cargo writer owns shared host build directory; ordinary hook/commit and acceptance matrix serialize behind it. |
| History, embeddings/OpenAPI, format/full affected inventory | NOT RUN | Depend on real compiler baseline; no current runtime assertions proved. |

The completed production implementation does not satisfy the phase exit yet. No test exclusions, temporary source patches, warning suppression or historical pass relabeling were used. No build artifact or shipped lane is qualified.

## Additional affected gates

- `openspec validate restore-exact-tree-rust-baseline`: exit0, output `Change 'restore-exact-tree-rust-baseline' is valid`. Structural spec acceptance only.
- `cargo fmt --all -- --check > /tmp/uar476-format.log 2>&1`: exit1, 83,833 output lines with existing differences including build.rs, acceptance fixture code, and vendor/patches/surrealdb-lifecycle. Check-only; no formatting writes. Full raw log retained at target/production-readiness-476/format.log (ignored output); SHA256 15a460b20ce7a6e208c2168e9231963c04ce8aabdcdd0d8aaf0db15d71fd6c73. No repository-wide reformat or claim of passing affected checks.
- These independent metadata/format validators overlapped briefly; neither ran Rust compilation nor wrote a build target. All real compile gates remain unstarted pending single-writer scheduling.

## Liter policy alignment follow-up

Operator selected “Retain current Liter source.” versions.toml now pins0d74c56659943e3fa44a3c8e6d51a27514f3b35c, retaining version2.3.0. Independent artifact-only review found zero findings: policy, HEAD gitlink and materialized dependency HEAD agree; dependency source and Cargo.lock diffs are empty. Report: .agent-team/uar-core/production-readiness-liter-pin-review.md. This resolves the policy discrepancy and adds no runtime acceptance. Implementation-complete inventory renewed as implementation-inventory-002.json.

No new tests/builds were run for the unchanged dependency source. Real #462 acceptance still waits for the existing original-checkout Cargo writer; no original process or shared build configuration was modified.

## Clean committed baseline a3bceb9a

The ordinary hook matrix passed warning-free on attempt003 and produced a3bceb9a412d86f23c4ad30dee450872b34e36bd. `bash scripts/verify-rust-test-baseline.sh --acceptance` then passed all3profiles on that clean unchanged commit/tree (ce569458f0e37e529dc08c74149dd699752043a5), including recursively clean pinned submodules. Retained regressions:4history,11embeddings,1OpenAPI passed; zero failed/ignored in those selected assertions. Full logs are under target/production-readiness-476/{exact-commit-baseline-001,history-001,embeddings-001,openapi-001}.log.

The subsequent full server-full inventory failed compilation exit101 before tests: obsolete integration fixture constructors and shared helper warnings. Raw diagnostics retained in evidence/full-inventory-attempt-001.txt. These are not labelled test assertion failures. A single fixture-compatibility repair cohort was added to the same phase; those earlier passes are source-bound history, not certification of the repaired tree.

Exact build environment for these checks: CARGO_BUILD_BUILD_DIR=/Users/gqadonis/.claude/worktrees/production-readiness-476/target/build; CARGO_CACHE_RUSTC_INFO=0; RUSTC_WRAPPER empty; PATH starts /usr/local/bin. The broad inventory also uses RUN_LLM_TESTS=0. Toolchain Cargo1.99.0-nightly59800466c, rustc1.99.0-nightlyb6839f4d0, x86_64-apple-darwin, LLVM22.1.8. Host/global source/cache/config were not reset. Earlier file-open-stalled attempts001/002 were stopped as incomplete (Cargo143, hook1), with samples under ignored target receipts. Exact opened filename in sample002 was inferred from Cargo layout, not captured as an argument.

## Clean9ed71e93 baseline and second inventory disposition

Clean unchanged9ed71e93 passed the3-profile guard and16specified assertions again. Artifact/source/config/log hashes recorded at target/production-readiness-476/committed-baseline-002.receipt.json. Full inventory002 exited101 before execution, exposing later fixture API and development-feature construction mismatches; no test body/legacy qualifier ran. Raw log copied to evidence/full-inventory-attempt-002.txt. The phase plan now uses full-target compile-only inventory plus full library execution and the specified regressions, so it will not revive historical opt-in installed/recovery runners. No runtime/security behavior or assertion was weakened.
