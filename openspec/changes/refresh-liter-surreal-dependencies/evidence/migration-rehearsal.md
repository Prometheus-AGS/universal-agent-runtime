# Disposable SurrealDB 2.x to 3.x migration rehearsal

Date: 2026-08-28

The rehearsal used only loopback listeners and in-memory disposable databases:

- Source: SurrealDB 2.6.5 on `127.0.0.1:28921`
- Target: SurrealDB 3.2.4 (`3.2.4+20260803.93ab219`) on `127.0.0.1:28922`
- Namespace/database: `migration/rehearsal`
- Production data and LaunchAgents: untouched

The v2 source was seeded with two `person` records and one `knows` relation.
The source count was 2. The official compatibility export path completed:

```text
surreal v2 export --v3 --endpoint http://127.0.0.1:28921 \
  --username root --password <disposable-password> \
  --namespace migration --database rehearsal v2-for-v3.surql
```

The export was 1,112 bytes with SHA-256
`d2353dee2263ed1753f59060a4b2a76f753af17a58b9497be02cb5bf6e7257f1`.
It was imported through:

```text
surreal import --endpoint http://127.0.0.1:28922 \
  --username root --password <disposable-password> \
  --namespace migration --database rehearsal v2-for-v3.surql
```

Post-import verification returned both source records, preserved their scalar
and array fields, preserved the `person:alice -> knows -> person:bob` relation
with `since: 2026`, and returned a target count of 2.

The uncomfortable boundary is explicit: this proves the current CLI path and
representative record/relation conversion only. It does not prove that an
arbitrary production schema needs no manual correction; operators must still
review every migration diagnostic and compare production-specific counts and
queries before cutover.
