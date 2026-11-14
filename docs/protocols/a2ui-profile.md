# UAR A2UI profile

RC5 targets final **A2UI 1.0** for the UAR API and bundled renderer. The dedicated
AG-UI route delivers ordered surface operations in ACTIVITY_SNAPSHOT events.

| Version field | Supported behavior |
| --- | --- |
| "1.0" | Inline components and initial data, component and data-model updates, deletion, and governed user actions |
| "v0.9", "v0.9.1" | Compatibility operations and retained history |
| "v1.0" | Release-candidate dialect refused |

For 1.0, use an approved catalog ID and {"@path":"/field"} for data bindings.
Inline initial data is available when components first bind. The supported Basic
Catalog ID is https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json.
UAR's custom catalog is urn:uar:a2ui:catalog:1. Both use UAR's approved component
subset. Remote function execution and agent-supplied catalogs are excluded.

Send the standard renderer version and action envelope in forwardedProps.a2uiAction.
For 1.0 actions, supply interaction_id and the host-published surface_generation
and keep both stable across
transport retries. The [logical action contract](../operations/a2ui-interaction-recovery.md)
defines receipt lookup, reconciliation, and refusal.

UAR uses PEM 5.0.0 for entity state. Node consumers can read the wire through
@prometheus-ags/a2ui-react/headless from 5.0.0. Flutter consumers use
entity_graph_flutter 5.0.0. RC5 qualification covers the UAR wire and its
Linux amd64 API deployment.

## Security boundary

A2UI crosses an agent-to-client trust boundary. UAR treats every message as
untrusted data and follows the protocol's catalog model: only pre-approved
component names resolve to native widgets. Validation fails closed and produces
a visible, non-executable diagnostic. Action payloads are typed data submitted
through the store/service boundary and remain subject to normal run ownership,
authorization, and server-side validation.

## Upstream references

- [A2UI specification versions](https://a2ui.org/)
- [Final 1.0 protocol](https://github.com/a2ui-project/a2ui/blob/d56f84f92a8478220d652af6ed7dd7abe5bfa14c/specification/v1_0/docs/a2ui_protocol.md)
- [v0.9.1 protocol family](https://a2ui.org/specification/v0.9-a2ui/)
- [Concepts and message lifecycle](https://a2ui.org/concepts/overview/)
- [Renderer development guide](https://a2ui.org/guides/renderer-development/)
- [Renderer support matrix](https://a2ui.org/reference/renderers/)
