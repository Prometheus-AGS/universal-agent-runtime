## 1. Establish the Lock Boundary

- [x] 1.1 Retain the clean committed-lock negative control with the exact frozen-install command, non-zero output, and unchanged stale-lock digest.
- [x] 1.2 Retain two independent pnpm 11.15.0 lock-only regenerations from commit `1274039a` and verify their SHA-256 digests are identical.
- [x] 1.3 Record a structured HEAD-to-regeneration comparison covering importer specifiers, importer resolved values, package keys, snapshot keys, and common snapshot bodies.

## 2. Implement the Minimum-Delta Nested Lock

- [x] 2.1 Replace `frontend/pnpm-lock.yaml` with the deterministic regeneration and verify its importer specifiers match all ten current workspace projects.
- [x] 2.2 Restore the HEAD bodies for the three noncausal common snapshots named in `design.md` and verify no other common package or snapshot body changes lack a manifest cause.
- [x] 2.3 Verify `frontend/package.json`, `frontend/pnpm-workspace.yaml`, the entity-management gitlink, root `pnpm-lock.yaml`, and product source remain unchanged by the child implementation.

## 3. Verify Reproducibility and Parent Compatibility

- [x] 3.1 Run frozen lock-only validation under pnpm 11.15.0 and verify the nested lock digest remains unchanged.
- [x] 3.2 Run a frozen install from an empty `frontend/node_modules` tree in a detached external worktree and verify the nested lock digest remains unchanged.
- [x] 3.3 Run `pnpm typecheck`, `pnpm lint`, and `pnpm -C frontend test src/entities/sync.test.ts`; verify each exits zero and neither the nested nor root lock digest changes.
- [x] 3.4 Run scoped diff checks and strict OpenSpec validation and retain literal commands and observed outputs in `verification.md` and evidence files.

## 4. Close the Child Phase

- [x] 4.1 Run artifact-refiner with progressive checkpoints and verify active/history artifacts, schemas, references, constraints, hashes, and receipts agree.
- [x] 4.2 Obtain history-free artifact critic and judge PASS decisions on the frozen candidate and correct every blocking finding before archive.
- [x] 4.3 Sync the accepted delta into `openspec/specs/frontend-build-tooling/spec.md`, archive the change, and verify strict validation of the active change before archive, the canonical spec after sync, and archived-task completeness through OpenSpec's archive validator.
- [x] 4.4 Complete KBD Reflect, append the nested-workspace lesson to `.prometheus`, commit only the permitted child surface, push the existing branch, and hand the resulting child commit back to parent screen certification.
