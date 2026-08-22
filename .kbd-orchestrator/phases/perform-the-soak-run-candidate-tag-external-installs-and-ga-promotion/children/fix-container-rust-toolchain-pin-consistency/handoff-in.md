# Handoff in — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-container-rust-toolchain-pin-consistency

**Spawned by:** perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

## Why this child was spawned

The immutable candidate's local production-image build selected floating
`nightly` despite an existing dated pin, then failed compiling locked
`diskann-wide` on ARM64 before resilience assertions could run. Parent freeze
rules prohibit patching the candidate inline.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/assessment.md
- .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md

## Success criteria

- The Docker backend build explicitly consumes the dated `RUST_TOOLCHAIN` pin.
- A checked-in local contract rejects pin mismatch and floating nightly.
- The pinned compatibility control and complete production image build pass.
- A replacement commit is reflected back to the parent, whose full
  certification restarts from zero.

## Expected deliverables

- One validated OpenSpec change and one scoped implementation commit.
- Row-form verification with the observed floating-nightly negative control.
- Child reflection and `handoff-out.md` naming the replacement candidate SHA.
