# Graph-child continuation persistence review

Execute phase: `uar-post-lifecycle-verification-readiness`. Verification CLOSED. Source inspection and this authorized report write only; no tests, checks, builds, imports, SQL probes, scripts, network or runtime requests.

## Blocking finding

**P1 — Optional execution source breaks exact CAS for already persisted rows.**

`src/uar/persistence/client_tools.rs` adds `#[serde(default)] execution_source: Option<ClientToolExecutionSource>` without omitting absent values during serialization. A pre-change durable row has no `execution_source` member. It deserializes as None but serializes with `"execution_source":null`.

Both provider transition implementations compare the stored representation with the reserialized before row: PostgreSQL uses `WHERE key=$1 AND data=$3`, binding `serde_json::to_value(before)`; Surreal uses `WHERE data=$before`, binding `serde_json::to_string(before)`. The legacy stored data therefore cannot match the new before representation. An old pending continuation cannot acknowledge settlement or claim results after upgrade, despite the field's deserialization default. Completed replay does not need this transition and is not affected by this specific failure.

Preserve the old absent-field serialization for None (with the new field omitted), or implement an explicit safe compatibility/migration strategy. Keep Some execution sources persisted and immutable. Later authorized real-path evidence should resume a pending row written before this field existed, through both providers. No query was run to establish backend execution behavior; the mismatch follows directly from the inspected serialization and equality predicates.

## Source assessment of new binding

- The public continuation lookup key remains owner/tenant/root agent/batch; child source does not become caller authority. Current child artifact identity comes from the resolved executing turn, with executing run checked against its environment.
- `GraphCallerTurnCapture` has private fields and no Deserialize; its constructors are crate-private and source search found construction in admitted manager graph setup and graph delegation. ThreadService checks retained root run/artifact, resolves the selected current child, intersects current policy, and mints the actual child run before consuming the capture once.
- Child execution source validates nonempty/control-free IDs, distinct root/child runs, and presence of graph state. Full-row transition equality preserves execution_source through settlement, claim and completion.
- Graph handoff reads the authenticated root-scoped row and compares actual child agent/run/thread, exact pending call IDs, visible assistant role and text before publication.
- Shared settlement acknowledgment awaits parent Done and, when execution_source is present, child Done before writing source_settled. No child-only acknowledgment was found in this inspected path.
- Shared continuation validation and CAS still bind protocol, schemas, exact full result batch and visible predecessor; graph state remains host-sourced.

## Limits and handoff

Only this report was created; no production changes or guards added. The blocker was sent to lead. Compilation, provider compatibility, policy readmission, cancellation, real approval, cold graph continuation and durable settlement remain unverified. The uncomfortable limit is that deserialization compatibility alone does not preserve an existing exact-representation CAS contract.

## Corrective disposition

Re-read the final field annotation and both provider transition predicates. `execution_source` now uses `#[serde(default, skip_serializing_if = "Option::is_none")]`. A legacy absent member deserializes as None and is omitted again when the before row is serialized. The added field therefore no longer changes the PostgreSQL JSON value or inserts a new member into Surreal's serialized string for legacy rows. Remaining struct field ordering is unchanged by omitting this field. Some values still serialize and remain part of full-row immutable transition validation.

The reported P1 is resolved at source level. This appended disposition preserves the original finding and supersedes its blocking status. Neither provider equality predicate was weakened. No new blocker found in this correction. No serialization program, query, build or other verification executed; byte-level runtime compatibility and actual legacy continuation acceptance remain unverified pending the lead's authorized gate. Only this report was appended.
