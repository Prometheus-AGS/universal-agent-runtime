# Separate findings

- #466/#465: Liter policy/source mismatch: versions.toml 31c7e632; committed gitlink 0d74c566 (prepared streaming body fix). Preserve working source. Owner question pending; no reset or pin change.
- Dependency audit at phase exit: pnpm reports 21 vulnerabilities, including DOMPurify critical GHSA-cg2g-xh64-v4q7 and Zod high GHSA-mrhx-c7xc-m3w5. This is audit output, not a demonstrated exploit. Scope separately against pinned compatibility decisions, not automatic #462 dependency upgrades. No public claim corrected by hiding findings.
- Host build topology: repo target-dir is worktree-relative but ~/.cargo/config.toml build.build-dir is globally shared at /Volumes/my-passport/cargo/build-dir. Original-checkout Cargo PID69032 was observed active. Preserve its work and serialize; do not erase caches, redirect it or infer its outcome.

- Full Rust formatting inventory failed with broad pre-existing differences (83,833 log lines). Preserve existing conventions; no unrelated formatting sweep added to #462. Exact raw log/digest in acceptance receipt.

## Superseding disposition
Operator selected retaining current Liter source. The pending policy/source mismatch above is resolved by aligning versions.toml to0d74c56659943e3fa44a3c8e6d51a27514f3b35c. Qualification remains pending; no dependency source was changed.
