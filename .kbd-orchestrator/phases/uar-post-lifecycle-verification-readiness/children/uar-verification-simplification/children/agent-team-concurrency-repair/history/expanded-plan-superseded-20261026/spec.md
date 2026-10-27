# Spec — agent-team-concurrency-repair

Status: specified, not implemented, not adopted and not verified. OpenSpec backend resolved from the existing project. One change: `openspec/changes/repair-agent-team-concurrent-operation/`. Read proposal.md, both specs/**/spec.md deltas, design.md, tasks.md and verification.md together. Stable acceptance IDs: ADMIT-01, SETTLE-01, TEAM-01. Stable tasks:1.1,2.1,2.2,2.3,3.1,3.2,3.3. No task is complete.

The change adapts cand-001/cand-002 and retains cand-003 from analysis. Requirements separate the concrete inspected-source mismatch from the unconfirmed historical cause and require truthful outcome/accounting reconciliation. All actual execution remains pending.

`adopted-contract.json` is the schema-named DRAFT required by kbd-spec; null digest/revision, explicit pending adoption and implementationComplete=false make its status clear. No canonical adoption/bind-contract/signature was performed. It neither resets budgets nor supersedes the parent contract/obligations. `fixture-inputs.json` fingerprints retained input documents only; named live/source/authority prerequisites remain NOT_EXERCISED, to be frozen in Plan. No new runtime liveness or credential check occurred.

Applicable recalled lesson: authored source and an older delivered artifact are different evidence, and unresolved persistence outcomes must remain distinct from task setup. Unrelated recalled UI gaps are not added to this repair.

Next: Plan orders concrete writers, operational fixture/activation commands, capability merge/archive ordering, remaining-allowance reconciliation and the final review/build/BDD packet. Until those are bound, the proposed commands are requirements rather than executable authority. No OpenSpec validate or product verification was run; CLI status reports artifact existence only.

## Review disposition

Whole-change artifact review completed in two bounded rounds. Round1 found that uncertainty could incorrectly satisfy terminal idempotency; requirements now require real known-terminal replay including a genuine usage=None fixture, or BLOCKED. Round2 PASS with an isolation wording warning; the exact allowed fixture set and two separately identified settlement calls plus cached-command control were clarified afterward. That narrow clarification was not sent for a third review. Review receipts, warning resolution and anti-theater score0.0 are in review/spec/. Native-only/cross-model-unverified limitation remains.

The OpenSpec CLI reported all proposal/spec/design/task artifacts present. This is metadata, not validation or acceptance; openspec validate, source review, compiler/build/test and live fixture operations remain unrun.
