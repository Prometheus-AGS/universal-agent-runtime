# `@prometheus-ags/a2ui-react` — REFERENCE IMPLEMENTATION ONLY

> **Do not import this package from UAR product code.** It exists purely
> as a pinned reference/cross-testing implementation. The UAR-owned A2UI
> renderer is `@prometheus-ags/a2ui-uar` (Change 17,
> `a2ui-uar-renderer-on-webcore`, out of scope for this package), built on
> [`@prometheus-ags/a2ui-core`](../a2ui-core).

Vendored, version-pinned re-export of Google's
[`@a2ui/react`](https://www.npmjs.com/package/@a2ui/react) — the A2UI
project's official React renderer (https://a2ui.org/, upstream repo
[`a2ui-project/a2ui`](https://github.com/a2ui-project/a2ui), directory
`renderers/react`).

## Pinned version

| Field | Value |
|---|---|
| Package | `@a2ui/react` |
| Pinned version | **0.10.1** (exact, no semver range) |
| License | Apache-2.0 |
| Confirmed via | `npm view @a2ui/react` on 2026-07-14 — `latest: 0.10.1`, published 3 weeks prior |
| npm tarball shasum | `a3525584427d1e4fec46af1413feb8e5e8cc7c8d` |

Update procedure and pinning rationale are identical to
[`../a2ui-core/UPSTREAM.md`](../a2ui-core/UPSTREAM.md) — this package is
pinned the same way, against `@a2ui/react` instead of `@a2ui/web_core`.

## Why this package exists at all

Change 17 (`a2ui-uar-renderer-on-webcore`) cross-tests the UAR-owned
renderer against `@a2ui/react` "as reference impl" (see the phase plan's
done condition for that change). Keeping a pinned copy in the workspace,
rather than an ad-hoc devDependency added later, makes that cross-testing
reproducible and keeps the reference version under the same update
discipline as `@prometheus-ags/a2ui-core`.

## License

Apache-2.0, © Google LLC (upstream). See [`LICENSE`](./LICENSE).
