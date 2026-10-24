# Issue #426 execution contract — 2026-10-24

## Authority and position

The operator approved the complete proposed plan, including a final minor version bump, with “Implement the proposed plan.” Active phase: `uar-post-lifecycle-verification-readiness`, Execute. OpenSpec change: `update-parking-lot-admission`. The canonical journal registers its tasks; generated waypoint/progress/task projections must not be edited directly.

The prior waypoint revision 268 reported 144/166 run implementation items, while the active phase's prior projection reported six completed changes. Run totals and active-phase totals are different scopes; neither constitutes acceptance. The earlier exactNextCommand stopped caller-tool work; the new typed plan revision records the newly authorized issue scope.

## Approved decisions

- Exact upstream source: `4b747350c6883497813bed33623a9937ad7a5894`, manifest 0.3.0, replacing `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`.
- Narrow dispatcher adapter. UAR retains actual worker handles, ResourceScope leases, durable failure transitions and publication fencing.
- One slot/job; worker count zero uses CPU count; 100 waiting descriptors in server composition. Waiting bound is independent of active jobs and durable accepted backlog.
- Normal priority, local FIFO, zero head skips. No new default execution budget, durable upload refusal, second durable queue or unrelated runtime scheduling.
- Root UAR version becomes 1.3.0 after production implementation and before the final inventory. Independently versioned JS/Tauri packages retain their versions.
- versions.toml and pinned host/container toolchains remain unchanged. Vendored provenance owns the new Parking Lot source record.

## Single-writer ownership

| Writer | Owned implementation | Handoff |
|---|---|---|
| parking_vendor | vendor/git/prometheus-parking-lot-rs/**; vendor/git/README.md | .agent-team/uar-core/handoffs/issue-426-vendor.md |
| parking_adapter | ingestion_worker.rs; optional ingestion_admission.rs and rag/mod.rs declaration | .agent-team/uar-core/handoffs/issue-426-adapter.md |
| parking_acceptance | tests/ingestion_admission.rs; durable_ingestion.rs/lifecycle_owned_resources.rs if required; docs/releases/v1.3.0-parking-lot.md | .agent-team/uar-core/handoffs/issue-426-acceptance.md |
| lead | root Cargo.toml/Cargo.lock; src/server.rs; CHANGELOG.md; OpenSpec/KBD/team/memory records | issue-426-inventory.md |

All writers share the checkout and must preserve other edits. The existing .codex/config.toml modification is unrelated and excluded. At most four participants are active. Independent reviewer launch is deferred while verification is CLOSED.

## Verification lock and command classification

Gate status: **CLOSED**. No test, lint, formatter, typecheck, Cargo check, compiler probe, build, package validation, OpenSpec validate, independent code-review gate, or indirect equivalent is authorized. Before commands/delegation, classify them as source inspection, authorized implementation/documentation/memory writing, or verification. Only the first two categories may run.

Allowed work so far: source/config/manifest reads; GitHub issue and exact upstream source retrieval; native task dispatch; OpenSpec artifact scaffolding/instructions; canonical KBD status and typed scope/task registration; local team coordination records; authorized production/test/doc authoring. None establishes a runtime PASS.

The KBD apply driver's lifecycle hooks are not invoked as a workaround for the lock; typed canonical registration/transitions record authored work directly without running hook-driven acceptance. One redundant change-start transition was refused because task starts already moved the change to InProgress. Registration and six task starts committed; no state reset or guessed retry was used.

Before verification, reconcile the entire active phase and every implementation task, collect handoffs, and record a source-bound inventory. Present remaining work and exact proposed commands to the operator and wait for explicit approval. Any implementation edit closes the gate again. No GitHub issue edits, PR, push, tag, release, paid jobs or deployment is authorized.

## Exit criteria

Source deliverables include the exact vendored snapshot and compatible lockfile, bounded production adapter, authored finite integration coverage, scope/version documentation, UAR1.3.0 identity and durable writer handoffs. Verification, independent review and delivered-artifact compatibility remain separately pending until approved commands actually run. Do not mark the change accepted, archive it, or close issue #426 based on source authoring alone.

The uncomfortable limitation is that no compiler or runtime evidence may be collected under the current lock. Source inspection reduces implementation ambiguity but cannot certify toolchain compatibility or worker settlement behavior.
