## Context

Phase `uar-uiux-architecture-2026-11`, build items 7, 8, 9; unresolved review finding
critical 4; scenario area CFG. All claims below were read at the cited line in this
session unless marked UNVERIFIED. Nothing was built or run.

Source facts this design stands on:

- `ConfigManager::load` and `load_without_watcher` call `AppConfig::load_with_cli`, then
  `vault::resolve` (feature `vault`), then `config.security.validate()`
  (`src/config_manager.rs:42-75`). `reload` repeats the same three steps and swaps an
  `ArcSwap` (`:103-130`). A failed reload keeps the current config.
- `AppConfig::load_with_cli` (`src/config.rs:1741`) layers compiled defaults, one YAML file
  (explicit `--config`, else `./config.yaml`, else `~/.uar/config.yaml`; `:1829-1840`), CLI
  overrides, legacy `LLM_*` variables, `UAR_*` variables and provider-key shortcut variables
  (`:1937-1990`). It then runs `security.validate()`, `memory.validate_injection()`,
  `a2a.validate()`, `service_instance.validate()` and `project_instructions.validate()`
  (`:2012-2030`). Env, CLI and shortcut secrets are merged into the same `AppConfig`.
- `AppConfig` and `SecurityConfig` derive `Deserialize` only (`:243`, `:711`); there is no
  serializer for them, so the only faithful on-disk form of a loaded config is its source
  bytes.
- `SecurityConfig::validate` refuses: reserved delegable roles, mismatched principal
  triples, a JWT algorithm that does not match the verifier, the built-in fallback JWT
  secret when `jwt_required` is true, an empty admin key when
  `settings_mutation_auth_required` is true, and several remote-profile requirements
  (`src/config.rs:829-962`). `start_server` re-runs `validate_identity_policy` on the
  boot snapshot (`src/server.rs:491`).
- Boot reads `llm.embedding.*` rows with `get_typed(..)?` and an `ensure!` on the dimension
  (`src/server.rs:1244-1267`). `get_typed` returns `Err` only when the stored JSON fails to
  decode into the requested type (`src/uar/settings/manager.rs:492-497`), because
  `get_value` swallows storage errors (`:468-489`).
- `load_provider_configs_from_db` and `protect_provider_credentials` return on the first
  undecodable `provider.*` row (`manager.rs:1168-1187`, `:1190-1217`); the caller logs and
  continues without hydrating anything (`server.rs:1173-1180`).
- Ownership is reconstructed from `updated_at.is_some()` (`manager.rs:274-276`). The
  provider seed writes `updated_at: Some(now)` (`:997`) and skips any existing row
  (`:976-978`). The Postgres `source` column mirrors the same rule (migration
  `20261030010000_settings_source.sql`).
- Non-restart keys are written to the "effective" map on set (`manager.rs:666-671`);
  `setting_apply_mode` classes `governance` as Live, `server|security|persistence` as
  Restart, everything else NextTurn (`manager.rs:73-80`).
- The reload endpoint returns the JSON schema, not a result (`server.rs:3726-3752`), and no
  frontend file calls it (searched `frontend/src`; external SDK use is UNVERIFIED).

## Goals / Non-Goals

Goals:

1. A saved, validated copy of the YAML that booted, usable for a fail-closed boot fallback.
2. Zero chance that the fallback boots a weaker security posture than the operator last
   deliberately accepted, beyond the one residual named in Risks.
3. Persisted bad values never prevent boot; the operator sees exactly which value was ignored.
4. Hot reload applies what is safe to apply, DB wins for operator edits, and the response
   says what happened.
5. The provider seed implements decision 1 (DB wins) instead of skipping existing rows.

Non-goals: console rendering; deleting stored rows; changing `llm.default_provider`
semantics; the vault; editing pins.

## Decisions

### D1. What is saved: source bytes plus a digest record, nothing else

On a boot that reaches readiness from the primary file, save to the LKG directory:

- `config.yaml` — the exact bytes `load` read, written once from the same read buffer that
  fed the parser (no second read, so no time-of-check gap);
- `config.yaml.lkg.json` — `{version: 1, sha256, saved_at (RFC 3339), source_path (display
  only), format, binary_version}`.

Never saved: environment values, CLI overrides, defaulted values, vault output, provider-key
shortcut values, any `AppConfig` field. Because `AppConfig` has no serializer this is also the
only possible design; adding `Serialize` to `AppConfig` is rejected (it would put
`jwt_secret` and `llm.api_key` one `#[derive]` away from disk).

Honest limit: if the operator wrote a secret into the YAML, that secret is duplicated in the
LKG copy. It already sits on disk in the same trust domain. The copy is written `0600` in a
`0700` directory to avoid widening access. Windows ACL handling is best effort (Windows is
Experimental per `.claude/rules/rust.md`).

Location: `$UAR_LAST_KNOWN_GOOD_DIR`, else `~/.uar/last-known-good/` (the directory that
already holds `~/.uar/config.yaml`, `config.rs:1837-1840`, and `~/.uar/run`,
`process_supervisor.rs:184`). Containers set the variable to a path on the existing `data`
volume (`k8s/base/uar-deployment.yaml:63-65`). A config file path that does not resolve to
an existing regular file (environment-only configuration) has nothing to save, and boot
falls through with no fallback.

Write protocol: temporary file in the same directory, owner-only mode, `fsync`, rename over
the target, `fsync` of the directory. A torn write is detectable because the record digest
must equal the SHA-256 of `config.yaml`; a mismatch means "no LKG".

