## Purpose

Ensures the production runtime image's backend compilation uses the dated
repository toolchain so clean local candidate builds are reproducible.

## ADDED Requirements

### Requirement: Production backend uses the repository's dated Rust toolchain
The production image SHALL declare one dated Rust channel, SHALL keep that
declaration equal to the repository Rust toolchain channel, and SHALL select
that declared channel for the production backend compilation. The effective
candidate build argument SHALL equal the repository channel. An unqualified
moving channel such as `nightly`, or a mismatched external build argument, MUST
NOT override the dated declaration.

#### Scenario: Clean Linux ARM64 production image uses the repository channel
- **WHEN** the production image is built from a clean source checkout for `linux/arm64` with the effective `RUST_TOOLCHAIN` build argument recorded
- **THEN** its Rust installation and backend compilation both select the dated repository channel and the locked backend dependency graph compiles successfully

#### Scenario: Docker and repository pins disagree
- **WHEN** the production image channel declaration differs from the repository Rust toolchain channel
- **THEN** local toolchain-consistency validation fails before the image is accepted as a candidate

#### Scenario: Backend selects floating nightly
- **WHEN** a production-image Rust build command selects an unqualified moving `nightly` channel
- **THEN** local toolchain-consistency validation fails even if a dated channel is installed elsewhere in the image

#### Scenario: Candidate build argument overrides the repository channel
- **WHEN** the effective candidate `RUST_TOOLCHAIN` build argument differs from the repository channel
- **THEN** local toolchain-consistency validation fails before the production image build begins

### Requirement: Candidate toolchain evidence is local and source-bound
Before a production image becomes an immutable candidate, local verification
SHALL record `rustc -Vv` for both dated channels, SHALL demonstrate
compatibility with the locked dependency that failed under the incompatible
channel, and SHALL complete the actual production-image build for a clean
implementation commit. The verification artifact MAY be committed as the
direct evidence-only descendant of that tested commit and SHALL name the tested
commit. After that evidence commit exists, the canonical KBD handoff SHALL
resolve its SHA and the parent certification SHALL rebuild that final handoff
commit from zero. Routine build verification MUST NOT be delegated to GitHub
Actions.

#### Scenario: Repository channel accepts the locked ARM64 dependency
- **WHEN** `diskann-wide 0.54.0` is checked on an ARM64 host with `nightly-2026-07-18`
- **THEN** the check completes successfully and records `rustc -Vv` for that exact channel

#### Scenario: Moving channel is the negative control
- **WHEN** the same locked dependency is checked on the same ARM64 host with the observed incompatible `nightly-2026-08-22`
- **THEN** the check fails with the recorded E0283 compiler diagnostic and records `rustc -Vv` rather than resolving a later moving alias

#### Scenario: Image evidence is non-circular and source-bound
- **WHEN** the complete local production-image build succeeds from a clean implementation commit and its evidence is added afterward
- **THEN** the direct evidence-only commit names the tested implementation commit, and the subsequent canonical handoff records the evidence commit SHA that the parent must rebuild and certify from zero
