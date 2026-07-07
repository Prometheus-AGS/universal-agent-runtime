EXECUTION: uar-dependabot-remediation-2026-07
Project: universal-agent-runtime
Date: 2026-07-07
Selected backend: openspec
Dispatched to: SELF (Claude Code CLI, self-executing)
Backend rationale: OpenSpec is available at project root and this project's
established practice (see prior phases: add-run-cancellation, agent-spec-v2,
guardrail-pii-block, etc.) drives every change through an OpenSpec change dir
for spec-backed traceability. plan.md's own COMMANDS TO RUN section already
names `/opsx:new <change-id>` per change; none of the 8 change dirs exist yet
under openspec/changes/, so this execute phase creates them (plan.md deferred
proposal.md/tasks.md authorship to execute time, per this project's stated
practice) then drives each one task-by-task via /kbd-apply — never bare
/opsx:apply.
Backend entrypoint: /opsx:new <change-id> (scaffold) -> /kbd-apply <change-id>
(task-by-task loop, fires task:before/task:after, syncs progress.json +
waypoint) -> /kbd-apply verify -> /kbd-apply archive
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/plan.md

EXECUTION SCOPE

- kreuzberg-reachable-vulns: bump kreuzberg pin (or patch lopdf/quick-xml) to clear 3 reachable document-parsing CVEs
- surreal-memory-transitive-vulns: resync surreal-memory git pin / cargo update surrealdb-core; disclose reachability for ammonia/crossbeam-epoch/rsa
- direct-network-facing-vulns: bump hickory-proto + tokio-tar to patched versions
- first-party-direct-dep-hygiene: replace/patch serde_yml+libyml (direct dep), check anyhow/memmap2 point releases
- grcov-toolchain-refresh: bump grcov dev-dependency to drop the old cargo-binutils/clap2 chain (or disclose as accepted dev-only risk)
- npm-root-remediation: npm audit fix (semver-safe) on root package-lock.json
- frontend-npm-remediation: pnpm audit --fix (safe) on frontend/pnpm-lock.yaml
- sdk-typescript-lockfile-and-ci-audit-fix: real lockfile + vitest bump for sdks/typescript; new scheduled cargo-audit/npm-audit CI workflow; correct DEPENDENCY_MANAGEMENT.md's stale claim

DISPATCH CONTRACTS

- kreuzberg-reachable-vulns -> SELF
  Entry: /opsx:new kreuzberg-reachable-vulns; then /kbd-apply kreuzberg-reachable-vulns
  Model class: mid
  Concrete model: session default (no model_policy.registry in project.json; self-executing, no per-change model swap)
  Model rationale: plan.md scored Medium-High complexity — single-crate pin bump but requires upstream-tag research before deciding patch-override vs. clean bump
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json (change_status -> DONE) and committing

- surreal-memory-transitive-vulns -> SELF
  Entry: /opsx:new surreal-memory-transitive-vulns; then /kbd-apply surreal-memory-transitive-vulns
  Model class: mid
  Concrete model: session default
  Model rationale: reachability analysis across 3 independent advisories (ammonia/crossbeam-epoch/rsa) — bounded but requires judgment
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing; "not reachable" is an acceptable disclosed outcome per plan.md

- direct-network-facing-vulns -> SELF
  Entry: /opsx:new direct-network-facing-vulns; then /kbd-apply direct-network-facing-vulns
  Model class: mid
  Concrete model: session default
  Model rationale: plan.md scored Low-Medium; mechanical version bump but network-facing so verify carefully
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- first-party-direct-dep-hygiene -> SELF
  Entry: /opsx:new first-party-direct-dep-hygiene; then /kbd-apply first-party-direct-dep-hygiene
  Model class: mid
  Concrete model: session default
  Model rationale: plan.md scored Medium — possible serde_yml->serde_yaml replacement requires usage-site verification, not a pure version bump
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- grcov-toolchain-refresh -> SELF
  Entry: /opsx:new grcov-toolchain-refresh; then /kbd-apply grcov-toolchain-refresh
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Low — dev-dependency-only version bump or disclosed accepted-risk note
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- npm-root-remediation -> SELF
  Entry: /opsx:new npm-root-remediation; then /kbd-apply npm-root-remediation
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Low — npm audit fix within semver ranges
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- frontend-npm-remediation -> SELF
  Entry: /opsx:new frontend-npm-remediation; then /kbd-apply frontend-npm-remediation
  Model class: small
  Concrete model: session default
  Model rationale: plan.md scored Low — pnpm audit --fix within the frontend workspace
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

