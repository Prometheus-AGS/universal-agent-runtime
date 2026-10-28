# Source-backed startup and maintenance observers

These files are authored source, unimported and unrun. Verification is **CLOSED**. They are not execution authority. The uncomfortable constraint is that a genuine boot or retention claim cannot pass until the frozen fixture actually creates the isolated stores, binds the runtime to them, supplies real model credentials and preserves the worker's log receipt. Missing prerequisites fail closed; there are no configured success booleans.

`observers_startup.py` uses Python's standard library plus an explicitly named frozen `psql` executable for Postgres. Surreal uses its existing fixture's loopback HTTP `/sql` transport, following `scripts/release/uar_1_1_1_support.py` rather than opening or creating an embedded datastore handle. The adapter supports only the runner's `postgres_pgvector` and `surreal_disk` profiles. Surreal's actual disk backing remains the separate source-backed admission obligation; an HTTP endpoint by itself cannot prove disk durability.

## Frozen setup and scope

The prepare owner creates a fresh application schema/database and a private state directory before the runner launches its server. Compute the scope identically in preparation and the server wrapper:

`uar_acceptance_` followed by the first 20 lowercase hexadecimal characters of SHA256 of the exact UTF-8 `UAR_ACCEPTANCE_RUN_ID`.

Postgres must have a real schema with that exact name; its application connections must select that schema. Surreal must use that exact database within the configured namespace. Never point the observer at a user datastore. Preparation must not insert application records. The script inspects every base application table's row count, excluding Postgres migration metadata `_sqlx_migrations`, before the first runtime process starts. A nonempty application table aborts the observation. It never truncates tables, drops a schema/database, deletes an agent or manufactures an empty-store receipt.

Each backend supplies environment references through `environment_env`:

- `UAR_ACCEPTANCE_STARTUP_CONFIG`: absolute path of its frozen observer JSON configuration.
- The referenced config's `state_dir_env`: absolute backend-specific private state-parent directory. Preparation creates its derived scope subdirectory with mode0700. The observer requires this directory; it does not create a missing owned directory.
- Named Postgres connection or Surreal endpoint/username/password references. Credentials remain in the process environment and are never printed or passed as command-line arguments.

The config schema is:

```json
{
  "schema_version": 1,
  "state_dir_env": "UAR_ACCEPTANCE_STARTUP_STATE_PARENT",
  "command_seconds": 30,
  "maintenance_seconds": 60,
  "postgres": {
    "connection_env": "UAR_ACCEPTANCE_POSTGRES_CONNECTION",
    "schema_scope": "run_sha256_20",
    "psql_binary": "/absolute/frozen/path/to/psql"
  },
  "surreal": {
    "url_env": "UAR_ACCEPTANCE_SURREAL_HTTP_ORIGIN",
    "username_env": "UAR_ACCEPTANCE_SURREAL_USERNAME",
    "password_env": "UAR_ACCEPTANCE_SURREAL_PASSWORD",
    "namespace": "uar_acceptance",
    "database_scope": "run_sha256_20"
  }
}
```

Only the selected adapter's section is consumed. These illustrative executable paths are not assertions that those files exist. Resolve and freeze the real Python/psql executables, both observer source/config files, server/configuration/preparation inputs and their identities in task2.9. No installation, dependency download, database launch, schema creation or credential resolution is implicit in an observer invocation.

Set the real server's `sessions.persisted_ttl_secs` to a positive explicitly frozen duration, for example3600 seconds, to protect ordinary case history while admitting the one deliberately aged record. The opt-in source lives in `src/config.rs::SessionsConfig` and `RunManager::sweep_retained_state`. Do not use timing as a success predicate. Ensure info-level structured server logging retains the actual success marker described below. The core owns named private stdout/stderr capture files and exports their paths as `UAR_ACCEPTANCE_SERVER_STDOUT_PATH` and `UAR_ACCEPTANCE_SERVER_STDERR_PATH`; the observer never reads arbitrary operator log files.

## Observer argv bindings

For each backend bind the following frozen argv arrays using the actual absolute Python and script paths. Each action reads the existing `UAR_ACCEPTANCE_CONTEXT` object supplied by `Context.observe`; there is no shell expansion.

