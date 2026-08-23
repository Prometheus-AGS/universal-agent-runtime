# UAR A2UI Svelte renderer

> **Current authority:** [A2UI product guide](/docs/product/a2ui). This private
> workspace package is a semantic-conformance renderer, not part of the React product UI.

`A2uiSvelteSurface` renders a `web_core` `SurfaceModel` using UAR's approved
protocol catalog. Pass the model as `surface`. Unknown component types fail
closed. This renderer exists to compare protocol semantics across frameworks;
it does not establish the `server-full` or embedded-mobile product UI.
