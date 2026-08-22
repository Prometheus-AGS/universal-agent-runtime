## Context

See `proposal.md` for the certification blocker. The root workspace includes
the pinned `frontend/packages/prometheus-entity-management` submodule as a pnpm
workspace project. Its committed manifest now differs from the root lock's
importer, so a clean frozen install fails before build preparation.

Two candidate resolutions exist. The initial operator-owned lock candidate
passed metadata-only frozen validation but also moved two pre-existing edges:
`@eslint/config-array` from `minimatch` 10.2.5 to 10.2.6 and `y-webrtc` from
`ws` 8.21.0 to 8.21.1. The corrected candidate restores both edges while
retaining `ws` 8.21.1 separately because the advanced `entity-graph-sync`
manifest pins it directly. Regenerating from the stale committed lock passes,
but current allowed ranges also move `lucide-react` from 1.32.0 to 1.33.0 and
collapse the preserved `y-webrtc` edge. Neither movement is required to repair
the observed importer mismatch.

## Goals / Non-Goals

**Goals:**

- Commit a root lock accepted by pnpm 11.15.0 frozen installation for the exact
  committed workspace and submodule pins.
- Preserve resolved versions unrelated to the submodule manifest advance.
- Retain replayable positive and negative evidence before parent certification
  resumes from a new immutable source commit.

**Non-Goals:**

- Changing manifests, dependency ranges, package-manager versions, submodule
  pins, product source, or generated frontend assets.
- Claiming that dependency installation proves product or browser behavior.
- Running or minting the parent browser certification inside this child.

## Decisions

### Adopt the exercised lock candidate rather than resolve current range latests

The child will retain the operator-owned candidate after the minimum-delta
correction. The resulting digest is
`645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`.
It includes the entity-management importer, restores the two noncausal HEAD
edges, and passes clean frozen installation without mutation.

A clean non-frozen regeneration was rejected because it resolves currently
allowed versions that differ from the graph already exercised. That would
combine an importer repair with unrelated dependency movement and would require
a separate dependency decision and verification surface.

### Verify both metadata consistency and installation

The positive proof will record the candidate digest before and after:

1. frozen lock-only installation with lifecycle scripts disabled; and
2. full frozen installation with lifecycle scripts disabled.

Both commands must exit zero and leave the digest unchanged. The retained
negative control is the clean `fa4ffb96` worktree where the committed stale
lock exits non-zero with `ERR_PNPM_OUTDATED_LOCKFILE`, including the reported
missing and mismatched importer counts.

Lock-only validation was rejected as sufficient because it proves importer
metadata but does not prove that the complete frozen dependency graph can be
installed. That distinction was observed: the first surgical correction
passed lock-only validation but a clean install failed because the new direct
`ws` 8.21.1 record had been removed. The final graph retains both required
`ws` versions and passes from empty dependency directories.

### Keep certification ownership in the parent

This child produces one new source commit containing the lock repair and its
contract evidence. The parent must then create a fresh clean worktree from that
commit, run preparation with frozen dependencies, and mint a new source-bound
browser bundle. Reusing any earlier bundle is forbidden because its source and
dependency graph differ.

## Risks / Trade-offs

- **[Large generated diff obscures unintended resolution movement]** → Compare
  the adopted candidate with both the stale lock and the clean-regeneration
  control, record the only candidate/control differences, and run the existing
  supply-chain validator.
- **[A frozen check passes because of an existing install]** → Run in the clean
  certification worktree after the corrected source commit before parent build
  preparation; the child command itself remains scoped to dependency validity.
- **[Retaining an older allowed version delays an available update]** → Treat
  dependency upgrades as separate changes. Reproducibility and minimum scope
  take precedence in this blocker repair.

## Migration Plan

Commit the corrected root lock with its OpenSpec and KBD child evidence. If the
lock must be rolled back, parent certification remains blocked because the
prior source commit is observably not frozen-installable; no earlier evidence
bundle may be promoted as a substitute.
