EXECUTION: fix-container-rust-toolchain-pin-consistency
Project: universal-agent-runtime
Date: 2026-08-22
Selected backend: openspec
Dispatched to: Codex SELF
Backend rationale: The defect is bounded, has a strict validated OpenSpec change, and requires source-bound local evidence.
Backend entrypoint: /opsx:apply fix-container-rust-toolchain-pin-consistency
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-container-rust-toolchain-pin-consistency/plan.md

EXECUTION SCOPE

- fix-container-rust-toolchain-pin-consistency: Bind the production backend image build to the dated repository Rust toolchain and reject mismatches locally.

DISPATCH CONTRACTS

- fix-container-rust-toolchain-pin-consistency → Codex SELF
  Entry: /opsx:apply fix-container-rust-toolchain-pin-consistency
  Progress file: .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-container-rust-toolchain-pin-consistency/progress.json
  Canonical tasks: openspec/changes/fix-container-rust-toolchain-pin-consistency/tasks.md
  Handoff: commit the implementation, verify that clean commit locally, commit evidence as its direct child, then reflect the child and restore the parent on the resolved evidence-commit SHA.

APPROVAL GATES

- OpenSpec strict validation: PASSED before Execute.
- Independent history-free artifact critic: APPROVE.
- Independent history-free artifact judge: APPROVE.
- Complete local production image build: REQUIRED before evidence commit.
- Artifact-refiner validation: REQUIRED before child completion.

FALLBACK CONDITIONS

- Stop rather than fall back if the dated Rust channel, locked dependency graph, workspace Cargo files, GitHub Actions, runtime source, deployment behavior, or public API must change.
- Stop if the one-line selector repair exposes an unrelated production-image failure.

VERIFICATION REQUIREMENTS

- `bash -n scripts/verify-runtime-image-toolchain-pin.sh`
- Positive repository pin/effective-channel contract check.
- Floating selector, repository-pin mismatch, and effective-channel mismatch negative controls.
- Same-fixture ARM64 checks under `nightly-2026-07-18` and `nightly-2026-08-22`, with hashes, status, `rustc -Vv`, and E0283 failure recorded.
- `docker buildx build --check --platform linux/arm64 --build-arg RUST_TOOLCHAIN=nightly-2026-07-18 -f Dockerfile .`
- Complete local production image build from the clean implementation commit with the same platform and build argument.
- `openspec validate fix-container-rust-toolchain-pin-consistency --strict`
- Required artifact-refiner validation gate.

PROGRESS LEDGER

- IN_PROGRESS fix-container-rust-toolchain-pin-consistency — Codex SELF

OUTPUTS

- Dockerfile selector repair.
- Local toolchain-pin consistency script.
- Locked isolated dependency-control fixture.
- Row-form verification evidence and child reflection/handoff.

BLOCKERS

- NONE

REFLECTION HANDOFF

- Consume the plan-to-delivery delta, tested implementation SHA, evidence-only handoff SHA, all positive and negative controls, complete-image result, and the exact parent certification restart command.

EXECUTION READY
