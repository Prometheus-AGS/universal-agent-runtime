# Concrete runtime fixture observers

Implementation source only. The operator verification gate is **CLOSED**. Nothing in this document authorizes invoking an observer, importing it, starting a service, resolving a provider, inspecting Docker, or running any check. The example JSON has `executable=false` and is deliberately refused by the executable entry point. No runtime or acceptance result is claimed.

`observers_runtime.py` implements the owned lifecycle and actual observations required by the local user-outcome runner. It imports only the guarded startup observer's read adapter, ownership scope derivation, private receipt writer and closed failure type. Native request, provider, accounting, persistence and full-harness contracts were read from repository source. The implementation uses no configured success booleans as runtime evidence.

## Ownership and process contract

Each backend needs its own pre-existing absolute mode0700 parent, bound through `backend.environment_env.UAR_ACCEPTANCE_STATE_PARENT`. Runtime and startup configuration `state_dir_env` must reference that alias. Both observers derive `uar_acceptance_` plus SHA256(packet run ID UTF-8)[:20]. Prepare creates that directory exclusively, then creates mode0700 `home`, `tmp`, `ingest` and `surreal-data` children. A repeated or pre-existing scope is refused. The core sets HOME and TMPDIR to those children and owns stdout/stderr capture and all runtime/observer processes. Each backend must have a distinct parent; the packet run ID itself is shared across backend profiles.

Postgres preparation creates only the exact fresh scope schema within the explicitly supplied database. Its initial application-table count must be zero. Server connections use `options=-c search_path=<scope>,public`, preserving access to a pre-existing public pgvector extension. The observer's Store connections use only the owned scope and require that exact `current_schema()`. Runtime migrations and all application records must reside in the first search schema. Public extension access does not grant permission to modify public application records. Supply a dedicated acceptance database; an operator database is outside this fixture's approved scope. Preparation/cleanup never truncate a user table.

Surreal requires a fresh, explicitly prepared run-owned container. Preparation requires no pre-existing namespace, then defines the configured namespace and exact scope database and reads actual empty application counts. Its paired frozen lifecycle commands are operator-reviewed effects, never discovered or invented at execution. The cleanup command must remove only that scope container. The container's exact name, ownership label, digest-pinned image, daemon Args, loopback8000/tcp publication and retained writable bind mount are checked through actual frozen Docker inspection. The engine path must be `rocksdb:` or `surrealkv:` and fall under the exact run-owned bind mount. HTTP availability alone cannot prove disk durability.

The runtime wrapper renders a frozen JSON template, writes a private mode0600 config and exclusive PID launch receipt, changes cwd to the owned root, and replaces its process using the frozen native binary's repository-defined `--config <path>`. The core's Popen PID therefore remains the actual native runtime PID. Launch observations require that PID alive, the exact owned launch receipt, unchanged binary bytes and unchanged private rendered config. Readiness waits for actual `/healthz`, `/readyz` and a full-harness runtime epoch; cold restart keeps the same durable scope and receives a new launch receipt. This is process and source binding within a trusted local operator boundary, not OS remote attestation.

Launch also requires a frozen `mcp_template`, rendered as the owned cwd's private
`mcp.json` and retained unchanged across restart. Ambient MCP_CONFIG_PATH/DIR
cannot redirect it. The complete frozen `static_files` closure must include
`index.html` under the explicit `static_directory`; launch sets only that exact
UAR_STATIC_DIR. `team_execution_max_active` explicitly selects3–4 members rather
than inheriting ambient settings. Launch receipts retain these inputs and MCP
digest. `UAR_ACCEPTANCE_TOOLS_EFFECTS_ROOT` renders to the exact private
`<owned root>/tools-effects` leaf for MCP/native tool configuration.

Actual positive inference must also establish its owned genuine forwarding
chain. `budget_proxy.config_path` names the frozen shared private proxy config;
`provider_bindings` maps each registered provider to its exact model listener.
The runner supplies its actual budget process PID/nonce/input identity. The
observer reads that owner's private SQLite ledger read-only, requires completed
successful requests to the approved upstream/model and checks actual serialized
run wire hashes when available. Helper/graph calls without those hashes are
explicitly labeled owned-binding plus settled-accounting evidence, not exact
per-helper wire correlation. The special structured-format metadata listener
must be invocation-owned, forward into the same budget listener, and retain
matching request/response byte hashes across both hops. An arbitrary loopback
endpoint cannot establish genuine inference.

