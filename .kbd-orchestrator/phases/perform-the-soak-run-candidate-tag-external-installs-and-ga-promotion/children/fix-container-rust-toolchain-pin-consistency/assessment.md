ASSESSMENT: fix-container-rust-toolchain-pin-consistency
Project: universal-agent-runtime
Date: 2026-08-22
Codebase baseline: Immutable candidate 487fb394 installs dated nightly-2026-07-18 in the production Docker toolchain stage but the backend build overrides it with floating cargo +nightly.
Cross-tool progress: none inside this new child; the parent shutdown child is complete at 28/29 OpenSpec tasks and awaits a replacement candidate certification.

IMPLEMENTATION STATUS
- Dated Rust toolchain declaration: DONE — Dockerfile ARG RUST_TOOLCHAIN and rust-toolchain.toml both name nightly-2026-07-18.
- Toolchain installation: DONE — the Docker toolchain stage installs RUST_TOOLCHAIN as its default toolchain.
- Backend compilation consistency: PARTIAL — Dockerfile line 225 invokes cargo +nightly, bypassing the declared/default dated pin and resolving the moving nightly channel.
- Compatibility evidence: DONE — an isolated ARM64 diskann-wide 0.54.0 probe passed under nightly-2026-07-18 and failed under floating nightly with the same three E0283 errors observed in Docker.
- Regression prevention: MISSING — no local check currently rejects an unqualified +nightly selector in the production Dockerfile.
- Replacement candidate: MISSING — no source repair or successor immutable commit exists.

CROSS-TOOL PROGRESS
- NONE — the child was created by the canonical KBD runtime at revisions 278–279. Existing parent work completed the graceful-shutdown implementation and focused evidence; no other tool has implemented this child.

SPEC GAP SUMMARY
- The current Dockerfile contradicts its own dated-pin comments and rust-toolchain.toml invariant because the backend selector remains floating.
- openspec/specs/gke-deployment/spec.md is stale: it requires a Rust 1.87/latest-stable three-stage image, while the delivered production image intentionally uses a dated nightly polyglot toolchain. Planning must add an exact spec delta without pretending the stale stable-image wording describes current behavior.
- The historical pin-rust-nightly-and-bump-go phase verified Dockerfile syntax but explicitly deferred the full image build. It therefore did not exercise the backend selector after cache invalidation.
- The current child scope permits only child KBD artifacts. Plan must explicitly permit Dockerfile, a narrow OpenSpec change, a local toolchain-consistency check, verification evidence, and append-only project history before Execute.

BUILD HEALTH
- build check: FAIL — CARGO_TARGET_DIR=/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/target scripts/certify-operational-resilience-local.sh certify reached docker build, then cargo +nightly failed compiling diskann-wide 0.54.0 on Linux ARM64 with three E0283 errors and exit 101.
- native control: PASS — the same certification attempt completed the native release build at candidate 487fb394 in 49.93 seconds with the three pre-existing warnings.
- pinned dependency control: PASS — cargo +nightly-2026-07-18 check compiled diskann-wide 0.54.0 and the scratch crate in 14.22 seconds.
- floating dependency negative control: FAIL AS INTENDED — cargo +nightly check on Rust 1.100.0-nightly reproduced all three E0283 errors and exited 101.
- known violations: production backend compilation bypasses the authoritative dated toolchain pin; current canonical gke-deployment toolchain wording is stale.
- test coverage: PARTIAL — the direct compatibility pair proves the failure mechanism, but no checked-in regression check or successful source-bound production image build exists yet.

CONSTRAINT CHECK
- AGENTS.md violations: the Dockerfile contradicts the repository rule to verify and pin dependency/toolchain versions; no new violation was introduced by this assessment.
- constraints.md violations: N/A — no separate constraints file governs this surface.
- phase-order compliance: Assess is active; no product source edit has occurred.
- GitHub Actions policy: COMPLIANT — all observations and controls ran locally, and no workflow change is proposed.
- immutable-candidate rule: COMPLIANT — candidate 487fb394 remains unchanged; any repair must create a new commit and restart certification from zero.

GOAL PROGRESS
- Make the production Docker backend consume the dated RUST_TOOLCHAIN pin: NOT MET — cargo +nightly still selects the moving alias.
- Prove pinned ARM64 passes while floating nightly reproduces E0283: MET — both controls were observed with exit 0 and exit 101 respectively.
- Commit a replacement candidate and return parent to certification: NOT MET — source repair, verification, commit, child reflection, and parent handoff remain.

ASSESSMENT COMPLETE
