## 1. Audit

- [x] 1.1 Inventory existing examples in `sdks/{rust,python,typescript}/examples/`
- [x] 1.2 Inventory existing doc tooling (`cargo doc`, `sdks/typescript/typedoc.json`, `sdks/python/docs/conf.py`)
- [x] 1.3 Inventory existing CI coverage of the three SDK directories

## 2. Rust cookbook (6 new examples)

- [x] 2.1 `sdks/rust/examples/embeddings.rs`
- [x] 2.2 `sdks/rust/examples/knowledge_base_crud.rs`
- [x] 2.3 `sdks/rust/examples/document_ingest.rs`
- [x] 2.4 `sdks/rust/examples/run_checkpoints_resume.rs`
- [x] 2.5 `sdks/rust/examples/list_knowledge_bases.rs`
- [x] 2.6 `sdks/rust/examples/error_handling.rs` (self-contained; targets an unreachable port; asserts on the `miette::Diagnostic`)
- [x] 2.7 `cargo build --examples --locked` passes for all 12 Rust examples

## 3. Doc tooling verification (no new tooling needed)

- [x] 3.1 `cargo doc --no-deps -p universal-agent-runtime-sdk` builds with zero warnings
- [x] 3.2 `sphinx-build -b html sdks/python/docs <out>` builds successfully
- [x] 3.3 `npm run docs` (typedoc) in `sdks/typescript` builds successfully

## 4. `tools/validate-examples.sh`

- [x] 4.1 Script builds/typechecks every example across all three SDKs
- [x] 4.2 Script runs the self-contained `error_handling` example end-to-end
- [x] 4.3 Script supports `VALIDATE_EXAMPLES_LIVE=1` for an opt-in full live-server run
- [x] 4.4 Script exits non-zero on any failure, reports a pass/fail/skip summary

## 5. CI wiring

- [x] 5.1 New `sdk-examples` job in `.github/workflows/ci.yml` running `tools/validate-examples.sh` on every push/PR

## 6. Verification

- [x] 6.1 `bash tools/validate-examples.sh` passes locally (19/19 checks)
- [x] 6.2 `openspec validate sdk-examples-cookbook-rustdoc --strict` passes
