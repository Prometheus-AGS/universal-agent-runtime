# Proposal: UAR execution authorization

## Why

Root approval still accepts run-only decisions and authenticated HTTP MCP session crossover is untested. Exact prepared invocation and existing claim-before-effect must survive a strict owned-client cutover.

## What Changes

- **BREAKING**: root approval requires exact pending identity; no legacy local exception.
- Migrate owned UAR positive helpers alongside the separately owned Boss approval client.
- Diagnose all session operations on unchanged production with valid A/B JWTs; correct only reproduced crossover.

## Capabilities

### New Capabilities
- execution-bound-authorization: approved exact-approval and evidence-gated session contract.

### Modified Capabilities
None in this new execution spec root; original provider expected revision and host claim behavior are preserved.

## Impact

Child project UUID: 06d6112e-346a-4417-8296-3b0ba3dba220. Product root: /Users/gqadonis/.claude/worktrees/bauar-uar. Accepted product base: a7cb972992d4f83db6585449ea81af0fe4a1c990. Parent phase: bossfang-uar-authorization-and-execution. Only this nested workstream owns the child KBD state. Original repository KBD state and C05 delivery gates remain untouched.

Exact source inventory, dependency closure, contracts and diagnostic protocol: [UAR prerequisites](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/.kbd-orchestrator/phases/bossfang-uar-authorization-and-execution/evidence/execute/uar-prerequisites.md).

Parent bauar-02 tasks 2,3 (UAR helper portion),5,6,7,8 map to six child tasks. Parent 02/1 inventory/cutover and 02/4 Boss claim boundary remain parent/Boss ownership. Spec includes the cross-component acceptance contract; this child does not claim to implement Boss files.
