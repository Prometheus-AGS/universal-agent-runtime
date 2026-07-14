# Upstream: `@a2ui/web_core`

`@prometheus-ags/a2ui-core` vendors [`@a2ui/web_core`](https://www.npmjs.com/package/@a2ui/web_core)
from npm — the core rendering / state-management library of the
[A2UI project](https://a2ui.org/) (Agent-to-User Interface), published by
Google (`a2ui-team <a2ui-owners@google.com>`, org repo
[`a2ui-project/a2ui`](https://github.com/a2ui-project/a2ui), directory
`renderers/web_core`).

## Pinned version

| Field | Value |
|---|---|
| Package | `@a2ui/web_core` |
| Pinned version | **0.10.4** (exact, no semver range) |
| License | Apache-2.0 |
| Confirmed via | `npm view @a2ui/web_core` on 2026-07-14 — `latest: 0.10.4`, published 2026-07-08 |
| npm tarball shasum | `ca8e1bbbd2e3b875211f02e44ef24838189d3255` |

The pin is enforced in `package.json`'s `dependencies` field with an exact
version string (`"0.10.4"`, not `"^0.10.4"`), so `pnpm install` will not
silently drift to a newer upstream release. `pnpm-lock.yaml` additionally
locks the resolved integrity hash.

## Why a dependency pin instead of a source copy

`@a2ui/web_core` ships **compiled JS + `.d.ts`** on npm, not its original
TypeScript source — the actual TypeScript lives in the upstream monorepo
(`a2ui-project/a2ui`, `renderers/web_core/`), which is a separate,
independently-versioned project with its own build tooling. Copying the
npm tarball's compiled output into this repo would mean vendoring build
artifacts we cannot usefully edit, and would make every upstream bump a
manual artifact-diff exercise instead of a `pnpm update` review.

Change 16 (`a2ui-vendor-google-core-react`, this package) and the changes
that consume it — Change 17 (`a2ui-uar-renderer-on-webcore`) and Change 22
(`a2ui-inspector-lit-svelte-renderers`) — both build **on top of**
`@a2ui/web_core`'s public API (message processing, data/component/surface
models, catalog types) rather than modifying its internals. Nothing in the
current phase plan requires patching `@a2ui/web_core` source. If a future
change needs to fork or patch upstream behavior, that is the trigger to
switch this package from a dependency pin to a real source vendor (clone
`a2ui-project/a2ui`, extract `renderers/web_core/`, and preserve its
Apache-2.0 headers file-by-file) — at which point this file should be
updated to record that decision.

## How to update the pin

1. Check the current published version and changelog:
   ```bash
   npm view @a2ui/web_core versions --json
   npm view @a2ui/web_core dist-tags
   ```
   or read `CHANGELOG.md` in the published tarball
   (`npm pack @a2ui/web_core@<version>` then inspect).
2. Review the upstream `renderers/web_core/CHANGELOG.md` for breaking
   changes to the exported surfaces this package re-exports
   (`.`, `./v0_8`, `./v0_9`, `./v0_9/basic_catalog`).
3. Bump the **exact** version string in
   `frontend/packages/a2ui-core/package.json` `dependencies["@a2ui/web_core"]`.
4. Update the table above (pinned version, confirmation date, shasum).
5. Run `pnpm -C frontend install` to regenerate `pnpm-lock.yaml`, then
   `pnpm -C frontend --filter @prometheus-ags/a2ui-core typecheck`.
6. If Change 17's renderer (`frontend/packages/a2ui-uar/`) or the Change 22
   Lit/Svelte renderers exist by the time of the bump, re-run their test
   suites against the new pin before merging.

## License

Apache-2.0, © Google LLC. The full license text is preserved unmodified
in [`./LICENSE`](./LICENSE) (copied verbatim from the upstream package),
and every upstream source file this package re-exports retains its
original `Copyright 2025 Google LLC` / Apache-2.0 header inside
`node_modules/@a2ui/web_core` — this package does not touch or strip
those files.
