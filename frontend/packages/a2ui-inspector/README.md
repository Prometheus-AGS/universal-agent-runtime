# A2UI Inspector

> **Current authority:** [A2UI testing guide](/docs/product/a2ui-testing). This
> private workspace package is a development inspector, not a customer runtime surface.

The inspector is a React panel for examining A2UI message streams. Components
consume hooks, hooks expose the Zustand store, the store owns validation and
`web_core` processing, and an injected service owns `EventSource` I/O.

Freeze pauses presentation, not ingestion. The panel keeps a bounded in-memory
history (500 messages by default), reports dropped messages, and does not
persist the stream. Hosts must inject redaction before displaying secrets or
private runtime data.

The `./storybook` export provides stable addon/panel identifiers and the panel
component. This package does not install or configure Storybook for a host.
