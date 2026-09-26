# Protocol sources and document validation boundary

The [AG-UI](ag-ui.md), [A2UI](a2ui.md) and [A2A](a2a.md) documents define proposed adapters over one UAR task service. Their new UAR semantics are official draft contracts; ordinary upstream wire objects retain the upstream field names and meaning.

[source-pins.json](source-pins.json) records exact source URLs/revisions, byte hashes, retrieval dates and evidence classification. Files in [upstream/](upstream/) are unmodified documentation fixtures, isolated from runtime generation and dependency resolution. Their accompanying licenses remain included. None installs or upgrades an SDK.

| Boundary | Available source for the completed-document gate | Limits |
|---|---|---|
| AG-UI event frame | [Pinned upstream JSON Schema](upstream/ag-ui.schema.json), root `$ref` to Event, from commit b8ebd02c84a3a2757990da47aebf7c55b708b1ef | Shape validation can cover standard/subagent/CUSTOM events; cannot establish ordering, service behavior or SDK compatibility |
| UAR custom payload | [custom-event.schema.json](../schemas/custom-event.schema.json) | Apply after standard CUSTOM frame validation; owner authorization/recovery remain runtime obligations |
| UAR surface action | [surface-action.schema.json](../schemas/surface-action.schema.json) | Proposed host wrapper; not an upstream v1 action |
| A2A body | [v1.0.1 published generated schema](upstream/a2a.schema.json), appropriate `$defs` for SendMessageRequest/StreamResponse/Task | Generated artifact is explicitly non-normative; release-tagged a2a.proto and binding text govern disagreements. JSON-RPC wrapper is a separate transport envelope |
| A2UI inner message | UAR revision a54dd9591dfeaff8694252b69d2d40f0f28e2250, src/uar/a2ui/protocol.rs, plus [existing profile](../../../../protocols/a2ui-profile.md) | No independent exact UAR-profile JSON Schema fixture is claimed here. Compare preserved field shapes statically now; actual profile parser/rendering acceptance belongs to I3 |
| Trace wrapper | [trace-row.schema.json](../schemas/trace-row.schema.json) | Explanatory internal rows are deliberately not wire or persisted-record schemas |

No runtime conformance follows from document validation. A semantic assertion such as "fresh approval recorded" or "no second issue created" in a trace needs a real integration scenario after implementation. The entire artifact set receives its single completed-document validation gate under the coordinating parent; the protocol author did not run independent partial validators.

Context7 supplied initial schema pointers. Direct source inspection corrected its stale TypeScript event-file path after upstream reorganized the core source and avoided copying older A2A examples with obsolete role/state spellings. This illustrates why source provenance and released contracts are kept distinct from documentation search results.
