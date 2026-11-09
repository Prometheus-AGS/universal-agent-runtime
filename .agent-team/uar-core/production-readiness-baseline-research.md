# #462 local baseline safeguard — source investigation

Date: 2026-10-31. Owner: baseline_research. Scope: source inspection and this handoff only. VERIFICATION LOCK CLOSED: no tests, lint, format, typecheck, Cargo/compiler/build, review-driven verification, external mutation, or source-identity gate was executed. Others share this checkout; their edits were preserved.

## Position and observed defect

`.kbd-orchestrator/current-waypoint.json` reports active phase `identity-pass-through`, parent `uar-verification-simplification`, revision 700, plan revision 178, 144/170 implementation items complete. It does not currently identify this production-readiness cohort. The lead must reconcile canonical phase state before any proposed verification; this handoff cannot open the gate.

`src/uar/runtime/presentation_history_tests.rs` currently initializes `RunStreamState.caller_handoff_confirmation` twice in `manager_with_run` (one before `verified_owner`, another after `history`). `src/uar/runtime/manager.rs:60-62` includes this module only under `cfg(test)`. A normal library check/build cannot establish that this library test target compiles. This is source evidence, not an observed compiler result. The parent was notified; this worker made no source repair.

`.prometheus/gotchas.md` contains the 2026-10-27 entry “Gate plans must follow the real-path evidence rule”: stale lib-test initializers remained broken since October 20/22 and were discovered incidentally while executing a filtered gate. It also records that filtered/mock tests cannot replace real-path behavior acceptance. #462's baseline must explicitly remain supplementary compile/regression evidence.

## Existing integration points

| Path | Existing behavior and consequence |
| --- | --- |
| `scripts/dot/acceptance.py` | `source_identity()` fingerprints tracked/untracked scoped source, file mode/symlink, dirty patch, recursive submodules and content. `boundary()` checks the full production ledger, handoffs, current phase, inventory digest, `verification_authorized`, and exact source identity. It is hardwired to `dot-feature-support`; do not call it as a generic gate for the current waypoint. |
| `scripts/dot/acceptance.py` | `source_command(target)` is `cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test <target> -- --include-ignored --nocapture --test-threads=1`. `source_gates()` runs entire real integration targets serially and records command/log digest/result; it requires substantial existing infrastructure. It does not compile the root library test target explicitly. |
| `scripts/dot/build_delivery.py` | `run_candidate_build()` serially runs `pnpm install --frozen-lockfile`, `pnpm -C frontend install --frozen-lockfile`, `pnpm build`, `node scripts/validate-static-bundle.mjs static`, and `cargo build --locked --release --no-default-features --bin universal-agent-runtime --features server-full`. Root library unit-test code is outside this build's claim. |
| `scripts/dot/finalize_source.py`, `scripts/dot/reconcile.py` | Preserve source lineage and distinguish source/local acceptance from independently installed published artifacts. Reuse their evidence concepts, without making a lib baseline certify an installed artifact. |
| `scripts/generate-release-input-provenance.mjs` | Default immutable release source requires a clean checkout and recursive matching gitlinks. Optional `--candidate-inventory <path>` calls Dot `acceptance.boundary()`, so this option is also Dot-specific. This script is a gate and was not executed. |
| `scripts/prepare-release-evidence-local.sh` | Takes `<archive-directory> <evidence-directory>`, requires `TEST_EVIDENCE_FILE`, `SECURITY_AUDIT_EVIDENCE_FILE`, `BUILD_RECEIPT_FILE`, validates each `.source_sha` against HEAD, and includes evidence in the signed manifest. It also signs/attests remote image content: do not invoke casually for this baseline. |
| `scripts/validate-release-manifest.mjs`, `schemas/release-manifest.schema.json` | Bind test/audit evidence file bytes and `.source_sha` to release source, but do not currently require a #462 feature matrix or exact content-bound library baseline payload. |
| `scripts/certify-release-candidate.sh` | Installed-archive behavior certification, with its own fixtures; not a source library baseline integration point. |
| `docs/product-support-matrix.json` | Stable `minimal`, stable `server-full`, preview `server-postgres`/`postgres-backend`. Existing strings advertise `cargo test --lib --features minimal`, `cargo check --lib --features server-full` plus BDD/certification, and `cargo test --features postgres-backend`. The server-full check string misses `cfg(test)`. |
| `.github/workflows/deploy.yml`, `.github/workflows/docs.yml` | Deployment/publication workflows. Do not add tests, baseline compilation, typechecks, or product certification. Preserve them. Editing workflow-invoked scripts would require the policy validator before/after, which is presently prohibited by CLOSED lock; prefer a new purely local entry point. |

The VLBV document `docs/validation/uar-validation-methodology.original.html` section 9 explicitly preserves KBD phase ownership and existing adopted-contract `scenarioMatrix` / `boundaryPacket.postImplementationOnly` rather than introducing a competing workflow. A #462 baseline belongs in the adopted phase's explicit command packet after implementation completion, not an automatic per-edit hook.

## Recommended minimal design

