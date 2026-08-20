PLAN: fix-frontend-pnpm-lock-consistency
Project: universal-agent-runtime
Date: 2026-08-20
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-frontend-pnpm-lock-consistency: reconcile the independently active frontend workspace lock with the pinned manifests while preserving unrelated common resolutions
   - Scope: nested frontend dependency lock, frontend-build-tooling spec, OpenSpec evidence, child KBD and append-only history
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: HIGH
   - Details: Reproduce the pnpm 11.15.0 candidate from the committed lock and current pinned entity-management manifests, then retain the HEAD bodies for the three common snapshots whose movements are not required by those manifest changes. Prove both frozen metadata and empty-dependency-tree installation without lock mutation. Do not edit manifests, product source, the root lock, submodule pins, generated frontend assets, or parent screen evidence.

EXECUTION ROUND ORDER
Round 1: fix-frontend-pnpm-lock-consistency

VERIFICATION ORDER
1. Retain the clean committed-lock negative control: frozen install exits 1 with `ERR_PNPM_OUTDATED_LOCKFILE` while the stale lock remains byte-identical.
2. Reproduce the clean resolver candidate twice and record identical SHA-256.
3. Apply the minimum-delta correction by retaining the HEAD bodies for `@typescript-eslint/project-service` 8.64.0, `chromatic` 16.10.0, and `storybook` 10.2.13; record the exact HEAD-to-candidate audit.
4. Observe frozen lock-only and empty-dependency-tree installs exit 0 without changing the nested lock.
5. Run `pnpm typecheck`, `pnpm lint`, and the focused frontend SSE unit; assert the nested and root lock hashes remain unchanged across each.
6. Run strict OpenSpec, scoped diff, artifact-refiner, and history-free critic/judge gates before archive and commit.

SCOPE CUTS AND TRADE-OFFS
- Do not adopt the operator-owned main-worktree lock candidate: its SHA differs from both independent clean regenerations and it carries a different set of peer-context and allowed-range choices.
- Do not keep fresh-resolver changes to the pre-existing project-service 8.64.0, chromatic 16.10.0, or storybook 10.2.13 common snapshots. Frozen installation proves they need not move.
- Do not add a validator script in this child. The OpenSpec requirement and replayable evidence establish the missing nested-workspace contract without expanding product/tooling source.
- Parent browser certification remains outside this child and resumes only from the child commit.

COMMANDS TO RUN
`/opsx:new fix-frontend-pnpm-lock-consistency`

PLAN COMPLETE
