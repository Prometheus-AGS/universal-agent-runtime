# Handoff in — dot-feature-support› issue-379-dot-replan

**Spawned by:** dot-feature-support

## Why this child was spawned

The parent Dot phase is in Execute at change 07, with changes 08–17 pending.
Its plan places verification after individual changes, contrary to the
operator's full-phase boundary. Issue #379 introduces nine Kubo Topology B
conformance requirements, most of which are not in the Dot feature contract.
Investigate overlap, ordering, and release scope before mutating the parent
plan or starting another production subsystem.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/dot-feature-support/assessment.md
- .kbd-orchestrator/phases/dot-feature-support/plan.md
- .kbd-orchestrator/phases/dot-feature-support/progress.json
- https://github.com/Prometheus-AGS/universal-agent-runtime/issues/379
- https://github.com/GQAdonis/kubo/blob/feat/ipfs-mcp-server/mcp/uar/requests/CONFORMANCE.md
- AGENTS.md full-phase verification and agent-team rules

## Success criteria

- Map all nine issue requirements to existing Dot commitments, current source,
  missing contracts, and dependencies without claiming unrun behavior.
- Recommend a dependency order, ownership, and release split that preserves the
  Dot first-release commitments for local and shared hosting.
- Give a concrete parent-plan revision that defers every verification task
  until all parent-phase production implementation is complete.
- Identify unresolved contract or release decisions for the operator; do not
  silently choose them or alter the parent Execute ledger.

## Expected deliverables

- Child assessment, analysis, and plan with a reviewable issue-to-change matrix
  and revised dependency/order proposal.
- Handoff-out stating exact parent plan and KBD ledger edits to make after the
  child planning loop, plus decisions still needed.
