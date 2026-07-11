# UAR A2UI profile

UAR 1.0 supports **A2UI v0.9.1** through the stable profile
`uar.a2ui/1`. Upstream describes v0.9.1 as the current production release; it
standardizes `application/a2ui+json` and uses these message types:

- `createSurface`
- `updateComponents`
- `updateDataModel`
- `deleteSurface`

The supported catalog is UAR-owned and maps declarative A2UI components to
local React components. Agent messages cannot supply React code, HTML,
JavaScript, CSS, event handlers, or component module URLs. Unknown components,
properties, bindings, references, and actions are rejected before rendering.

The initial approved catalog is `Text`, `Button`, `TextField`, `CheckBox`,
`ChoicePicker`, `Row`, `Column`, `Card`, and `Divider`. It deliberately excludes
remote media and arbitrary rich-content components until their URL, content,
privacy, and accessibility policies are separately certified.

## Version policy

| Protocol | UAR status | Compatibility |
|---|---|---|
| A2UI v0.9.1 | GA via `uar.a2ui/1` | Covered by the UAR 1.x compatibility promise |
| A2UI v1.0 | Experimental candidate | Explicit opt-in only; no UAR 1.x stability guarantee |
| Legacy UAR artifact forms | Transitional | Accepted while callers migrate to the shared renderer |

The v1.0 candidate adds client-to-server `actionResponse`, action IDs, and
renames theme configuration to `surfaceProperties`. UAR does not infer those
semantics for a v0.9.1 surface. Candidate messages must declare their version
and use an experimental profile so a future upstream change cannot silently
change production behavior.

## Security boundary

A2UI crosses an agent-to-client trust boundary. UAR treats every message as
untrusted data and follows the protocol's catalog model: only pre-approved
component names resolve to native widgets. Validation fails closed and produces
a visible, non-executable diagnostic. Action payloads are typed data submitted
through the store/service boundary and remain subject to normal run ownership,
authorization, and server-side validation.

## Upstream references

- [A2UI specification versions](https://a2ui.org/)
- [v0.9.1 protocol family](https://a2ui.org/specification/v0.9-a2ui/)
- [Concepts and message lifecycle](https://a2ui.org/concepts/overview/)
- [Renderer development guide](https://a2ui.org/guides/renderer-development/)
- [Renderer support matrix](https://a2ui.org/reference/renderers/)
