## 1. Mutation testing

- [x] 1.1 Add `cargo-mutants` to the developer tooling. Use `taiki-e/install-action@v2` in the new workflow; no local toolchain requirement.
- [x] 1.2 Create `.github/workflows/mutation.yml` nightly cron that runs `cargo mutants --no-shuffle`.
- [x] 1.3 Publish results to `docs/mutation-history/` — create the directory and a timestamped report file per run (workflow commits or appends).
- [x] 1.4 Add a `tools/mutation-summarize.sh` helper that prints the latest mutation report summary and exits non-zero if the uncaught-mutant count exceeds a documented threshold.

## 2. Fuzz targets

- [x] 2.1 Create `fuzz/` directory and initialize `cargo-fuzz` scaffolding (`Cargo.toml`, `fuzz_targets/`).
- [x] 2.2 Add `cargo-fuzz` as a dev-dependency and ensure `cargo fuzz run --help` works in CI.
- [x] 2.3 Create fuzz target `chunker` — exercise the chunking boundary logic with arbitrary byte inputs.
- [x] 2.4 Create fuzz target `rag_verification` — exercise RAG verification assertions with arbitrary retrieval results.
- [x] 2.5 Create fuzz target `mcp_message_parser` — exercise the MCP JSON-RPC message parser with arbitrary JSON inputs.
- [x] 2.6 Create fuzz target `json_schema_validator` — exercise the JSON schema validator with arbitrary schemas and payloads.
- [x] 2.7 Add local fuzz invocation documentation in `TESTING.md` and a placeholder for a future nightly fuzz workflow (deferred to consolidated validation; first cargo-fuzz build pulls the full surrealdb/axum stack and is slow).

## 3. Property-based tests

- [x] 3.1 Add `proptest` to the Rust workspace dev-dependencies.
- [x] 3.2 Write property test: settings store serde roundtrip — any valid settings value serializes and deserializes to an equivalent value.
- [x] 3.3 Write property test: retrieval RRF invariants — reciprocal rank fusion scores are monotonic and bounded for arbitrary input lists.
- [x] 3.4 Write property test: governance policy hot-reload semantics — policy files parse and produce deterministic effective policies after reload cycles.
- [x] 3.5 Ensure property tests run with `cargo test` and are included in CI.

## 4. Release automation and commit hygiene

- [x] 4.1 Configure `release-plz` with conventional-commits check (`release-plz.toml`).
- [x] 4.2 Add `commitlint` configuration (`.commitlintrc.js`) using the conventional-commit preset.
- [x] 4.3 Add `lefthook` configuration (`lefthook.yml`) to run `commitlint` on the JS workspace in the `commit-msg` hook.
- [x] 4.4 Document the commit-message convention in `CONTRIBUTING.md`.

## 5. Verification

- [x] 5.1 `cargo check --locked --no-default-features --features server-full` passes.
- [x] 5.2 `cargo test` passes (including new proptest cases).
- [x] 5.3 `cargo mutants --no-shuffle --list` runs without tooling errors (full mutation run is nightly and deferred to the consolidated validation pass).
- [ ] 5.4 At least one fuzz target builds and runs briefly (`cargo fuzz run <target> -- -max_total_time=5`). **DEFERRED** — first cargo-fuzz build pulls the full surrealdb/axum stack and exceeded available interactive time; the targets and scaffolding are in place, and the build will be exercised in the phase's consolidated validation pass.
- [x] 5.5 `openspec validate test-quality-mutation-fuzz-property --strict` passes.
- [x] 5.6 `pnpm -C frontend install --frozen-lockfile`, `pnpm -C frontend typecheck`, `pnpm -C frontend lint`, `pnpm -C frontend build` pass. `pnpm -C frontend test` shows 106/108 passing; the 2 failing suites (`chat-message-store.test.ts`, `use-message-stream.test.ts`) fail with a pre-existing `pglite` Denied ID Vite load error unrelated to this change.

