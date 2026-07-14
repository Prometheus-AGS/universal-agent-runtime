# `@prometheus-ags/a2ui-core`

Vendored, version-pinned re-export of Google's
[`@a2ui/web_core`](https://www.npmjs.com/package/@a2ui/web_core) —
the A2UI project's core rendering / state-management library
(message processing, data/component/surface models, catalog types).

This package exists so UAR code imports a stable **internal** path
(`@prometheus-ags/a2ui-core`) instead of reaching into the upstream
package directly, while the actual upstream version is pinned and
bumped deliberately. See [`UPSTREAM.md`](./UPSTREAM.md) for the pinned
version, the rationale for pinning-over-copying, and the update
procedure.

## Usage

```ts
// Default (v0_8) surface, matching @a2ui/web_core's own default export.
import { MessageProcessor } from '@prometheus-ags/a2ui-core';

// v0.9 surface.
import { MessageProcessor } from '@prometheus-ags/a2ui-core/v0_9';
```

## Scope

This package is consumed by:
- Change 17, `a2ui-uar-renderer-on-webcore` — the UAR-owned React
  renderer built on this library (out of scope for this package).
- Change 22, `a2ui-inspector-lit-svelte-renderers` — Lit and Svelte
  renderers built on this library (out of scope for this package).

It does not itself render anything.

## License

Apache-2.0, © Google LLC (upstream). See [`LICENSE`](./LICENSE).
