# UAR A2UI Lit renderer

> **Current authority:** [A2UI product guide](/docs/product/a2ui). This private
> workspace package is a semantic-conformance renderer, not part of the React product UI.

`<a2ui-lit-surface>` renders a `web_core` `SurfaceModel` using UAR's approved
protocol catalog. Assign the model to its `surface` property. Unknown component
types fail closed. This renderer exists to compare protocol semantics across
frameworks; it does not establish the `server-full` or embedded-mobile product UI.