- sdk-typescript-lockfile-and-ci-audit-fix -> SELF
  Entry: /opsx:new sdk-typescript-lockfile-and-ci-audit-fix; then /kbd-apply sdk-typescript-lockfile-and-ci-audit-fix
  Model class: mid
  Concrete model: session default
  Model rationale: plan.md scored Medium — new lockfile + new scheduled CI workflow + doc correction, three coordinated artifacts
  Progress file: .kbd-orchestrator/phases/uar-dependabot-remediation-2026-07/progress.json
  Handoff: report completion by updating progress.json and committing

APPROVAL GATES

- NONE (all 8 changes are dependency-version/CI-hygiene fixes with no product-facing behavior change; each still gets its own verify checkpoint below)

FALLBACK CONDITIONS

- If a Round 1 change requires a breaking-change major bump that touches src/ business logic beyond the pinned dependency (e.g. serde_yml replacement forces API changes across multiple modules), stop self-executing that single change and re-route it through kbd-plan for its own sub-decomposition rather than forcing it through in one shot.
- If cargo audit/test/clippy checkpoint fails after Round 1 and the failure is not attributable to a specific change, halt Round 2/3 and re-run kbd-assess on the regression before continuing.

VERIFICATION REQUIREMENTS

- Round 1 (Rust, shared checkpoint after all 5 land): `cargo audit` confirms targeted CVEs cleared (not just version-number churn); `cargo test --lib` full suite green; `cargo clippy --lib` zero new warnings vs. current baseline.
- Round 2 (npm, shared checkpoint after both land): `npm audit` / `pnpm audit` re-run shows count drop; `bun run build` + `bun run check` succeed; `pnpm -C frontend build` succeeds.
- Round 3 (own checkpoint): `sdks/typescript` vitest run green after bump; new CI workflow YAML valid and its trigger confirmed to actually fire.

PROGRESS LEDGER (refreshed 2026-07-07 via /kbd-execute re-entry, step 5 of 8)

- [DONE] kreuzberg-reachable-vulns — SELF — archived 2026-07-07-kreuzberg-reachable-vulns
- [DONE] surreal-memory-transitive-vulns — SELF — archived 2026-07-07-surreal-memory-transitive-vulns
- [DONE] direct-network-facing-vulns — SELF — commit fc0f7bd, archived 2026-07-07-direct-network-facing-vulns
- [DONE] first-party-direct-dep-hygiene — SELF — commits 8c1c6fb+eecd09a, archived 2026-07-07-first-party-direct-dep-hygiene
- [DONE] grcov-toolchain-refresh — SELF — commits 20c9795+c719072, archived 2026-07-07-grcov-toolchain-refresh — ROUND 1 COMPLETE (5/5)
- [PENDING] npm-root-remediation — SELF — Round 2, next up
- [PENDING] frontend-npm-remediation — SELF — Round 2
- [PENDING] sdk-typescript-lockfile-and-ci-audit-fix — SELF — Round 3

PROCESS NOTE: /kbd-apply's `verify`/`archive` (openspec validate/archive) hard-require
at least one delta spec per change. Round 1's first 3 changes originally declared
"Capabilities: None" and had to be retrofitted with a new `dependency-security-posture`
capability (see openspec/specs/dependency-security-posture/spec.md and
progress.json's decisions.openspec_verify_archive_gap_2026-07-07). Changes 4-5
followed the pattern from the start with no retrofit needed. Round 2/3 changes MUST
add a dependency-security-posture delta (ADDED Requirement, SHALL/MUST on the
requirement's own unwrapped first line -- the openspec parser only reads line 1)
before running verify/archive.

artifact-refiner QA gate (`/refine-validate`/`/refine-code`) referenced by
/kbd-execute's per-change QA step is NOT wired up in this project -- no
`.agent/skills/artifact-refiner/SKILL.md` or `.kbd-orchestrator/constraints.md`
exist. This was formally decided as retired automation in a prior phase
(`artifact-refiner-gate-decision`, D-E) and is not re-carried as open debt for
this phase either -- skipped per that standing decision, not silently.

OUTPUTS

- Round 1: openspec/specs/dependency-security-posture/spec.md (new capability,
  5 Requirements, one per Round-1 change); docs/DEPENDENCY_MANAGEMENT.md gained
  6 new disposition sections (kreuzberg, rsa, hickory-proto, serde_yml/quinn-proto,
  grcov/proc-macro-error2); 5 changes archived under openspec/changes/archive/2026-07-07-*/

BLOCKERS

- NONE

REFLECTION HANDOFF

- kbd-reflect should consume: final cargo audit / npm audit / pnpm audit counts (delta vs. the 52 Dependabot + 17 cargo-audit + 6 net-new npm baseline in assessment.md), any disclosed "not reachable / accepted risk" items (change #2, #5), the serde_yml replacement decision and whether it was a clean swap, and whether the new scheduled CI workflow (change #8) actually fired on its first scheduled/dispatch run — that's the process fix this whole phase exists to close out.

EXECUTION READY
