# `@prometheus-ags/a2ui-react` — reference implementation only

> **Current authority:** [A2UI product guide](/docs/product/a2ui). This private
> workspace package is a pinned reference renderer and is not imported by UAR product code.

This package provides a version-pinned internal path for Google's official
`@a2ui/react` renderer. UAR product code imports
`@prometheus-ags/a2ui-uar`; this package exists only for comparison and
cross-framework conformance fixtures.

| Field | Value |
|---|---|
| Upstream package | `@a2ui/react` |
| Workspace wrapper version | `0.10.1` |
| Exact upstream dependency | `@a2ui/react@0.10.2` |
| Workspace publication | `private: true` |
| License | Apache-2.0 |

Exports are `.` for the upstream default surface, `./v0_9` for the comparable
protocol surface, and `./styles` for upstream styles. Follow
[`../a2ui-core/UPSTREAM.md`](../a2ui-core/UPSTREAM.md) when updating the pin.
