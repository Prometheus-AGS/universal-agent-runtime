# A2UI Inspector

Development-only React panel for A2UI message streams. Components consume hooks; hooks expose the Zustand store; the store owns validation and `web_core` processing; the injected service owns `EventSource` I/O.

“Freeze preview” freezes presentation, not ingestion. Buffered messages and the live connection remain visible and Resume applies the queue. History is bounded (500 by default), never persisted, and reports dropped items. Hosts should inject a redaction function into `createEventSourceService` before displaying secrets or private runtime data.

The `./storybook` entry exports stable addon/panel identifiers and the panel component. It intentionally does not install Storybook; the Change 25 host owns Storybook configuration and registration.