When saved: once, after the boot readiness checkpoint (the point at which `start_server` has
bound its listener and marked its scopes ready; exact line chosen at implementation), and
only when the booted source was the primary file. A process that booted from the LKG never
re-saves it. A successful hot reload does not save: a reload that changes a restart-class key
has not yet been proved bootable (see Risks).

Save failure (read-only filesystem, no space) is a `warn!` with name
`config.lkg.save_failed` and an entry in boot status. It never blocks boot.

### D2. Fallback eligibility is a closed list

Replace the single fallible `load_with_cli` call with two stages that return typed
failures:

| Stage | Contents | On failure |
|---|---|---|
| Read | read the resolved config file bytes | `LoadFailure::SourceRead` (eligible) |
| Layers | builder layers + `try_deserialize` + the derived defaults at `config.rs:1993-2008` | `LoadFailure::SourceParse` (eligible) |
| Validate | every check at `config.rs:2012-2030`, vault resolution, `config_manager.rs` `security.validate()` | `LoadFailure::Refusal` (never eligible) |

`start_server`'s `validate_identity_policy` (`server.rs:491`) runs after load and applies to
whichever config booted, so a fallback config is checked by it too.

Fallback runs only when **all** hold:

1. the failure is `SourceRead` or `SourceParse` on an existing file (a missing file is an
   operator path error and never falls back);
2. the invocation is the server run (`command` is `None`), not a one-shot command (`main.rs:92`);
3. neither `--strict-config` nor `UAR_STRICT_CONFIG=true` is set;
4. `UAR_CONFIG_FALLBACK` is not `off`;
5. a digest-valid LKG exists;
6. a second load that substitutes the LKG bytes for the file — same CLI, same environment,
   same vault — passes **all three stages**; otherwise the original error is returned
   unchanged;
7. the second load's `security.is_remote()` is false. A remote deployment profile never
   boots from the fallback: it runs under a supervisor where a crash loop is visible, and it
   is where a silent weaker posture costs the most.

Rationale for (6): because the environment and CLI layers are identical, a second success
proves the difference was the file bytes, so the failure was attributable to the YAML.
Rationale for the closed `Refusal` class: the review finding is that a tightened security
config which fails must not boot on the older, weaker one. A refusal is detected by stage,
not by message matching, so a new refusal added to `validate` is excluded automatically.

Unchanged and deliberately outside fallback: governance bootstrap fails closed with
governance enabled and mutation unavailable (`server.rs:93-103`); the local-only provider
audit bails startup (`server.rs:1189-1220`); credential policy errors bail
(`server.rs:1097`).

Substitution mechanism: `AppConfig::load_with_cli` gains a variant taking
`ConfigFileSource::{Resolved, Bytes{bytes, format}}`; the `Bytes` arm uses
`config::File::from_str`. Env and CLI layering is untouched.