| Hook | Script positional action | Actual effect |
|---|---|---|
| `startup_initial_capture` | `capture` | Before first `ctx.start`, reads all application table counts and writes an exclusive private initial receipt; requires empty application data |
| `startup_initial` | `initial` | Reopens the same actual store, verifies identity against that receipt and requires at least five persisted seeded agent rows |
| `startup_maintenance` | `maintenance` | `values.action=prepare` ages exactly one real persisted session; `observe` reads durable absence and exact matching production worker success line |
| `startup_maintenance_read` | `read` | After a true runtime process restart, reads the same durable session partition and reports its actual row or null |

An argv array therefore has three entries: absolute Python executable, absolute `observers_startup.py`, and the action shown above. The core invokes `startup_initial_capture` after owned resource preparation and before first server launch. It is mandatory when U21 is selected: omitting capture leaves `startup_initial` without its exclusive receipt and fails. Other cases' generic readiness/admission/inference/session/cleanup observers still need their own real fixture implementations; these startup observers do not replace them.

## Real maintenance proof

The U21 case first runs a genuine useful prompt and takes the actual returned session UUID. It passes that UUID to the maintenance `prepare` action. The script reads that persisted row; it does not synthesize a serialized Session or create a row. It derives the storage partition from the actual persisted owner and logical ID, matching `session_storage_key` in `src/uar/persistence/mod.rs`, including the credential-principal prefix. Ambiguous or missing rows fail. The only product mutation sets that row's `last_activity` to `2000-01-01T00:00:00Z`, preserving its other data. SQL `RETURNING`/Surreal `RETURN AFTER` captures the actual mutation outcome atomically so a fast real retention worker cannot race away the fixture's before receipt.

The next genuine prompt produces real runtime activity, waking the production retention owner. The observation action never deletes the session. It requires both a durable absent row and a new log line containing the exact target session ID, actual owner ID and the production marker `persisted session purged by TTL sweep`. The successful delete branch in `src/uar/runtime/manager.rs` emits this metadata. Only a bounded line digest and closed metadata survive as its public receipt; private arbitrary logs and messages are not emitted. Polling is bounded and watches real state. Expired time, count-only logs, scripted deletion or a fixture's declaration cannot pass.

A later true process restart uses the same store and reads durable absence again. The case also proves the saved operator prompt remains useful before and after initialization/maintenance, checks exact saved built-in artifacts and restores its run-owned edit. Receipt and state files remain until the owned cleanup action confirms retention before teardown. Cleanup must include the backend-specific private state directory; this script never deletes a user path.

## Source-proven repairs and remaining limits

The inspected Postgres TTL query used an obsolete `owner::session` split while current keys are length-prefixed. It now reads actual persisted `data.owner_id` and `data.id` and admits the pair only if rebuilding `session_storage_key` matches the stored row key. Surreal already reads `owner_id`/`logical_id`. Startup seeding preserves existing complete artifacts, so deletion cannot be graded successful through later reseeding.

All SQL/API/log contracts above were authored from repository source, not executed. No adapter query, CLI invocation, process launch, import, test, compile or acceptance result has been observed for this new code. Database availability, executable paths, schema versions, physical durable backing, credentials and exact logs remain execution prerequisites. A transport or upstream failure emits only its type and a withheld-cause statement; this prevents credential-bearing database error strings from leaking and explicitly leaves that cause unrecovered. Closed author-owned predicate diagnostics remain readable.

## Shared read-only adapter API

`observers_runtime.py` may import this guarded module without executing its CLI. `run_scope(context)` derives the exact fixture schema/DB name; `session_storage_key(owner_id, session_id)` reproduces the UTF-8 byte-length-prefixed key and principal-session namespace. `Store(config, context)` requires the prepared scope already to exist and establishes actual storage identity. `Store.counts()` returns every application base-table count. `Store.session_data(owner_id, session_id)` reads the full Session payload by exact storage key and validates its actual owner and logical session ID; messages are available in the `messages` field. Surreal's `logical_id` is restored as logical `id` for callers. `Store.session(session_id, owner_id=None)` provides metadata only, with an optional exact owner filter; absent rows return null and ambiguous logical identities fail. These read helpers issue no writes. Only the explicitly invoked `Store.age` method mutates the one startup-maintenance-owned session.

Postgres queries use `sessions.id` as storage key and `sessions.data` JSONB as the Session payload. Surreal sessions hold Session fields directly with `logical_id` carrying the logical UUID. The reusable bounded query methods are `Store.pg_query(sql)` (one JSON result) and `Store.sql(sql)` (checked Surreal statement result). Preparation and cleanup remain owned by the runtime observer; they must not call the age action or write startup-maintenance records.
