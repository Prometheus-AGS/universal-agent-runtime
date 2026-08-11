# Cross-harness handoff protocol

How a KBD phase splits between the Claude Code desktop harness and the Codex
desktop harness. Written 2026-08-11 from what `uar-spec-conformance-2026-08`
actually did, including what went wrong.

## The split

| Stage | Harness | Why |
|---|---|---|
| assess · analyze · spec · plan | **Claude Code** | Reads across the repo, argues with critics, writes the contract |
| execute · reflect | **Codex** | Bounded, test-shaped, multi-file work against a written contract |

The boundary is the **spec handoff**. Everything Codex needs must be on disk and
in git before it starts, because it does not share this conversation.

## What Claude Code owns

1. `goals.md`, `assessment.md`, `analysis.md`, the change specs, and
   `EXECUTION-CONTRACT.md`.
2. **Adversarial review of the whole change set in one packet**, not change by
   change. The failures that matter across a handoff are cross-change: a
   verification gate contradicting its own acceptance criteria, two changes
   editing the same file with no stated order, a dangling reference to another
   change's task.
3. `progress.json` seeded with `generatedBy: "agent-seeded"` and no
   `sourceRevision`/`frontier`, so it claims no runtime provenance.
4. The Codex prompt (below).
5. **Independent verification after Codex reports done** — re-run the phase's own
   verification command on a fresh checkout. Reading the executor's committed
   artifacts is not verification; this phase found the executor honest, which is
   exactly why the check must be habitual rather than suspicion-driven.

## What Codex owns

Execution against the contract, commits per change, and its own reflection notes.
It updates task checkboxes; it does not author new specs.

## The EXECUTION-CONTRACT.md is the load-bearing artifact

A spec set that validates individually can still be unexecutable by an agent that
cannot ask questions. In this phase the adversarial review returned
**INSUFFICIENT** on six findings, and **every one was about autonomous
executability rather than correctness**:

- implicit CI inheritance between changes
- a dangling "the pinned command" reference resolvable only from another change
- ambiguous scope for a requirement binding all changes
- no verification-record format
- three spec deltas to one capability with no precedence rule
- an undefined boundary for what counts as satisfied

The contract must therefore state, explicitly:

| Element | Why |
|---|---|
| **Execution order**, and whether it is load-bearing | Changes editing the same file must not run in parallel |
| **Precedence** when several deltas touch one capability | Otherwise the executor guesses at merge |
| **The verification command, quoted verbatim** | Never "the pinned command from change 1" |
| **Which requirements bind all changes** vs one | Ambiguous scope produces two different deliverables |
| **What counts as satisfied**, including exclusions | "Does an `absent_` case count?" must not need a human |
| **One verification-record format** | Rows from three changes must be comparable |
| **Stop conditions** | The executor must know when to halt rather than guess |

## Stop conditions are the safety mechanism

Codex halted for 15 hours in this phase rather than check a box that would have
misrepresented the result — the spec targeted L3 for capabilities the runtime has
no dependency on. **That halt was correct behaviour and produced a real
correction to the spec.**

Design the contract so halting is cheaper than guessing. Every stop condition
should name a specific observable, not a vibe:

- the verification command diverges from the baseline in a way this change does
  not explain
- a runtime change beyond the named permitted surface appears necessary
- a task seems to require editing the spec being measured against
- a pre-existing failure unrelated to the change in hand

## The Codex prompt

Point at the contract; do not restate it. Include:

- the worktree/branch to work in, created from current `main`
- **read `EXECUTION-CONTRACT.md` first**, named as non-optional
- the change list in execution order
- the goal condition — what "done" means, verifiably
- the verification command verbatim, with every load-bearing flag explained
- tier discipline and which hooks will block
- the permitted runtime surface, if any
- stop conditions
- reporting constraints (this phase: no aggregate percentage, no runtime verdict)
- commit per change; do not push; do not open a PR

## Known failure modes from this phase

**The ledger goes stale.** `progress.json` read 1/6 while the truth was 6/6 for
most of a day. Neither harness updates it automatically. Claude Code should
reconcile it at close, from the executor's real state rather than from the plan.

**Branch state is easy to misread.** Codex pushed to a branch and the work was
merged to `main` by PR before Claude Code looked. Local `main` was stale, so a
first check reported "nothing from Codex is on main" — wrong. **Fetch and compare
against `origin/main`, not the local ref**, before concluding anything about
where work lives.

**Scope changes arrive silently.** Codex added five exclusions where the reviewed
spec sanctioned one, and wrote a repo-wide CI prohibition into a
measurement-phase spec delta. Both were defensible; neither went through review.
The reconciling harness must diff the merged spec against the reviewed spec and
surface every delta — the executor is not obliged to flag its own scope changes,
so the check belongs on the authoring side.

## Reconciliation checklist at phase close

- [ ] Fetch; compare the executor's branch against `origin/main`, not local
- [ ] Re-run the verification command independently on a fresh checkout
- [ ] Diff the merged spec against the reviewed spec; surface every delta
- [ ] Update `progress.json` from real state
- [ ] Write `reflection.md` leading with the delta, not the result
- [ ] Gate any worktree removal on unique `.prometheus` content, not `git status`
- [ ] Record open scope questions as operator decisions, not as closed items
