# Specification — `fix-container-rust-toolchain-pin-consistency`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: validate the child change's source-bound toolchain repair and its
  evidence without expanding the implementation or claiming parent
  operational resilience.
- Deterministic execution: required.
- Inputs: the committed implementation, child OpenSpec artifacts, clean ARM64
  fixture controls, clean production-image build receipt, and scoped git diff.

## Target state

- The production backend command explicitly consumes the dated Docker
  `RUST_TOOLCHAIN` argument.
- Repository, Docker default, and effective build argument agree exactly.
- Floating and mismatched selectors fail before Docker compilation.
- The locked ARM64 dependency compiles on the repository channel and reproduces
  the observed E0283 failure on the incompatible channel.
- A clean `linux/arm64` production image completes from the implementation SHA.
- Evidence remains profile-limited and binds a later evidence-only handoff to
  the tested implementation commit without claiming the parent soak passed.

## Uncomfortable fact

The earlier syntax-only Docker validation was green while the actual backend
command escaped to the moving `nightly` alias. Only a complete clean production
image build exercised the failing dependency path.
