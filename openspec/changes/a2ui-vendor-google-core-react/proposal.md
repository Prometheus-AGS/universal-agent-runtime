## Why

The `uar-grade-a-upgrade-2026-07` phase plan (operator decision Q2)
commits UAR to an A2UI (Agent-to-User Interface) surface built on
Google's A2UI project rather than a from-scratch protocol
implementation. Change 16 is the foundation for that six-change
A2UI workstream (Changes 16–21): before UAR can build its own
renderer (Change 17), migrate entity components (Change 18), or add
a design-system bridge (Change 19), the workspace needs a pinned,
disclosed, license-clean dependency on Google's upstream A2UI
libraries.

## What Changes

- Add `frontend/packages/a2ui-core/` — a version-pinned workspace
  package that re-exports Google's **`@a2ui/web_core`** npm package
  (confirmed real and published: v0.10.4, Apache-2.0, maintained by
  `a2ui-team <a2ui-owners@google.com>`, upstream repo
  `a2ui-project/a2ui`, directory `renderers/web_core`).
- Add `frontend/packages/a2ui-react/` — a version-pinned workspace
  package that re-exports Google's **`@a2ui/react`** npm package
  (confirmed real and published: v0.10.1, Apache-2.0, same org),
  explicitly labeled **reference implementation only** — not the
  UAR-owned renderer (that is Change 17, out of scope here).
- Preserve Apache-2.0 license text (`LICENSE` copied verbatim into
  both new packages) and per-file `Copyright 2025 Google LLC`
  headers (untouched, inside the real npm packages this workspace
  now pins).
- `frontend/packages/a2ui-core/UPSTREAM.md` records the exact pinned
  version, how it was confirmed, and the update procedure.

### Plan correction: none needed on package names

The plan's assumed npm scope/names (`@a2ui/web_core`, `@a2ui/react`)
are correct — verified directly against the npm registry on
2026-07-14 (`npm view @a2ui/web_core`, `npm view @a2ui/react`) and
cross-checked against `a2ui-project/a2ui` on GitHub (linked from
both packages' `homepage`/`repository` fields, `https://a2ui.org/`).
Both packages exist, are actively published (most recent releases
6 days and 3 weeks prior to this audit), and are Apache-2.0.

### Vendoring approach: pinned dependency wrapper, not a source copy

"Vendoring" is implemented as an **exact-pinned npm dependency behind
a stable internal import path**, not a copy of the npm tarball's
compiled output into git. Rationale, made explicit per this change's
instructions:

- `@a2ui/web_core` and `@a2ui/react` ship **compiled JS + `.d.ts`**
  on npm; the actual TypeScript source lives in the separate
  `a2ui-project/a2ui` monorepo. Copying the tarball into this repo
  would vendor build artifacts we cannot meaningfully edit or diff
  against upstream changes.
- Changes 17 and 22 — the only changes in this phase that consume
  `@a2ui/web_core` — both build **on top of** its public API (a
  React renderer and, later, Lit/Svelte renderers). Neither modifies
  or extends `@a2ui/web_core` internals. Change 17's done condition
  cross-tests against `@a2ui/react` purely as a **reference**, again
  without modifying it. Nothing in Changes 17–21 requires patching
  upstream source, so there is no forcing function for a full source
  vendor in this change.
- The two new packages (`@prometheus-ags/a2ui-core`,
  `@prometheus-ags/a2ui-react`) pin an **exact** version (no `^`/`~`
  range) as their only dependency and re-export it, per surface
  (`.`, `./v0_8`, `./v0_9`, `./v0_9/basic_catalog` for core; `.`,
  `./styles` for react). This gives UAR code a single internal
  import path to depend on, and a pinned-upgrade discipline
  (`UPSTREAM.md`), while remaining trivial to keep current.
- If a future change needs to fork or patch `@a2ui/web_core`
  internals, `UPSTREAM.md` documents that as the trigger to switch
  this package from a dependency pin to a real source vendor
  (clone `a2ui-project/a2ui`, extract `renderers/web_core/`,
  preserve headers file-by-file) — deferred, not attempted here.

## Capabilities

### New Capabilities

- `a2ui-core-vendoring`: the vendored, pinned, license-disclosed
  dependency on Google's `@a2ui/web_core` and `@a2ui/react`, and the
  workspace package structure that exposes them to later A2UI
  changes.

## Impact

- **New workspace packages:** `frontend/packages/a2ui-core/`,
  `frontend/packages/a2ui-react/`, both `pnpm` workspace members
  (matched by the existing `packages/*` glob in
  `frontend/pnpm-workspace.yaml`; no config change needed).
- **Dependencies added:** `@a2ui/web_core@0.10.4` (exact),
  `@a2ui/react@0.10.1` (exact), both Apache-2.0, both transitively
  pulled in via `pnpm install` (see `pnpm-lock.yaml` diff).
- **No product code changes.** Nothing in `frontend/src/` imports
  either new package yet — that begins with Change 17.
- **Lint scope:** the root `eslint.config.js` already ignores
  `packages/**` (workspace packages own their own lint/test
  lifecycle, matching the existing `prometheus-entity-management`
  convention), so `pnpm -C frontend lint` does not traverse the new
  packages. Each new package has its own `typecheck` script
  (`tsc --noEmit`), run and verified independently.

## Out of scope

- **Change 17** (`a2ui-uar-renderer-on-webcore`): the UAR-owned
  React renderer built on `@prometheus-ags/a2ui-core`. Not
  attempted here — this change only makes the upstream libraries
  available as pinned dependencies.
- **Full source vendoring** of `a2ui-project/a2ui`. Out of scope:
  the pinned-dependency-wrapper approach above is the right answer
  while no change modifies upstream internals; cloning and
  re-hosting the upstream TypeScript source is a materially larger,
  separate decision to make only when a real forcing function
  (a required upstream patch) appears.
- **Changes 18–21** (entity component migration, design-system
  bridge, realtime backbone, theming/a11y/i18n). Not attempted:
  each is independently scoped and sequenced after Change 17.
