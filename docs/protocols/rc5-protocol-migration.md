# Migrating a UAR client to RC5

Use the dedicated AG-UI agent route with a 1.0 client. A new run declares
protocolVersion "1.0" on its first SSE event. Preserve the last SSE ID for a
reconnect and the original run and thread IDs for replay.

    {
      "version": "1.0",
      "createSurface": {
        "surfaceId": "ticket",
        "catalogId": "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json",
        "dataModel": { "title": "Review ticket" },
        "components": [
          { "id": "root", "component": "Text", "text": { "@path": "/title" } }
        ]
      }
    }

Use an approved catalog and send initial data and components together when
they form one surface. Replace legacy path binding keys with @path in 1.0
messages. Translate any release-candidate "v1.0" payload before sending it.
Existing v0.9 and v0.9.1 history replays in its original dialect.

For actions, retain the interaction ID and the host's surface generation
through retries. A retry must carry the same semantic input. Consult the
receipt when the original response is lost. Remote renderer and agent
function calls are outside this release profile.

Use the published PEM 5.0.0 packages for client entity state. The
@prometheus-ags/a2ui-react/headless entry point reads A2UI in Node without a
browser. Flutter consumers use entity_graph_flutter 5.0.0. The Linux amd64
acceptance exercises UAR itself and checks emitted messages with the headless
reader.

- [UAR AG-UI profile](ag-ui-profile.md)
- [UAR A2UI profile](a2ui-profile.md)
- [A2UI action recovery](../operations/a2ui-interaction-recovery.md)
- [PEM 5.0.0 migration guide](https://github.com/Prometheus-AGS/prometheus-entity-management/blob/v5.0.0/website/docs/operations/migration-5.md)
