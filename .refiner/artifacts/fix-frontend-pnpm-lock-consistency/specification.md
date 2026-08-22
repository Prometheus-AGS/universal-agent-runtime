# Specification

Refine the `fix-frontend-pnpm-lock-consistency` candidate as a direct-content
artifact. The candidate must make the independently active ten-project frontend
pnpm workspace reproducible under pnpm 11.15.0 without changing dependency
intent or unrelated resolutions, and must return only a reviewed child commit
to parent screen certification.

Acceptance authority:

- child KBD goals, plan, execution contract, and `scope.json`;
- OpenSpec proposal, `frontend-build-tooling` delta, design, tasks, and
  per-requirement verification;
- exact stale-lock, deterministic-regeneration, causal-delta, frozen-install,
  Tier 0/focused-unit, and scope receipts.

The uncomfortable counterexample is a lock that passes frozen metadata checks
but still fails package materialization or silently moves unrelated common
snapshot bodies. Both failure modes must be excluded by observed evidence.
