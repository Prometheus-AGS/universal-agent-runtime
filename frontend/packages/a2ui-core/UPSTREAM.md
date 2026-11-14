# Upstream: `@a2ui/web_core`

`@prometheus-ags/a2ui-core` re-exports the A2UI web rendering and state-management
API from the workspace's Zod 4 source port. The source port retains the
upstream export paths. The workspace alias preserves the wrapper's existing
imports for the default, v0.8, and v0.9 entry points.

| Field | Value |
|---|---|
| Upstream source | [`@a2ui/web_core@0.14.0`](https://www.npmjs.com/package/@a2ui/web_core/v/0.14.0) |
| Workspace wrapper version | `0.10.4` |
| Exact compatibility dependency | `@prometheus-ags/a2ui-web-core-zod4@0.14.0-rc.1` |
| Source port | [`../../vendor/a2ui-web-core`](../../vendor/a2ui-web-core) |
| License | Apache-2.0 |

The port keeps A2UI catalog validation, schema conversion and binding on one
Zod major. This matters because passing a Zod 4 component schema into the
published Zod 3 web core can fail during catalog import, binding, or invalid
message reporting. The React reference wrapper is paired with the same core so
the cross-renderer comparisons can exercise the existing wire messages.

The `/v1_0` entry point exports the final wire API from
[`@a2ui/web_core@0.15.0`](https://www.npmjs.com/package/@a2ui/web_core/v/0.15.0).
Both consumer workspaces apply the
[PEM v5.0.0 compatibility patch](https://github.com/Prometheus-AGS/prometheus-entity-management/blob/b8397dab176bbbbfba22ff2ceba22a0be90dadf3/patches/%40a2ui__web_core%400.15.0.patch)
to the registry package. It directs the upstream library's Zod imports to
`zod/v3` within the pinned Zod 4.6.7 distribution. Use this entry point for
final protocol data. The UAR renderer and its existing component catalog share
native Zod 4 schema and state objects through the local source port.

Update the source port and its provenance together when moving the upstream
revision. Keep both workspace aliases and lockfiles aligned. At the phase
verification boundary, typecheck the wrapper and run the existing first-party
renderer, reference comparison, inspector and Lit/Svelte conformance targets.
Published independent client evidence stays bound to its separately recorded
package versions.

## License

Apache-2.0, © Google LLC. The full license text is preserved unmodified
in [`./LICENSE`](./LICENSE). The source port preserves upstream license and
copyright notices.
