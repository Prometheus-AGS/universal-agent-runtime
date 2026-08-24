## 1. Pin the published runtime

- [x] 1.1 Replace the frontend `workspace:*` dependency with exact `@prometheus-ags/prometheus-entity-management` 3.0.2 and add exact `@prometheus-ags/entity-graph-core` 3.0.2 peer ownership; verify the manifest contains neither a range nor a workspace selector for either product dependency.
- [x] 1.2 Regenerate the root and frontend lockfiles from their own workspace roots; verify both importers resolve registry snapshots with the recorded 3.0.2 integrity metadata and no application `link:` target.

## 2. Verify the dependency boundary

- [x] 2.1 Inspect `pnpm list` and `pnpm why` from both roots; verify one compatible 3.0.2 core runtime satisfies the 3.0.2 React package and preserve the observed command output.
- [x] 2.2 Run the existing platform/entity import boundary and frontend Tier 0 checks; verify application and test source still access the package only through `frontend/src/platform/entities`.
- [x] 2.3 Run `openspec validate adopt-entity-management-3-0-2 --strict`, write row-form `verification.md` with source SHA/profile/limits, and commit only this change's manifest, lockfile, OpenSpec, and KBD artifacts.
