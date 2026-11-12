# Persistence capabilities by store

`GET /api/uar/persistence/capabilities` reports, for the running instance, each
durable capability of its persistence store and where memory is kept:

```json
{
  "persistence": { "backend": "postgres", "capabilities": { "supports_durable_run_history": true, "...": false } },
  "memory": { "enabled": true, "store": "memory_service_surrealdb" }
}
```

Controller/list operations that need a capability the store lacks fail with HTTP 503 and
`{"error": {"code": "capability_unsupported", "capability": "supports_..."}}`.
Protected checkpoint resume uses its distinct actual HTTP503 error code
`checkpoint_resume_unsupported`; it does not rename that refusal to the
controller capability code. Neither path reports success. The separate `GET /api/uar/capabilities` document
describes protocol and service capabilities, not storage.

| Capability | SurrealDB | PostgreSQL |
|---|---|---|
| `supports_durable_tool_receipts` | yes | yes |
| `supports_durable_client_tools` | yes | yes |
| `supports_durable_mcp_elicitations` | yes | yes |
| `supports_durable_agui_runs` | yes | yes |
| `supports_durable_a2a_bindings` | yes | yes |
| `supports_durable_run_history` | yes | yes |
| `supports_durable_accounting` | yes | yes |
| `supports_durable_ingestion` | yes | yes |
| `supports_durable_approvals` | yes | yes |
| `supports_channel_observers` | yes | no |
| `supports_durable_observers` | yes | no |
| `supports_durable_agent_instances` | yes | no |
| `supports_durable_checkpoint_resume` | yes | no |

On PostgreSQL the four missing capabilities mean team execution, follow-ups,
schedules, observers and checkpoint resume are refused with
`capability_unsupported`. Team execution is SurrealDB-only by design for 1.4.

The table is the default for each store. The endpoint reports what the running
instance actually supports; read it rather than the table when they differ.

Memory is never stored by the persistence layer: both stores refuse memory
writes with `memory_service_required`, and the memory service keeps its own
SurrealDB store. On a PostgreSQL deployment, enable `memory` to get memory.

The selected current lane is Linux amd64 container APIs behind a trusted
host/gateway, embedded SurrealDB 4.0.0 on an owned SurrealKV volume by default,
and restricted PostgreSQL 18. Companion SurrealDB deployments use the 4.0.0
image pinned in [versions.toml](../../versions.toml) and persistent deployments
use SurrealKV. Companion deployment and restart behavior require their own
qualification on the new image. The [embedded upgrade qualification](embedded-surrealdb-4-upgrade.md#qualification-scope)
retains the recorded migration evidence. The [canonical recovery contract](../product-support-matrix.md#execution-and-recovery-guarantees)
and [final checklist](../releases/v1.4.0-final.md) distinguish acknowledgement,
retention, replay, effects and upgrade evidence. Automatic memory also requires
an explicit [operator relevance cutoff](memory-injection-provenance.md).