Visibility: on fallback, `error!(name: "config.boot.fallback_active", ..)` carries the
failure class, the YAML key path when the parser reports one, the failed source path (the
configured file's path, so the console can name the rejected file), the failed source digest, the
LKG digest and its saved time. It never carries a parsed value or raw parser text, because
parser messages can quote the offending value (for example a typed field holding a
secret). The file watcher keeps watching the primary path; the first valid change reloads it
and flips status to `fallback_cleared_by_reload`.

### D3. Tolerant persisted values

Embedding (`server.rs:1244-1267`): each of the four reads (`backend`, `model`,
`vector_dimension`, `send_dimensions`) is evaluated independently. A decode error or an
out-of-range dimension skips that one value, keeps the config-built value for it, and
records `{key, reason: invalid_type | out_of_range}`. The rest of the overlay logic is
unchanged, including the existing keep-config-built-backend branches at `:1307-1325`.
Because the dimension decides the vector space, a skipped dimension leaves the config-built
dimension; the existing embedding retire/recover routes
(`api/settings.rs:/embedding/retire-primary`, `/embedding/recover`) remain the operator path
for a real space change.

Providers: `load_provider_configs_from_db` returns decoded configs plus a list of skipped
`{key, reason: undecodable}` and does not fail the batch. `protect_provider_credentials`
skips an undecodable row the same way and does not touch it. The registry keeps whatever the
config seeded for that provider id. Rows are never rewritten or deleted by this path.

Both lists are reported by boot status and by one `warn!(name:
"settings.persisted_value.ignored")` per entry with the key and reason only.

### D4. One decision function for boot and reload

Extract the per-key branch of `initialize_with_governance_default`
(`manager.rs:241-327`) into one pure function:

`decide(existing: Option<&Settings>, desired: &Settings, cached_source) -> Insert | OverwriteConfigOwned | KeepWithDrift | Unchanged`

Boot and reload both call it, so they cannot diverge. Ownership stays `updated_at.is_some()`
(and `cached_source == Api`), which is the rule already documented at `manager.rs:268-274`.

Reload reconcile (`SettingsManager::reconcile_from_config(&AppConfig) -> ReconcileReport`):

- touches only keys present in the freshly built desired set; never deletes a row;
- skips every `governance.*` key. Governance posture has its own post-commit publication
  rule (`.prometheus/gotchas.md`, 2026-08-28, "Database-live settings events can race
  runtime publication"); a reload must not alter it;
- merges per key into the cache (boot replaces the whole map; reload must not drop
  `provider.*`, `skill_config.*` or other keys);
- writes config-owned rows with `updated_at: None` so a reload never promotes a config value
  to "operator-edited";
- updates the "effective" map only for keys the settings change's class table
  (`uiux-settings-truthful-status-and-removals` D1/D3) marks `live` or `next_turn`; `restart`
  keys are written to the DB and reported as `restart_required`. For an `inert` key the report
  distinguishes by where the table says the value is read. A key read from YAML at boot (for
  example `server.port`) that is edited **in the YAML** is reported `restart_required`, because
  the next start applies it. A key with no reader anywhere (for example `vision.auto_detect`)
  edited in the YAML is reported `not_applied`. The same YAML-at-boot key edited **in the
  database** through the settings API is `not_applied` (the settings change's rule); the
  reload report never says "in effect" for either. A key the table does not list is treated as
  `restart`, the table's default. This replaces the current
  rule at `manager.rs:666-671` (everything except `server|security|persistence`);
- returns `{applied: [{key, apply}], retained_with_drift: [{key}], restart_required: [{key}],
  not_applied: [{key}], skipped: [{key, reason}], unchanged: n}`. Keys only; never values.

Concurrent reconciles (file watcher plus the explicit route) are serialized by one async
mutex. With several replicas each reading the same YAML, reconciles are idempotent: a write
happens only when `decide` returns Insert or OverwriteConfigOwned, so a second replica finds
nothing to do (compare-before-write; a lost race writes the same value twice).

### D5. Provider reconcile (build item 9)

Replace `seed_providers_from_registry` with `reconcile_providers(desired, preserve_api_key)`.
`desired` comes from a scratch `ProviderRegistry` seeded through the same calls boot uses
(`seed_from_llm_config`, `seed_from_configs`; task 3.2 confirms they have no side effect
outside the scratch registry). For each desired provider P with stored row R:

| Case | Action |
|---|---|
| R absent | insert, config-owned (`updated_at: None`) |
| R config-owned | overwrite from P when the data differs (credential handling unchanged: the row holds an API key only when no protected store exists, `manager.rs:985-991`) |
| R operator-edited | keep R. Append models of P whose `id` is absent from R and not already in R's *offered set* (below). Record drift for every other differing top-level field, by field name only |

*Offered set.* The risk of an additive merge is resurrecting a model an operator deleted: the
YAML still lists it on every boot. The row therefore remembers which YAML model ids were
already offered to it; only ids never offered are merged. Storage of the offered set is a
decision for implementation with two candidates, neither verified: an extra field inside the
row's `data` (UNVERIFIED whether `ProviderConfig` decoding tolerates unknown fields), or an
`internal.*` settings row following the `internal.model-verification.*` precedent
(`manager.rs:608`; UNVERIFIED whether the `internal` type exists for the Postgres
foreign key). Task 3.3 settles this by reading both before choosing. The behavior is fixed
either way: an operator-deleted model is not re-added by a restart or reload.

A merge write bumps `updated_at` (the row is still operator-owned and the revision token
`setting_revision`, `manager.rs:84-89`, must change so an in-flight edit sees a conflict).

Default provider: `get_default_provider_id().is_none()` seeding stays (`manager.rs:1018-1026`).
A YAML default that differs from the stored default is reported as drift on
`llm.default_provider` and cannot be resolved through the drift route (D6); the only
controls remain the live default route and the system-model change.

No data migration. Rows already stamped by the old seed look operator-owned and therefore
stay DB-wins: the conservative reading. An operator who wants YAML to flow into such a row
resolves its drift explicitly.

### D6. Drift visibility and resolution

`GET /api/uar/settings/drift` (`api/settings.rs:1239`) keeps its entries and gains, per
entry: `config_value` (through the same masking as the stored value),
`detected_at`, `detected_by: boot | reload`. Provider drift entries carry `fields: [name]`
and no values.

`POST /api/uar/settings/drift/{key}/resolve`, body `{"resolution":"use_config"}`, admin-key
guarded like `/drift` (`require_admin_key`). Writes the latest reconcile's desired value as a
config-owned row, clears drift, returns `{key, revision}`. `404` when no drift exists for the
key; `409` for `llm.default_provider` and for `governance.*`. "Keep the stored value" is a
no-op and is not persisted: the entry stays listed until the YAML or the row changes.
Persisting an acknowledgement is rejected for now (no named failure; it adds a settings
type).

### D7. Reload report and routes

- `POST /api/uar/settings/reload` (admin-key guarded) runs `ConfigManager::reload` then the
  reconcilers, and returns `{status: "applied" | "unchanged" | "rejected", reloaded_at,
  applied, retained_with_drift, restart_required, not_applied, skipped}` using D4's report plus the
  provider report. A rejected reload (parse, refusal, strict conflict) returns `422` with
  `{status: "rejected", class}` and no writes occurred.
- `POST /.well-known/uar-config/reload` keeps its body (the schema) and now also performs
  the reconcile. Existing callers see the same body; it stops being a no-op.
- The file watcher calls the same pipeline, so a saved edit and the route behave identically.
- After provider rows are written, the reconciler calls a `ProviderRefresh` trait object
  implemented by `uiux-realtime-topics-and-replica-refresh` (`ReplicaState::refresh_now`)
  so the live registry matches the rows before the route returns. With no settings manager
  or no realtime bus the call is a no-op and the registry is not touched (there is no
  persisted copy to reconcile).

Both routes and the status route are additions to `administration_capabilities.rs` (entries
such as the existing `settings.reload`, `:206-210`) and to `openapi.rs`.

### D8. Boot status

`GET /api/uar/settings/boot-status` (admin-key guarded), shape:

```
{ "version": 1,
  "config": { "mode": "primary" | "last_known_good" | "fallback_cleared_by_reload",
              "source_digest": "...",
              "fallback": null | { "class": "source_read"|"source_parse", "key_path": "..."|null,
                                   "failed_source_path": "...",
                                   "failed_source_digest": "...", "lkg_digest": "...",
                                   "lkg_saved_at": "...", "lkg_binary_version": "..." },
              "last_known_good": { "present": bool, "digest": "..."|null, "saved_at": "..."|null,
                                   "last_save_error": "class"|null } },
  "embedding": { "ignored": [ { "key": "...", "reason": "invalid_type"|"out_of_range" } ] },
  "providers": { "skipped": [ { "key": "provider.x", "reason": "undecodable" } ] } }
```

No value, secret or parser text appears anywhere in it. The console (change
`uiux-settings-truthful-status-and-removals`) consumes this shape.

## Risks / Trade-offs

- **Explicit limitation: a remote deployment can still fail to start on a bad YAML.** The
  fallback never runs for the remote profile (D2 rule 7), so in the replica topology a broken
  `config.yaml` stops that replica's start with the original error. This contradicts the
  release claim "YAML changes never prevent startup" for remote deployments, and the operator
  was not asked. The design keeps the exclusion on purpose: a silent older posture in a remote
  deployment costs more than a crash that a supervisor makes visible. It is listed under
  Unresolved decisions for the operator below; the release claim must be narrowed to local
  profiles unless the operator decides otherwise.
- **Residual weaker posture.** A YAML that is both unparseable and intended to tighten
  security gives the parser nothing to read, so the fallback boots the previous validated
  config. Controls: local profile only, never under strict mode, an off switch, a loud
  status, the watcher retrying the primary file. This is the one case the design cannot
  close. A deployment that prefers a crash over any older posture sets
  `UAR_CONFIG_FALLBACK=off`.
- **LKG tamper.** The digest guards torn writes, not an attacker with write access to the
  state directory, who could plant a weaker config that boots when the primary breaks. Same
  trust domain as the YAML itself; the permissions above are the only control.
- **LKG is only as old as the last boot.** A live reload is not promoted until a restart
  proves it. Between a reload and the next boot the LKG is the older config.
- **Additive merge resurrection** is bounded by the offered set; until task 3.3 chooses its
  storage the bound is a design commitment, not a verified mechanism.
- **Reload writes from N replicas.** Idempotent by compare-before-write; a lost race writes
  an identical value.
- **Unreadable-vs-missing** is deliberately not eligible when the file is absent. An
  operator who deletes `config.yaml` gets the original error, not a silent LKG.

## Verification surface (tests are authored at the boundary, not now)

Scenarios run against the production binary with real files, a real embedded store and the
real HTTP routes. No unit, mock or structural check counts as acceptance. The auth rule of
2026-10-26 applies: scenarios send no JWT, key or login; the process runs with
`security.jwt_required: false` behind the existing external identity proxy where an owner is
required. The admin routes added here use the admin-key guard; scratch YAML sets
`security.settings_mutation_auth_required: false` (a valid local configuration, which the
guard honors, `src/server.rs:3733-3739` and `require_admin_key`, `src/uar/api/settings.rs:446`) so no admin key is needed, and the
receipt records that choice. SCN-CFG-003 deliberately sets the opposite value as its second
refusal edit, in a run that never reaches the HTTP layer.

No scenario makes a model call (R4). Providers in scratch YAML point at a closed local port
and are never contacted. If a later revision needs a model turn, it uses `src/bin/stub-llm.rs`
(`stub-llm <fixtures.json>` or `STUB_LLM_FIXTURES`, `STUB_LLM_PORT`), never a real route.

Shared fixture contract (every CFG scenario):

- scratch root `$SCN_ROOT/cfg-<NNN>/` containing `home/`, `config.yaml`, `lkg/`, `store/`;
- `HOME=$SCN_ROOT/cfg-<NNN>/home`, `UAR_LAST_KNOWN_GOOD_DIR=$SCN_ROOT/cfg-<NNN>/lkg`;
- embedded SurrealKV store under `store/` (never the installed service's store);
- a listener port from the gate's reserved scratch range; never `1906` (default) or `28000`
  (installed SurrealDB service port, `.prometheus/gotchas.md` 2026-08-23);
- one process at a time per scenario; cleanup owner is the scenario runner, which stops the
  process and removes the scratch root;
- no database command other than reads and the scratch-store fixture writes named in
  SCN-CFG-004/005; no deletion of rows.

Prerequisites declared for the lead's build step (R5):

- the main binary: `cargo build --locked --no-default-features --features server-full`;
- the store fixture tool, which `cargo build` without `--examples` does not produce:
  `cargo build --locked --example surrealql_store_runner --no-default-features --features
  surreal-backend` (`Cargo.toml:156-159`, `required-features = ["surreal-backend"]`;
  invocation from source: `surrealql_store_runner <endpoint> <namespace> <database> <
  query.surql`, endpoint form `surrealkv:///path`, run only against a stopped store, header
  of `scripts/acceptance/fixtures/surrealql_store_runner.rs`); the namespace and database are
  the ones the scratch YAML's persistence settings name;
- no baseline worktree is needed by this change's own controls. The lead's runner-integrity
  gate builds its pre-phase baseline under `~/.claude/worktrees/` via
  `scripts/worktree-new.sh`.

## Frozen oracle fixtures

Written now, read by the boundary runner, never edited by the implementer (R1). A change to
any file requires a revision of this change.

| File | Contents |
|---|---|
| `fixtures/cfg-oracle.json` | env sentinel names and values and the exact two expected LKG files and record keys, modes; the marker setting (`resilience.requests_per_second` = 7) and the broken YAML bytes; the two refusal edits with the log text each must produce; the deliberate-secret control; the four persisted fixture rows with their expected reasons; the provider reload values (`p1` A/B/C/M2, `p2` D/E); the applied (`resilience.requests_per_second` 7 to 9), restart (`memory.max_context_tokens` 2000 to 1500), YAML-at-boot (`server.port` YAML edit, reported `restart_required`) and no-reader (`vision.auto_detect` YAML edit, reported `not_applied`) keys, and the DB-edit cross-check (`server.port` through the settings API, `not_applied`); the five report lists; the HTTP status codes (422, 409, 404, 400, 200); the masked-drift sentinels for `memory.openai_api_key`; the boot-status shape; the watcher sample count |

The expected values are operator decisions, source files read at write time and fabricated
test data. None comes from the implementer's route table, classifier or audit output. The
restart/not-applied classes are those the settings change declares; if that change revises
its table, this fixture is revised with it.

## Acceptance

### Behavior: last-known-good copy holds only validated source bytes

| Field | Declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-config-boot-safety-and-live-reload` / tasks 2.1-2.3 |
| Production entry point | `universal-agent-runtime --config <scratch>/config.yaml` (server run, no subcommand), with `UAR_SECURITY__JWT_SECRET`, `OPENAI_API_KEY` and `UAR_LLM__API_KEY` set in the process environment to the sentinels in `fixtures/cfg-oracle.json` |
| Real collaborators | the production binary; a real YAML file; the real filesystem; embedded SurrealKV in scratch |
| Boundary exercised | process start, filesystem write of `lkg/config.yaml` and `lkg/config.yaml.lkg.json`, `GET /api/uar/settings/boot-status` |
| Observable result | after readiness `lkg/` holds exactly the two frozen files; `sha256(lkg/config.yaml)` equals the record digest and the digest of the YAML file; modes equal the frozen modes; no sentinel appears in any byte under `lkg/`; boot status reports `last_known_good.present: true` with the same digest |
| Negative control | start with an environment-only configuration (no config file): no LKG file is created and status reports `present: false`; separately start while `lkg/` is read-only: the process reaches readiness and status reports `last_save_error` |
| Isolation | shared fixture contract; sentinels are fabricated strings, not credentials |
| Prerequisites | production build of the `server-full` profile |
| Final local gate | one serialized `cargo build --locked --no-default-features --features server-full`, then the scenario runner `scripts/behaviors/configuration-boot-safety.cucumber.mjs` (new, path proposed, following the `scripts/behaviors/*.cucumber.mjs` convention) |
| Evidence | pending |
| Limitations | Windows permissions not covered; a secret written into the YAML itself is duplicated in the copy by design |

### Behavior: boot falls back only for an unparseable YAML, never for a refusal

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 2.4-2.8 |
| Production entry point | same binary and flags; scenario A replaces `config.yaml` with the frozen broken bytes between two starts; scenario B edits it to each frozen refusal edit |
| Real collaborators | production binary; real files; real embedded store; `/readyz` and `/api/uar/settings/boot-status` over HTTP |
| Boundary exercised | process start and exit code; listener bind; admin status route |
| Observable result | A: the second start reaches readiness, `/api/uar/settings/resilience` shows `resilience.requests_per_second` = 7, status reports `mode: last_known_good` with `failed_source_path` equal to the scratch `config.yaml` path, `failed_source_digest` equal to the digest of the broken file, and no substring of the broken bytes. B (both edits): the second start exits non-zero before binding any listener, the log contains the frozen refusal text and no `config.boot.fallback_active`, `lkg/config.yaml` is byte-identical to before |
| Negative control | B is the control for A, and it can reject: two different refusals (a JWT secret refusal and an admin-key refusal) are each refused. Second control: a remote deployment profile with the broken YAML exits non-zero |
| Isolation | shared fixture contract |
| Prerequisites | first start must succeed so an LKG exists |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | the residual in Risks (unparseable and tightened at once) is not provable by a scenario; the scenario proves the stage rule |

### Behavior: bad persisted embedding values and provider rows do not stop boot

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.5, 3.6 |
| Production entry point | production binary started against a scratch store that the runner prepared |
| Real collaborators | production binary; scratch SurrealKV store; for fixture preparation and post-check reads only, the `surrealql_store_runner` example against the **stopped** scratch store (`.prometheus/gotchas.md` 2026-10-29: never point a host `surreal` CLI at a UAR store) |
| Boundary exercised | the frozen rows in `fixtures/cfg-oracle.json` (`embeddingBadType`, `embeddingOutOfRange`, `providerUndecodable`), process start, `GET /api/uar/providers`, boot status |
| Observable result | the process reaches readiness; status lists `llm.embedding.vector_dimension` with the frozen reason and `provider.broken` as `undecodable`; a valid operator-edited provider created through the real provider API before the restart is still listed; a read through the runner after boot shows the rows unchanged |
| Negative control | `providerUndecodable` carries a credential-like value (`api_key: 12345`, sentinel recorded in the fixture): it must be skipped and the sentinel must not appear in boot status, API responses or logs. Second control: the malformed `governance.enabled` row keeps the existing fail-closed result (governance enabled, mutation unavailable) and is not listed as skipped |
| Isolation | shared fixture contract; the only writes are the frozen rows into the scratch store |
| Prerequisites | `surrealql_store_runner` example built (see Verification surface) |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | fixture rows are written out-of-band by design; the product never writes them |

### Behavior: reload and restart keep operator edits and merge YAML additions

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 3.1-3.4, 4.1-4.4, 4.7 |
| Production entry point | edit `config.yaml` while the server runs (file watcher path) and `POST /api/uar/settings/reload` |
| Real collaborators | production binary; real YAML; real provider API (`PUT /api/uar/providers/{id}`) for the operator edit; real store |
| Boundary exercised | provider rows, `GET /api/uar/providers`, `GET /api/uar/settings/drift`, reload route |
| Observable result | with the frozen values: after reload and again after restart `p1.default_model` is `B`, `M2` is in `p1.models`, drift lists `provider.p1` with field `default_model`, the reload response lists `provider.p1` under `retained_with_drift`. After the operator deletes `M2` and the process restarts, `M2` is not re-added |
| Negative control | an invalid YAML reload returns `422`, writes nothing and the earlier values remain; untouched `p2` follows the YAML (`D` to `E`), which would fail if every row were treated as operator-owned |
| Isolation | shared fixture contract; one scratch process |
| Prerequisites | none beyond the build |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | rows stamped by the old seed are treated as operator-owned; the scenario starts from a clean store |

### Behavior: the reload report is truthful and drift can be resolved

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 4.5-4.10 |
| Production entry point | `POST /api/uar/settings/reload`, `POST /api/uar/settings/drift/{key}/resolve` |
| Real collaborators | production binary; real store |
| Boundary exercised | settings rows, drift route, reload route |
| Observable result | after editing in the YAML `server.port`, `memory.max_context_tokens` (2000 to 1500), `resilience.requests_per_second` (7 to 9) and `vision.auto_detect` and reloading: the report lists `resilience.requests_per_second` in `applied`, `memory.max_context_tokens` and `server.port` in `restart_required`, `vision.auto_detect` in `not_applied`; the listener port is unchanged; editing `server.port` through the settings API instead reads back `application_status: not_applied`; after `use_config` on a drifted key the stored value equals the YAML value and a later YAML change reaches the row |
| Negative control | resolve `llm.default_provider` returns `409`; resolve a key with no drift `404`; resolve with `{"resolution":"drop"}` `400` |
| Isolation | shared fixture contract |
| Prerequisites | none beyond the build |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | "keep the stored value" is not persisted and is not tested |

### Behavior: drift, reload and resolve responses never expose a secret

| Field | Declaration |
|---|---|
| Canonical identity | same change / tasks 4.8, 4.9, 4.10 |
| Production entry point | operator sets `memory.openai_api_key` through `PUT /api/uar/settings/memory.openai_api_key` after the YAML seeded a different value; the YAML is edited to a third value; `POST /api/uar/settings/reload`; `GET /api/uar/settings/drift`; `POST .../drift/memory.openai_api_key/resolve` |
| Real collaborators | production binary; real store |
| Boundary exercised | the schema-sensitive masking path (`x-sensitive`, `masked_response`, `src/uar/api/settings.rs:557-597`) applied to the new `config_value` |
| Observable result | the drift entry for the key shows the masked form `***` for both the stored and the configuration value; none of the three sentinel strings appears in any response body of the five listed calls, nor in the process stdout and stderr |
| Negative control | the key is read once with the masking disabled by nothing: the runner reads the raw row through `surrealql_store_runner` on a stopped store and confirms the stored value is the operator sentinel, proving the value existed and was withheld rather than absent; separately `resolve` with an unknown resolution returns `400` |
| Isolation | shared fixture contract |
| Prerequisites | `surrealql_store_runner` example (post-check) |
| Final local gate | same runner |
| Evidence | pending |
| Limitations | masking of non-sensitive keys is not tested; the console's masked-secret round trip is `uiux-settings-truthful-status-and-removals`' scenario |

## Scenario matrix rows

All rows share: fixtures per "Frozen oracle fixtures" and the shared fixture contract;
restoration = stop the process and remove `$SCN_ROOT/cfg-<NNN>/`; production build identity
recorded in the receipt; negative controls are real invalid or conflicting inputs with a named
observable rejection, and a mandatory scenario whose control did not run is BLOCKED;
`nondeterministic: false` rows await readiness rather than sleeping, and use no model.

**SCN-CFG-001**
- name: LKG holds validated source bytes only
- actor: operator running UAR locally
- useful outcome: a recoverable copy of the config that booted exists and leaks no environment secret
- source requirement: `openspec:configuration-boot-safety#Last-known-good configuration holds validated source bytes only`
- reason now: the boot-brick fix is the highest-risk new disk write
- likelihood: UNKNOWN
- mandatory: true
- production entry: `universal-agent-runtime --config <scratch>/config.yaml` with sentinel secrets in the environment
- real collaborators: production binary, filesystem, embedded SurrealKV
- oracle: observation = bytes, mode and digest of files under `lkg/`; expected truth source = the YAML file bytes, their SHA-256 computed by the runner, and `fixtures/cfg-oracle.json` (`lkg`); comparator = byte equality, digest equality, exact file set, modes equal, `grep -c <sentinel>` equals 0 for every sentinel
- negative control: input = environment-only start (no file), and a read-only `lkg/`; expected rejection = no LKG file created, and process ready with `last_save_error` in boot status
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: argv, env names (not values), mode listing, digests, grep counts, boot-status body
- substituted boundaries: none
- limitations: Windows permissions not covered
- invalidation keys: `src/config_manager.rs`, `src/config.rs`, `src/main.rs`, `src/server.rs` readiness section, LKG module, `fixtures/cfg-oracle.json`
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#2.1`, `#2.2`, `#2.3`, `#6.1`

**SCN-CFG-002**
- name: unparseable YAML boots from LKG with a visible warning
- actor: operator who saved a bad edit
- useful outcome: next start comes up on the last good config and says so
- source requirement: `openspec:configuration-boot-safety#Boot falls back only for an unreadable or unparseable configuration source`
- reason now: item 7; the operator-visible brick
- likelihood: UNKNOWN
- mandatory: true
- production entry: two starts of the production binary with `config.yaml` replaced by the frozen broken bytes between them
- real collaborators: production binary, filesystem, embedded SurrealKV, HTTP routes
- oracle: observation = readiness, `GET /api/uar/settings/resilience`, boot-status body, log line name; expected truth source = `fixtures/cfg-oracle.json` (`bootYaml.markerSetting`, `brokenYamlBytes`) and the digest of the broken file; comparator = equality; boot status `failed_source_path` equals the `--config` path given and contains no substring of the broken bytes
- negative control: input = remote deployment profile YAML first run, then the broken edit; expected rejection = non-zero exit, no listener, the original error
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: both start logs, exit codes, status bodies, digests
- substituted boundaries: none
- limitations: does not prove intent of the broken edit; the remote profile is intentionally excluded (see Risks)
- invalidation keys: as SCN-CFG-001 plus `src/uar/settings/manager.rs` resilience seed
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#2.4`, `#2.5`, `#2.6`, `#2.7`, `#6.2`

**SCN-CFG-003**
- name: a security refusal never boots from LKG
- actor: operator who tightened security incorrectly
- useful outcome: UAR refuses to start instead of starting weaker
- source requirement: `openspec:configuration-boot-safety#A validation refusal never falls back`
- reason now: unresolved critical 4
- likelihood: UNKNOWN
- mandatory: true
- production entry: first start with `security.jwt_required: false` (valid, LKG saved); then one start per frozen refusal edit (`jwt-fallback-secret`, `admin-key-missing`)
- real collaborators: production binary, filesystem
- oracle: observation = exit code, listener absence, log text, `lkg/config.yaml` digest; expected truth source = `fixtures/cfg-oracle.json` (`refusalEdits`) and the LKG digest before the edit; comparator = non-zero exit, port closed, log contains the frozen text, digest unchanged, no `config.boot.fallback_active` line
- negative control: input = the second, different refusal edit (`admin-key-missing`); expected rejection = the same non-zero exit and no fallback, proving the rule is by stage and not by one message. Positive guard: the first edit with `UAR_SECURITY__JWT_SECRET` set to the deliberate value boots normally from the primary file
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: all exit codes, log excerpts, digests, port probe results
- substituted boundaries: none
- limitations: two refusals exercised; the stage rule covers the rest structurally
- invalidation keys: `src/config.rs` validation functions, `src/config_manager.rs`
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#2.5`, `#2.8`, `#6.3`

**SCN-CFG-004**
- name: malformed persisted embedding value does not stop boot
- actor: operator with an old or hand-edited store
- useful outcome: UAR starts and names the ignored value
- source requirement: `openspec:configuration-boot-safety#Malformed persisted values are skipped individually`
- reason now: item 7, second boot path
- likelihood: UNKNOWN
- mandatory: true
- production entry: production binary against a scratch store holding the frozen `embeddingBadType` row, and again with `embeddingOutOfRange`
- real collaborators: production binary, scratch SurrealKV, `surrealql_store_runner` for the fixture write and the post-check read
- oracle: observation = readiness, boot-status `embedding.ignored`, the post-boot row read; expected truth source = the fixture rows and their frozen reasons; comparator = listed with the frozen reason, row unchanged
- negative control: input = the frozen `governanceMalformed` row; expected rejection = the existing fail-closed result (governance enabled, mutation unavailable) and not listed under `embedding.ignored`
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json` (`persistedRows`)
- evidence required: runner write/read output, boot-status, governance status route
- substituted boundaries: fixture rows are written out-of-band (the product cannot write them)
- limitations: dimension mismatch against already indexed knowledge is not exercised
- invalidation keys: `src/server.rs` embedding overlay, `src/uar/settings/manager.rs` `get_typed`
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#3.6`, `#6.4`

**SCN-CFG-005**
- name: malformed provider row does not drop other providers
- actor: operator with one corrupt provider row
- useful outcome: other providers, including operator edits, still load; the bad row is named
- source requirement: `openspec:configuration-boot-safety#Malformed persisted values are skipped individually`
- reason now: item 7, third boot path
- likelihood: UNKNOWN
- mandatory: true
- production entry: production binary against a scratch store with one valid operator-edited provider and the frozen undecodable `provider.broken` row
- real collaborators: production binary, provider API, scratch store, store runner
- oracle: observation = `GET /api/uar/providers`, boot-status `providers.skipped`, the post-boot row read; expected truth source = the provider edited before restart and `fixtures/cfg-oracle.json` (`providerUndecodable`); comparator = valid provider present with its edit, `provider.broken` listed `undecodable`, row unchanged
- negative control: input = the undecodable row carrying the credential-like sentinel; expected rejection = the row is skipped and the sentinel appears in no status body, API response or log line
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: provider list bodies, status bodies, row reads, grep count of the sentinel over logs
- substituted boundaries: none beyond the out-of-band fixture row
- limitations: credential protection for the surviving rows is not re-proved here
- invalidation keys: `src/uar/settings/manager.rs` provider load and protect functions, `src/server.rs` hydrate block
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#3.5`, `#6.5`

**SCN-CFG-006**
- name: DB wins over YAML on reload and restart, YAML additions merge
- actor: operator who edits provider settings in the console and keeps a YAML in version control
- useful outcome: console edits survive; new YAML models still arrive; deleted models stay deleted
- source requirement: `openspec:provider-model-settings-certification#Provider bootstrap preserves persistent settings authority`
- reason now: item 9 and decision 1
- likelihood: UNKNOWN
- mandatory: true
- production entry: file edit with watcher, `POST /api/uar/settings/reload`, process restart
- real collaborators: production binary, real YAML, provider API, store
- oracle: observation = `GET /api/uar/providers` rows, `GET /api/uar/settings/drift`, reload response; expected truth source = `fixtures/cfg-oracle.json` (`reload.providerP1`, `providerP2`); comparator = field equality as in the Acceptance table
- negative control: input = an invalid YAML reload; expected rejection = `422`, no row changes
- nondeterministic: true (file watcher timing)
- stochastic: samples 5 watcher-triggered reloads; grader = the post-reload provider rows equal the frozen expectations within 10 s; stoppingRule = fixed N=5, all must pass
- fixtures: `fixtures/cfg-oracle.json`; two YAML providers pointing at a closed local port (never called)
- evidence required: before/after provider bodies, drift body, reload body, restart bodies
- substituted boundaries: provider upstreams are never contacted
- limitations: legacy rows stamped by the old seed are not covered
- invalidation keys: `src/uar/settings/manager.rs`, `src/config_manager.rs`, `src/uar/api/settings.rs`, `src/uar/api/providers.rs`
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#3.1`-`#3.4`, `#4.1`-`#4.4`, `#4.7`, `#6.6`

**SCN-CFG-007**
- name: reload report is truthful and drift resolves on request
- actor: operator using the reload route
- useful outcome: knows what applied, what needs a restart, what did nothing, and can adopt the YAML value
- source requirement: `openspec:configuration-boot-safety#A reload reports what it did` and `openspec:configuration-boot-safety#Drift can be resolved by adopting the configuration value`
- reason now: item 8; the current route is a silent no-op
- likelihood: UNKNOWN
- mandatory: true
- production entry: `POST /api/uar/settings/reload`, `POST /api/uar/settings/drift/{key}/resolve`
- real collaborators: production binary, real YAML, store
- oracle: observation = response bodies, `GET /api/uar/settings/resilience`, listener port; expected truth source = `fixtures/cfg-oracle.json` (`reload.applied`, `restartRequired`, `yamlAtBoot`, `noReader`, `dbEditCrossCheck`, `reportLists`, `statusCodes`); comparator = each key in exactly the frozen list, the live port unchanged, and the YAML-edit versus DB-edit statuses for `server.port` differ as frozen
- negative control: inputs = resolve `llm.default_provider` (`409`), resolve a key without drift (`404`), resolve with `{"resolution":"drop"}` (`400`); each rejection observed
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: all bodies and status codes
- substituted boundaries: none
- limitations: acknowledgement persistence not covered (not built)
- invalidation keys: `src/uar/api/settings.rs`, `src/uar/settings/manager.rs`, `src/config_manager.rs`, the settings change's class table
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#4.5`-`#4.10`, `#6.7`

**SCN-CFG-008**
- name: drift, reload and resolve responses never expose a secret
- actor: operator viewing drift for a secret setting
- useful outcome: drift is visible without leaking either secret
- source requirement: `openspec:configuration-boot-safety#Drift is visible with both values`
- reason now: non-waivable check SAFE-MASKED-SECRET-ROUNDTRIP had no scenario reading a secret-bearing drift entry
- likelihood: UNKNOWN
- mandatory: true
- production entry: the sequence in the Acceptance table on `memory.openai_api_key`
- real collaborators: production binary, real store, `surrealql_store_runner` post-check on a stopped store
- oracle: observation = the five response bodies and process output; expected truth source = `fixtures/cfg-oracle.json` (`driftMasking`); comparator = masked form `***` for stored and configuration values, `grep -c` of the three sentinels equals 0 in every body and in stdout and stderr
- negative control: input = `resolve` with `{"resolution":"drop"}` (expect `400`), and a post-check proving the operator sentinel was stored (so the masking withheld a real value)
- nondeterministic: false
- fixtures: `fixtures/cfg-oracle.json`
- evidence required: bodies, grep counts, the post-check read
- substituted boundaries: none
- limitations: only one sensitive key is exercised; others rely on the shared schema flag
- invalidation keys: `src/uar/api/settings.rs` masking, `src/uar/settings/schema.rs`, `fixtures/cfg-oracle.json`
- restoration: as common
- owner tasks: `uiux-config-boot-safety-and-live-reload#4.8`, `#4.9`, `#4.10`, `#6.8`

## Open decisions and UNVERIFIED items

- Offered-set storage (D5) is undecided; task 3.3 decides after reading `ProviderConfig`
  decoding and the `internal` settings type.
- Exact readiness line for the LKG save (D1) is chosen at implementation.
- Whether the existing manual scripts satisfy the admin-key guard without minting a
  credential is UNVERIFIED; the scratch YAML avoids the question by disabling
  `settings_mutation_auth_required` in scenarios other than SCN-CFG-003.
- The `/.well-known/uar-config/reload` response body is kept; external SDK consumers of it
  are UNVERIFIED.
- `config::File::with_name` extension resolution for an explicit `--config` value without an
  extension is UNVERIFIED; the design treats any path that is not an existing regular file as
  "nothing to snapshot".
- The namespace and database names in the scratch YAML's persistence settings (for the store
  runner) are read at implementation.
- Single-writer assignment is required for every file in the proposal's Impact list before
  implementation starts; none belongs to `uar-state`.

## Unresolved decisions for the operator

1. **Remote profile and "YAML changes never prevent startup".** The fallback is excluded for
   the remote deployment profile, so a bad YAML still stops a remote replica from starting.
   Options: (a) keep the exclusion and narrow the release claim to local profiles
   (recommended; a silent older security posture in a remote deployment is the larger risk);
   (b) allow the fallback in remote mode only when the LKG and the failed file differ in no
   security-namespace key, which cannot be computed when the failed file does not parse, so
   it is not offered; (c) allow it with a mandatory visible alarm. The operator has not been
   asked.
2. **Rows stamped by the old seed** are treated as operator-owned (conservative). Accept, or
   authorize a one-time migration that re-stamps them config-owned.
