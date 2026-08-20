## Context

See `proposal.md` for motivation. `frontend/pnpm-workspace.yaml` defines a
separate ten-project workspace whose importer set includes the pinned
entity-management submodule. The committed nested lock predates the current
submodule manifests. pnpm 11.15.0 therefore rejects frozen installation even
though the independently maintained repository-root lock is valid.

The main worktree already contains an operator-owned nested-lock candidate, but
its hash differs from two independent regenerations from the committed lock.
That candidate is evidence of the observed rewrite, not an automatic source of
truth.

## Goals / Non-Goals

**Goals:**

- Produce one nested lock accepted by pnpm 11.15.0 in frozen metadata-only and
  empty-dependency-tree installation.
- Tie every HEAD-to-candidate mutation to a current frontend or pinned-submodule
  manifest change.
- Preserve common snapshot bodies whose movement is optional resolver drift.
- Leave both lock hashes unchanged across the parent TypeScript, lint, and
  focused SSE unit commands.

**Non-Goals:**

- No manifest, root-lock, submodule-pin, product-source, generated-asset, or
  parent-certification change.
- No dependency modernization or adoption of the newest version allowed by an
  unchanged range.
- No browser certification inside this child.

## Decisions

### Regenerate twice from the committed lock and pinned manifests

Two detached clean worktrees at commit `1274039a` initialize the same submodule
pin and run pnpm 11.15.0 lock-only resolution. Byte-identical output establishes
resolver reproducibility. This is preferred to copying the dirty main-worktree
lock because that file carries prior execution residue and a distinct graph.

### Classify the candidate against HEAD, not only against another regeneration

Independent resolvers can agree on optional allowed-range upgrades. The audit
therefore compares importer specifiers, importer resolved values, package keys,
snapshot keys, and bodies directly with `HEAD:frontend/pnpm-lock.yaml`.

### Retain three pre-existing common snapshot bodies

The resolver candidate changes the bodies for six common snapshot keys. Three
changes follow the new submodule peer contexts (`tsx`, `jsdom`, and associated
Vitest paths). Three do not need to move:

- `@typescript-eslint/project-service@8.64.0` retargets its internal 8.64.0
  dependencies to 8.66.0.
- `chromatic@16.10.0` retargets `semver` 7.8.1 to 7.8.5.
- `storybook@10.2.13` retargets `semver` 7.8.1 to 7.8.5 and `ws` 8.21.0 to
  8.21.3.

Their HEAD bodies are retained. Frozen metadata and empty-tree installation are
the acceptance tests for this minimum-delta choice.

## Risks / Trade-offs

- **[Risk]** Peer-context key churn makes a large textual diff look like broad
  upgrades. **Mitigation:** retain a structured HEAD-to-candidate audit that
  separates importer intent, package key replacement, snapshot key replacement,
  and common-body mutation.
- **[Risk]** A metadata-only frozen check can pass while package materialization
  fails. **Mitigation:** also install from an empty `frontend/node_modules` tree.
- **[Risk]** A warm worktree can hide lock mutation or package residue.
  **Mitigation:** perform final clean installation in a detached external
  worktree and record pre/post hashes.
- **[Trade-off]** The child does not add a new validator script. The behavioral
  contract becomes canonical, while automation beyond the existing pnpm frozen
  gate remains deferred to avoid expanding a lock-only repair.

## Migration Plan

1. Replace the dirty main-worktree lock with the independently reproduced
   minimum-delta candidate.
2. Validate frozen metadata and focused commands in the implementation
   worktree without lock mutation.
3. Create a detached worktree from the child commit, initialize the pinned
   submodule, and run an empty-tree frozen install.
4. Archive the OpenSpec change only after artifact-refiner and independent
   critic/judge approval.
5. Return control to parent screen certification at the child commit.

Rollback is the single child commit; no runtime data or public interface is
migrated.
