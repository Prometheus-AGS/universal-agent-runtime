# Phase Reflection: codex-harness-comparative-analysis

Project: Universal Agent Runtime
Date: 2026-09-04
Implementation completion: 100% (10/10 changes; five planned rounds)
Archival: 10/10, including eight newly approved archives
Wider project implementation: 111/120; not a release-completion claim

## Delta, root causes and corrective actions

The first green phase suite did not prove the written requirements. Independent
review found five missing behaviors: governed read-only concurrency was unreachable,
metadata prematurely ended retry eligibility, primary chat replay started another
run, catalog pressure discarded titles, and never-dispatched remote work retained
budget leases. The original tests exercised helper paths or insufficient fixtures.
The corrective batch connected the real host paths, then added phase-end integration
regressions. Those tests exposed a related UAR-router/Agent-child mode mismatch and
a catalog separator cost problem; both were fixed before the final full rerun.

The final locked server-full check, formatting and test command exited0. Its
713 passing library tests, 94 broad integration tests, nine BDD scenarios/49 steps
and26 doctests are evidence for the executed matrix, not universal coverage.
One library test, one broad integration test and17 doctests were ignored.
Memory/PostgreSQL feature variants were excluded. Real remote-peer cancellation,
provider-side billing termination and several replay side-effect scenarios remain
unverified. The real-provider429 receipt is still unchecked and explicitly deferred.
The operator approved archival with those warnings preserved.

The original test-first/per-change cadence was superseded by the operator's
complete-phase-code-before-tests instruction. Source compilation remained the
cheap edit check; runtime tests were authored/executed at phase boundaries.
Stdio sandbox support used the proposal's rejection option rather than adding an
unverified OS sandbox. Remote delegation targets authenticated UAR peers, per the
operator's choice, rather than claiming arbitrary A2A peers enforce inherited
authority. Legacy skill activation remains the migration default until recall
evidence supports its separate default change.

## Goals

| Goal | Status | Evidence |
|---|---|---|
| Compare Codex and UAR across the named runtime axes | MET | assessment.md, analysis.md and evidence/ contain source-backed comparison; upstream-survey limitations remain disclosed |
| Produce a source-cited gap analysis including additional practices | MET | analysis.md and its verified Codex excerpts identify observed UAR defects; implemented corrections map to those gaps |
| Rank findings and write an ordered implementation plan with spec deltas | MET | plan.md names ten changes in five dependency rounds; all ten have archived delta specs |
| Execute the ten-change plan added by the operator | MET for implementation and local phase gate | Canonical10/10, archived10/10, full corrected suite passed; deferred/live limits are not erased |

The three original research/planning goals are3/3 met by their artifacts. The
operator's wider all-phases objective is PARTIAL: the parent presentation-selection
work and later certification/release work remain.

## Delivered Changes

- context-history-integrity — paired history, pinned system context, bounded outputs and checkpoint restore (by: Codex continuation; prior work preserved).
- deterministic-prompt-assembly — ordered authority fragments and redacted manifests (by: Codex).
- fail-closed-tool-arguments — compiled schema validation, declared effects and governed concurrency (by: Codex).
- model-path-resiliency — typed retry/failover, idle interruption and exact-run HTTP replay (by: Codex).
- progressive-skill-runtime — budgeted catalogs, explicit activation, retention and attribution (by: Codex).
- typed-turn-assembly — owned requests, staged contributors, immutable steps and shadow comparison (by: Codex).
- projected-mcp-runtime — host-local catalog/binding authority, lazy readiness, search and joined lifecycle (by: Codex).
- thread-native-subagents — durable threads across actor/graph/A2A adapters, narrow authority, shared budgets and cancellation (by: Codex).
- project-instructions-world-state — trusted instruction discovery and host-owned world-state deltas (by: Codex; existing archive preserved).
- typed-turn-default-flip — evidence-gated Typed default with Legacy rollback (by: Codex).

The context archive is dated2026-09-02; the other nine are dated2026-09-04.
archive-receipt.json records approval and before/after SHA-256 inventories for
all eight moves performed during closeout. No archive file content changed.
Nine main capability specs were synced and strict-validated. Approval UI/replay
requirements were preserved; the earlier legacy-default statement was scoped as
migration history before applying the evidence-gated Typed default.

## Verification

Command:
`cargo check --locked --no-default-features --features server-full &&
cargo fmt --all -- --check &&
cargo test --locked --no-default-features --features server-full`

Recorded terminal result: exit0, owned session72074. Build2m26s; broad integration
863.63s. The command's full result summary, initial failures and limitations are
retained at openspec/changes/archive/2026-09-04-typed-turn-default-flip/evidence/audit-correction-report.md.
No test was rerun just to move documentation. Each synced capability returned
`Specification '<name>' is valid` from `openspec validate <name> --type spec --strict`.
Archive hash comparisons matched; the deferred task remained `- [ ]`.

## Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with formal artifact-refiner logs | 0/10 |
| Formal first-pass pass rate | Unavailable; not0% or100% |
| Formal refinement iterations | Unavailable |
| Fallback review | Independent artifact-only source reviews and local phase tests |
| Corrective findings after initial green suite | Five named defects plus test-exposed related corrections |

Formal artifact-refiner QA was skipped because its required execution tools and
per-change logs were unavailable. No formal pass rate is invented. Independent
critics drove substantive fixes; review acceptance is distinct from test execution.

## Technical Debt and Coverage Limits

- Archived model-path-resiliency/tasks.md5.4: a real provider429 remains unobserved.
  Do not generate abusive traffic or relabel a controlled429 response as live.
- tests/agent_threads.rs: memory and PostgreSQL contracts are feature-gated;
  PostgreSQL additionally needs a real DATABASE_URL. server-full did not prove them.
- thread-native-subagents/evidence/: real-model cancellation reached a local child
  before its first response, not a trusted remote peer or an after-text case.
- tests/integration/live/chat_replay_cases.rs: loopback mode, enabled memory/quality
  side effects and cancelled-run replay limits remain. Existing SurrealKV shutdown
  warnings are disclosed, not silently treated as deployment certification.
- phase-close-verification.md enumerates remaining scenario coverage warnings,
  including approval rejection/timeout, recall misses and credential rotation.
- The parity corpus has three requests and live shadow smoke has two k3 cases.
  Zero differences in those cases is not all-provider or all-tool parity.
- Historical progress projections retain duplicate task rows and unrelated
  certification/publication summaries. They must not manufacture code gaps or
  current release evidence. Completion dimensions were not globally overwritten.

## Architecture Integrity

Mutations remain in trusted host services; contributor snapshots do not gain
native implementations, credentials, transport handles or write authority.
Real boundaries include owner/tenant replay checks, narrowed child policy,
exact MCP binding identity and never-dispatched lease accounting. No new guard
was added during sync/archive/reflection. No dependency pin or workflow changed.
Operator-owned versions.toml was not edited during closeout.

No new architectural violation was identified by the recorded reviews; that is
not an independent exhaustive security certificate. The existing checkout is
outside worktreeRoot. No worktree relocation, broad staging, commit, push,
deployment or destructive cleanup was performed. The absent constraints.md
provides no additional gate. Direct delegated shell execution remains denied
where physical confinement is not implemented; lifecycle ownership is not isolation.

## Cross-Tool Coordination Notes

Progress tracking: GAPS FOUND. Legacy task-history duplicates and inherited
completion summaries required separate OpenSpec counts and explicit evidence
receipts. Repeated continuations must follow owned process handles, not assume
a lost observation means a compiler stopped. No parallel Cargo writer was used
for the corrected final suite.

Handoff quality: the source plan and append-only execution notes preserved scope;
stale in-progress prose and old line numbers required source checks. Operator
pins, peer trust choice and archive confirmation were explicit gates. Once
archive approval arrived, the eight approved records moved without another
confirmation loop. Historical evidence paths refer to their original command
locations; archive directory paths are the current retrieval locations.

## Lessons Learned

- Test the installed host path, not only the helper carrying the same name.
- Build fixtures with meaningful nonempty metadata; test pressure and omission.
- Distinguish semantic model output from lifecycle/usage metadata for retry.
- An event cursor must identify projected frames and preserve exact run ownership.
- Never-dispatched proof can release a lease; uncertain dispatched work cannot.
- Keep phase implementation, local evidence, certification and publication distinct.
- Check every archive's bytes and preserve approved incomplete evidence explicitly.

## Next Phase Focus

Return to agui-a2ui-selection-architecture, currently2/3 implementation complete.
The remaining change is select-and-observe-presentations. It has no written plan,
so begin Spec/Plan before code:

1. Resolve the text/A2UI/hybrid selection contract against current policy and client capability.
2. Wire selection and observable AG-UI lifecycle through the delivered typed runtime.
3. Implement the full planned phase before running its real integration tests.

Read the parent artifacts and current source before defining tasks. Required UI
skills and independent critiques apply if UI code is touched. Do not treat a
research recommendation as authorization to widen external deployment or release
scope. No evolver-bridge.json exists for this child; no evolver state was changed.

## Context for Next Phase

Use this reflection, archive-receipt.json and the archived phase-close-verification.md
as prior context. Preserve the deferred429 and coverage warnings during subsequent
certification. Stop only for a genuine new authority/design conflict; ordinary
implementation continues autonomously under the operator's instruction.
