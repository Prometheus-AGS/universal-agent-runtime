# `@prometheus-ags/a2ui-core`

> **Current authority:** [A2UI product guide](/docs/product/a2ui). This private
> workspace package is an internal protocol dependency, not a published SDK.

This package provides UAR's pinned internal import path for Google's
`@a2ui/web_core`. It exposes the upstream default surface and the `v0_9`
surface used by UAR renderers. It does not render components or define UAR's
approved catalog.

```ts
import { MessageProcessor } from "@prometheus-ags/a2ui-core";
import { MessageProcessor as MessageProcessorV09 } from "@prometheus-ags/a2ui-core/v0_9";
```

See [`UPSTREAM.md`](./UPSTREAM.md) for the exact upstream pin, update procedure,
and license provenance. Update the workspace package and lockfile together; do
not bypass this package with an unpinned direct import.

The package is `private: true`. Its version is workspace identity, not registry
availability. Upstream code remains Apache-2.0; see [`LICENSE`](./LICENSE).