## Configuration and freezing

`fixtures/runtime-observer.example.json` describes one runtime config shape. Author separate PG and Surreal configs with the same global source/configuration/schema inventories and the selected backend's own `server_template`. Replace all example paths, origins and migration inventory entries before freezing. No example executable path asserts that a program is installed.

Freeze the real resolved Python, psql, Docker and native executables; the runner and case modules; both observer sources; all runtime/startup configs and JSON templates; and all source/schema files that contribute to the identities. Every command's absolute file argument and executable must be in `fixture.frozen_files` with actual bytes. Keep the runtime configs' following inventories identical across backends:

- `source_files`: the complete selected production/source denominator, sorted by path for hashing.
- `configuration_files`: both runtime configs, both server templates, startup configs and any other declared configuration inputs. Include the selected template. Hash each input's actual frozen bytes, then hash the canonical sorted `{path,sha256}` inventory. This avoids incompatible per-backend template hashes in the fixture-global `configuration_revision`.
- `schema_contract`: both adapters' required table/column maps and the complete top-level SQLx migration source list. `schema_revision` is SHA256 of that canonical object; installed PG versions and SHA384 source checksums must match the complete list exactly. Surreal reads actual DB/table definitions for the declared tables and fields. SCHEMALESS tables may declare an empty field list; that proves table presence rather than field constraints.

`source_identity` is SHA256 of the canonical sorted `{path,sha256}` source inventory; `artifact_identity` hashes actual native binary bytes. Digests use the `sha256:` prefix. Canonical JSON is Python `json.dumps(sort_keys=True,separators=(",",":"),allow_nan=False).encode()`. The selected rendered config has a separate actual launch digest. The fixture carries the global identities. The later whole-phase freeze owner must derive them from the final source; this document supplies no current digest or execution permission.

Template substitutions occupy a whole JSON leaf or argv element: `@scope@` or `@env:NAME@`. Partial interpolation is refused. The observer exports these derived values for whole-leaf substitution:

| Name | Value |
| --- | --- |
| `UAR_ACCEPTANCE_OWNED_SCOPE` | exact scope/schema/database/container name |
| `UAR_ACCEPTANCE_OWNED_LABEL` | `uar.acceptance.scope=<scope>` |
| `UAR_ACCEPTANCE_OWNED_STATE_DIR` | exact private root |
| `UAR_ACCEPTANCE_SURREAL_VOLUME` | `<root>/surreal-data:/data` |
| `UAR_ACCEPTANCE_MEMORY_PATH` | `<root>/memory` |

The generic startup config uses the same PostgreSQL `schema_scope=run_sha256_20`, Surreal `database_scope=run_sha256_20`, connection/credential references and namespace. Startup and runtime configs must identify the same stores. Parent credentials are explicit backend environment references or fixture `secret_env`/binding `credential_env`; the core removes ambient environment discovery and passes only the approved references. Bind `UAR_ACCEPTANCE_RUNTIME_CONFIG` and `UAR_ACCEPTANCE_STARTUP_CONFIG` explicitly. Freeze named provider credentials without printing them. `server_environment_env` is optional and may not override UAR, persistence, port, auth or legacy config selection. Put production settings in the frozen JSON template. Surreal's durability attestation is set only after actual disk inspection; the configured attestation alone is never proof.

Templates must select `persistence.provider=postgres` or `surreal` respectively. The wrapper replaces the datastore URL and Surreal root authentication/namespace/database from named fixture references, and sets `server.ingest_directory` within its private root. Enabled embedded memory must have the exact scope namespace/database and a private child `db_path`; remote memory is refused by this fixture. Explicitly configure all required runtime features, model paths, security, info-level maintenance logs and positive persisted-session retention in the templates. Avoid uncontrolled defaults requiring model downloads. Do not put secrets in frozen templates; generated configs are private and removed during cleanup.