Add one local-only baseline script (new path to be selected by lead) plus its narrow documentation/OpenSpec delta. It should run only when explicitly included in an operator-approved complete-phase packet. Avoid changing deployment workflows, the installed-artifact runner, or the Dot-specific boundary just to run a generic library safeguard. Reuse the phase's adopted exact source inventory and record actual argv, feature set, timestamps, exit result, complete compiler log and log digest. Capture the exact source before/after inside that approved command so the receipt cannot certify a changed tree. Do not treat HEAD alone as identity for dirty candidate work.

Proposed serial compile matrix (proposed commands only; none executed):

```sh
cargo test --locked --no-default-features --features minimal --lib --no-run
cargo test --locked --no-default-features --features server-full --lib --no-run
cargo test --locked --no-default-features --features server-full,postgres-backend --lib --no-run
```

Use one lead-owned `CARGO_TARGET_DIR`, serially. The Postgres combination is supported by real release source (`docs/releases/v1.2.3-acceptance.md`, `docs/releases/v1.3.0-dependency-review.md`) and Cargo feature definitions; `in-memory-backend` is a fixture feature in the existing Dot integration command, not required to make the supported production profile compile. `--no-run` is already used in `scripts/acceptance/prepare_artifacts.README.md:100`.

Root-package selection should be explicit if the new script can run from another directory or use workspace flags. The package is `universal-agent-runtime`; do not broaden this to all workspace targets inadvertently. Compile failures and all repository/vendor warnings are failures per operator rule; do not suppress warnings using RUSTFLAGS or blanket allow. Preserve failed logs and stop after the first failure. A later source edit closes the operator gate and invalidates affected evidence until renewed authority.

After successful compilation, #462 should also require named regression observations for `uar::runtime::manager::presentation_history_tests`, `uar::api::openai::embeddings::tests`, and `uar::api::openapi::tests`, under an explicitly approved final-phase command. Their scope is supplemental regression behavior, not production acceptance. If filtered tests remain disallowed for this task, use the full root `--lib` run for each profile and record presence of the required test names in actual harness output rather than proposing an exception silently. The lead must resolve that acceptance shape in the plan. Do not claim compile-only evidence ran the assertions. The OpenAPI and embeddings modules are `server`-gated, so all three named profiles include them; `api-docs` is not needed to include `src/uar/api/openapi.rs` itself.

The current #460 modules already contain 11 embeddings cases (ordered batch/canonical model, single input/dimension, base64 bytes, model mismatch, dimension mismatch, empty/oversized/blank input, encoding mismatch, backend unavailable, provider error mapping, malformed batch, token-ID deserialization refusal) and `spec_uses_package_version_and_documents_customer_routes`, which includes `/v1/embeddings`. Presentation-history has four async cases covering ring eviction, publication/terminal ordering, restored provenance/forgery, and exact subject/tenant failure-stream access. Preserve these checks. No need to create parallel duplicate test modules.

For eventual release claims, require the baseline receipt as an explicit local release prerequisite and bind it through the existing test-evidence artifact, source SHA plus candidate content identity, rather than adding any Actions job. This can initially be a documented release packet obligation; enforcing it in shared release validators expands scope and requires separately approved policy checks. Historical receipt presence is not fresh source acceptance.

## Source prerequisites

- Pinned host toolchain: `rust-toolchain.toml` selects `nightly-2026-07-18`; preserve `Cargo.lock` and `versions.toml`.
- `.gitmodules`: `models.dev`, `crates/prometheus-skill-system`, `frontend/packages/prometheus-entity-management`, `vendor/git/liter-llm`, `vendor/git/rust-mcp-filesystem`. Exact source identity recursively includes initialized submodules; mere directory existence is insufficient. `scripts/update-submodules.sh` exists but uses force/init/network/config mutation and was not run or recommended inside a closed verification command.
- Cargo local dependencies include `vendor/git/liter-llm/crates/liter-llm`, `vendor/git/surreal-memory-server/crates/surreal-memory`, `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-core`, optional `vendor/git/sycophancy-correction/crates/sycophancy-core`.
- Cargo patches require `vendor/patches/gemm-common-0.19.0`, `vendor/crates/esaxx-rs`, and the Surreal lifecycle SDK/engine/core/KVS paths listed in root `Cargo.toml`. These source exports must accompany an exact-tree compile; do not replace them with registry packages to make a probe work.
- Root `build.rs` consumes committed `catalog/provider_catalog.json` and committed `proto/a2a.proto`, `proto/a2a_v1.proto`, using pinned vendored protoc when `a2a-transport` is enabled. It intentionally does not fetch schema/catalog sources. Native/full-profile dependency build prerequisites were not executed or certified here.
- `scripts/package-offline-source.sh` recursively packages tracked files and runs `cargo vendor --quiet --locked`; this is not a harmless source read and must not be invoked under the closed lock.

## Handoff and limitations

Changed only this handoff. No production code, tests, workflow, credentials, configuration or memory changed. No unrequested guards added. Source inspection commands were `cat`, `sed`, `rg`, `pwd`; no verification command ran and no result is asserted. All proposed compile/regression commands, warning cleanliness, actual submodule readiness, real platform integration, current #460 behavior and final release usability remain unverified. The uncomfortable limit: a comprehensive lib compile still cannot certify provider inference, durable backend operation, or the installed customer artifact; the existing approved real-path gates remain necessary.
