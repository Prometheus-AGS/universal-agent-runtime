## Context

See `proposal.md` for motivation. The run contains 51 registered top-level phases: several are already canonical `complete`, while many legacy entries remain `pending` despite reflections or successor phases that establish a terminal outcome. KBD projections are generated from the canonical transition runtime, and the primary checkout contains unrelated operator work that must remain untouched.

Git has one secondary worktree, but its branch is tied to PR conflict-resolution work rather than a KBD phase and contains commits not present on `origin/main`. The phase cleanup therefore cannot infer association merely from the existence of a worktree.

## Goals / Non-Goals

**Goals:**

- Produce one evidence-backed disposition for every registered top-level phase.
- Reconcile canonical KBD state through supported transitions and preserve historical artifacts.
- Remove only phase-associated Git state proven obsolete and lossless to delete.
- Keep the cleanup isolated from product code and unrelated dirty working-tree state.

**Non-Goals:**

- Reopen or implement abandoned product initiatives solely because their legacy phase registration is pending.
- Rewrite historical assessments, plans, executions, or reflections to make partial delivery appear complete.
- Delete PR, dependency, or operator worktrees that are not associated with a reconciled phase.
- Merge, push, or modify remote branches.

## Decisions

### Use three explicit dispositions

Each phase is classified as `complete`, `cancelled`, or `retained`. `Complete` requires direct outcome evidence or a named successor that closes the remaining outcomes. `Cancelled` is used for unexecuted, superseded, or deliberately abandoned scope. `Retained` is reserved for relevant unfinished work that must be handled before cleanup finishes.

The alternative—marking every stale pending phase complete—was rejected because it would erase the distinction between delivered work and scope that was never executed.

### Use canonical transitions rather than editing projections

Status changes run through the KBD phase-transition command. Pending phases destined for completion pass through `in-progress` when required by the state machine; cancelled phases use the legal cancellation path. Generated projection changes are inspected after each class of transition.

The alternative—editing `position.json`, waypoint files, progress files, or task markers directly—was rejected because those files are projections and can diverge from canonical event history.

### Record successor closure without altering historical reflections

A reconciliation verification record maps partial legacy outcomes to the phase that completed them or to an explicit cancellation decision. Existing reflection text remains unchanged so the original delivery record stays truthful.

### Treat Git cleanup as a preservation gate

Candidate worktrees and branches must match a phase by explicit path, branch, or metadata evidence. Before removal, the audit checks tracked and untracked dirt, ignored `.prometheus` files, commits unique to the branch, and merge reachability from retained refs. An unrelated or unique branch is retained and reported.

## Risks / Trade-offs

- [Generated KBD projections overlap existing operator changes] → Apply only canonical transitions, inspect the exact diff, and exclude unrelated files from the cleanup commit.
- [Successor evidence is overstated] → Record the exact successor and prefer cancellation when the remaining outcome was abandoned rather than delivered.
- [A branch appears obsolete but contains unique history] → Retain it unless its work is first preserved on a durable ref.
- [A local-only canonical update cannot synchronize to the control plane] → Verify the local event log and projections, and report synchronization as unverified if the control plane remains unavailable.

## Migration Plan

1. Validate the OpenSpec plan and snapshot canonical phase and Git state.
2. Write the full disposition ledger before any transition.
3. Apply and verify canonical transitions in small groups.
4. Audit and remove only safe phase-associated worktrees and branches.
5. Validate the final OpenSpec change and KBD projections, then commit only scoped cleanup artifacts.

Rollback is performed by reverting the scoped reconciliation commit and replaying inverse legal transitions where the KBD runtime supports them. Historical phase artifacts are never deleted, so evidence remains available even if a disposition must be corrected.