For `store_lifecycle.surreal_disk`, the future frozen `prepare_argv` must launch the pinned image with exact name `@scope@`, label `@env:UAR_ACCEPTANCE_OWNED_LABEL@`, bind mount `@env:UAR_ACCEPTANCE_SURREAL_VOLUME@`, and explicit loopback host port matching `surreal.url_env`. Its daemon arguments must equal the configured `disk_container.args` after rendering. The source-backed example Args follow `scripts/release/uar_1_1_acceptance.py::start_surreal` and the image pin in `versions.toml`; they change the retained engine path to `/data/runtime.db`. Resolve and freeze the actual lifecycle commands during task2.9; the empty example arrays cannot execute. Docker may require daemon credentials in Args; those are private command inputs, never public receipts. Named secret values and inspect output remain private/redacted. The observer stores only inspect byte identity and closed disk metadata.

## Hook bindings and actual proof

Each binding is `[ABSOLUTE_FROZEN_PYTHON, ABSOLUTE_OBSERVERS_RUNTIME_PY, ACTION]`. Use `serve` for `backend.server.argv`; use the other six actions for the matching observer hooks. The core supplies `UAR_ACCEPTANCE_CONTEXT`, `UAR_ACCEPTANCE_FIXTURE_PATH`, server PID and stdout/stderr paths. `serve` reads the same context; there is no shell. Configs require executable schema1 only after the freeze owner supplies all inputs.

| Action | Observed effect or evidence |
| --- | --- |
| `prepare` | exclusive ownership marker; actual empty fresh schema/DB; real retained disk backing for Surreal |
| `ready` | alive owned runtime PID, unchanged launch/config/binary, actual ready HTTP statuses and epoch |
| `admission` | actual frozen identity inventory, same initial store, migration/table contract, live persistence endpoint and seeded agent rows; actual pgvector or Docker disk proof |
| `inference` | terminal persisted primary run/session/model; actual enabled provider endpoint and current verification; correlated settled usage subgroup |
| `session` | exact durable owner-qualified storage key and canonical persisted messages; actual current runtime epoch |
| `cleanup` | recorded runtime PIDs absent; bounded redacted typed job metadata retained before exact-owned schema/DB/container/private state teardown; absence confirmed |

Inference accepts `run_id`, `session_id`, `qualified_model`, optional `primary_qualified_model`, `role=primary|auxiliary` and optional nonempty `call_ids`. Final run inspection is compared with the primary binding. Actual accounting subgroup admission models are compared with `qualified_model`, and every requested call must exist, be settled and carry nonzero observed tokens and reconciliation evidence. The current provider view must match the frozen genuine binding's ID, protocol, endpoint and enabled state and expose the requested enabled model and current configuration verification. Its actual endpoint origin must be listed in `approved_genuine_origins` for that provider. No synthetic positive is accepted.

The uncomfortable limit is that an operator-controlled fake endpoint can reproduce model text and usage receipts. Frozen operator approval, current provider verification and correlated real dispatch receipts prove contact with the declared origin; they cannot cryptographically establish the upstream operator's identity. This boundary must remain explicit in the final evidence. Do not relabel it as externally attested genuine service.

Cleanup runs only after the core joins its owned processes and independently refuses any launch-receipt PID still alive. Before dropping state, it emits and saves bounded redacted counts and up to100 typed ingestion-job rows per application/principal table, with omitted-row counts and truncated error detail. Surreal serialized job JSON is decoded before selecting fields. Source bytes, full arbitrary logs and private reasoning are never copied. A failure to capture metadata blocks destructive teardown. Successful cleanup drops only the exact owned schema/DB, executes the frozen paired lifecycle, checks exact container absence, then deletes only the exclusively created private root. Parent directories, user stores and older evidence remain. Core evidence retains the returned before-drop receipt. An upstream error emits its type and withheld-cause statement or private output digests, never raw credentials.

All behavior above remains unexecuted under the CLOSED gate: no imports, Python checks, tests, formatting, Docker commands, SQL, service launch or provider requests were run while authoring these files. Missing installed inputs, schema/API compatibility, command behavior, bounded cleanup and backend parity require the later exact operator-approved integration packet.
