# Separate findings

- #466/#465: Liter policy/source mismatch: versions.toml 31c7e632; committed gitlink 0d74c566 (prepared streaming body fix). Preserve working source. Owner question pending; no reset or pin change.
- Dependency audit at phase exit: pnpm reports 21 vulnerabilities, including DOMPurify critical GHSA-cg2g-xh64-v4q7 and Zod high GHSA-mrhx-c7xc-m3w5. This is audit output, not a demonstrated exploit. Scope separately against pinned compatibility decisions, not automatic #462 dependency upgrades. No public claim corrected by hiding findings.
- Host build topology: repo target-dir is worktree-relative but ~/.cargo/config.toml build.build-dir is globally shared at /Volumes/my-passport/cargo/build-dir. Original-checkout Cargo PID69032 was observed active. Preserve its work and serialize; do not erase caches, redirect it or infer its outcome.

- Full Rust formatting inventory failed with broad pre-existing differences (83,833 log lines). Preserve existing conventions; no unrelated formatting sweep added to #462. Exact raw log/digest in acceptance receipt.

## Superseding disposition
Operator selected retaining current Liter source. The pending policy/source mismatch above is resolved by aligning versions.toml to0d74c56659943e3fa44a3c8e6d51a27514f3b35c. Qualification remains pending; no dependency source was changed.

## Candidate8e0602ee affected inventory failures

Full-target compile-only inventory failed before execution: settings_persistence fixture lacks ResilienceConfig.admission_timeout_ms, PersistenceConfig.surreal_auth_level, AppConfig.service_instance and upsert_provider_config force/flag argument. These are unrelated API fixture drift, not the #462 duplicate initializer; retained raw diagnostic in evidence/full-inventory-attempt-003.txt. Do not guess the Boolean meaning or add a default silently. No settings fixture edits authorized in the active warning cohort.

Full-library inventory:857passed,8failed,1ignored. Two Liter prompt-cache stream fixtures assume a first content/error event but encounter preserved #388 budget_receipt; context hierarchical tier fixture expects6 and observes7; two native-identity manager assertions fail; three remote thread-admission fixtures fail root readiness with skill_matching_unavailable/skill_service_resource_required. Exact panic summaries in evidence/full-library-attempt-003-failures.txt. These are observed failures; root causes beyond the explicit error are unverified. Do not remove budget receipts, loosen identity/approval gates, automatically copy the other chat's changes, or claim these runtime guarantees are certified. Reconsider relevant fixtures in their actual next capability/state slices; preserve separate source provenance.

Proper-feature retained approval compilation emitted17intentional shared-helper dead-code warnings and1sidecar deprecated registration warning. These are the only new repair cohort. Cargo exit0 is not a warning-free pass. No historical qualifier body ran.
