# Tasks — surreal-scoped-signin-and-embedded-fallback

Contract tests come first. Each test in section 2 is run once against the unmodified branch and must **fail** (record output under `openspec/changes/surreal-scoped-signin-and-embedded-fallback/evidence/`), then pass after section 3, unless marked *regression guard* (passes before and after) or *assumption falsifier* (tests SurrealDB itself, not UAR code; it can pass before, and a failure reopens D4). All tests run locally, never in GitHub Actions.

Tests that need a real server start `surrealdb/surrealdb` at the exact image pinned in `versions.toml` (`surrealdb_image`, v3.2.4 digest) on a random host port with a random root password, and are `#[ignore]` unless `UAR_TEST_DOCKER=1`. Provisioning uses root; UAR under test never receives root credentials unless the test says so.

Falsifier map:

| Source | Falsifier / finding | Tests |
|---|---|---|
| D4 falsifier 5 | `uar` namespace user cannot `SELECT`/`INFO` the `memory` and `compass` namespaces | 2.3 |
| D4 falsifier 6 | Docker stopped → SurrealKV within 10 s on macOS and Windows; session marked local-only; survives restart; not merged when Docker returns | 2.8, 2.13, 2.14 |
| D4 review r3, warning 6 | Linux missing from the fallback falsifier | 2.8 runs on Linux too |
| D4 assumption "SurrealKV works on Windows despite documented limits" | Windows soak | 2.16 |
| D4 decision "never root" | sidecar refuses root | 2.6 |
| D4 decision "a `ws://` remote URL" | ws works; http error explains | 2.1, 2.7 |

## 1. Bootstrap

- [ ] 1.1 Confirm `openspec validate surreal-scoped-signin-and-embedded-fallback --strict` passes; record output in `evidence/validate.txt`.
- [ ] 1.2 Register the change in the-boss child phase `the-boss-universal-agent-runtime` progress; verify it is listed.
- [ ] 1.3 Add test support `tests/support/surreal_container.rs`: start/stop/restart the pinned image, run provisioning SQL as root (namespace `uar` + database `uar` + `uar_app` `EDITOR` user; namespaces `memory` and `compass` each with one table and one record), and expose the ws URL; verify it starts and root can `INFO FOR ROOT`.
- [ ] 1.4 Confirm the provisioning statements in `design.md` Decision 1 run unchanged on 3.2.4; correct `design.md` if the syntax differs, before writing section 2.
- [ ] 1.5 Identify the session record type, table and create/list endpoints the-boss's D1 driver uses (UAR `session_id`), and record them in `design.md` Decision 5; verify by creating one session through that endpoint against embedded storage.

## 2. Contract tests (write first)

All in `tests/surreal_persistence_connection.rs` unless noted.

Scoped sign-in:
- [ ] 2.1 `namespace_user_starts_uar_on_ws_url` — UAR (standalone, in-process) with `ws://`, `surreal_auth_level: namespace`, `uar_app` credentials; asserts startup completes and a created agent/thread record is readable as root in `uar.uar`. (Fails before: UAR always signs in with `Root`.)
- [ ] 2.2 `namespace_editor_runs_uar_schema_and_live_queries` — same setup; asserts the four schema scripts applied (tables exist via `INFO FOR DB` as root) and a live-query event arrives for a write made through the API.
- [ ] 2.3 `namespace_user_cannot_read_other_namespaces` — signs in directly with the SDK as `uar_app`; asserts `SELECT` on the `memory` and `compass` tables, `INFO FOR NS` after `USE NS memory`/`compass`, `INFO FOR ROOT`, `DEFINE USER x ON ROOT ...` and `REMOVE NAMESPACE memory` all error and return no data; afterwards root confirms `memory` and `compass` are intact. *Assumption falsifier.*
- [ ] 2.4 `scoped_level_without_password_fails_as_config_error` — `surreal_auth_level: namespace`, no password, server root password not `root`; asserts startup error names `surreal_pass` and is not a server authentication error (proves no `root`/`root` attempt was made).
- [ ] 2.5 `default_auth_level_remains_root` — no auth level, root credentials; asserts UAR starts. *Regression guard.*
- [ ] 2.6 `sidecar_refuses_root_for_remote_url` — sidecar binary (with a launch token, per `sidecar-launch-security`) and a remote URL with no auth level, then with `root`; asserts non-zero exit before `READY` with the stated error, and root's session list on the server shows no sign-in from UAR. (Depends on `sidecar-launch-security` for the sidecar signal.)
- [ ] 2.7 `http_url_error_names_ws_equivalent` — `persistence.database_url: http://127.0.0.1:<port>`; if the connect fails with an unsupported-protocol error, asserts the startup error contains `ws://127.0.0.1:<port>`; if it connects, the test records that this build supports HTTP and asserts sign-in works (so the test documents which case holds).

Fallback:
- [ ] 2.8 `unreachable_remote_falls_back_within_10s` — sidecar with `remote_fallback: embedded`, namespace auth, container **stopped**; measures process start → `READY`; asserts < 10 s; `GET /api/config/persistence` reports `mode: embedded`, `configured_mode: remote`, `fallback.active: true`, `fallback.reason: remote_unreachable`. Run on macOS, Linux, Windows x64 (and Windows ARM64 when a machine is available); record timings per platform.
- [ ] 2.9 `unresponsive_remote_falls_back_after_timeout` — remote URL points at a local TCP listener that accepts and never answers; `remote_connect_timeout_ms: 2000`; asserts `READY` < 10 s and `fallback.reason: remote_timeout`.
- [ ] 2.10 `reachable_remote_with_bad_credentials_does_not_fall_back` — running container, wrong password, fallback enabled; asserts startup fails with the sign-in error and no file is created at `fallback_database_url`.
- [ ] 2.11 `fallback_disabled_keeps_hard_failure` — `remote_fallback: none`, container stopped; asserts startup fails as today. *Regression guard.*
- [ ] 2.12 `no_store_switch_when_remote_returns_while_running` — start in fallback, start the container, create a session; asserts it is in the fallback store and not in the remote, and status still reports fallback active.
- [ ] 2.13 `fallback_session_listed_local_only_and_survives_restart` — in fallback, create a session through the API the-boss's D1 driver uses (UAR `session_id`; task 1.5 pins the endpoint and table); asserts it appears in `GET /api/uar/persistence/local-only` with `active: true`; restart the sidecar with the container still stopped; asserts the session is still listed and still readable.
- [ ] 2.14 `fallback_session_not_merged_after_remote_returns` — after 2.13, start the container and restart the sidecar; asserts status reports `mode: remote`, `fallback.active: false`; a root `SELECT` on the remote finds no record with that session id; `GET /api/uar/persistence/local-only` reports `present: true, active: false` and still lists the session; the session record and the `uar_store_meta:store` record read from the fallback store are byte-identical (as JSON) to their values before the remote returned.
- [ ] 2.15 `persistence_status_and_logs_redact_url_credentials` — `database_url: ws://uar_app:<canary>@127.0.0.1:<port>`; asserts the canary is absent from `GET /api/config/persistence` and from stdout/stderr/log file.

Windows risk:
- [ ] 2.16 `surrealkv_fallback_windows_soak` (file `tests/surrealkv_windows_soak.rs`, `#[ignore]`, run manually on Windows x64 and ARM64) — sidecar in fallback; 8 concurrent sessions each writing 500 turns via the API; kill the process with `taskkill /F` mid-write; restart; asserts every write acknowledged before the kill is readable and the store opens without error; repeat 3 times. Record results; a failure reopens D4's storage decision for Windows.

## 3. Implementation

- [ ] 3.1 Config: `surreal_auth_level`, `remote_fallback`, `remote_connect_timeout_ms`, `fallback_database_url` in `PersistenceConfig` (`src/config.rs:644-681`), settings schema entries in the persistence settings type, validation of required fields per level. Verify 2.4, 2.5.
- [ ] 3.2 `SurrealDbProvider::new` takes an auth level; `Namespace`/`Database` sign-in; no root default at those levels; connect wrapped in the timeout; URL userinfo redacted in logs; `ws://` hint on unsupported protocol. Verify 2.1, 2.2, 2.7, 2.15 (log half).
- [ ] 3.3 Sidecar root refusal using the `sidecar-launch-security` guard signal. Verify 2.6.
- [ ] 3.4 Fallback classification and embedded open at `server.rs:606-626`; `FallbackState` in `AppState`; `uar_store_meta:store` record on first fallback open. Verify 2.8–2.12.
- [ ] 3.5 `persistence_info_handler` effective mode, `configured_mode`, `fallback`, redacted URL; `sync_stream_handler` uses the effective mode; `GET /api/uar/persistence/local-only`; one-time `persistence.local_only_store_present` log. Verify 2.13–2.15.
- [ ] 3.6 Update `config.remote.surreal.yaml` to `ws://` and namespace-level credential comments. Verify the file parses with `ConfigManager`.
- [ ] 3.7 Review the diff for SurrealDB ≥ 3.3.0 features (`DEFINE ACCESS ... WITH CONTEXT`, SCRAM, any syntax absent from 3.2.4); record "none" or the finding in `evidence/`.

## 4. Verification at the change boundary

- [ ] 4.1 Run `tests/surreal_persistence_connection.rs` once with `UAR_TEST_DOCKER=1` on macOS; record output.
- [ ] 4.2 Run 2.8 on Linux and Windows x64 (and ARM64 if available) and 2.16 on Windows; record per platform or record it as unverified.
- [ ] 4.3 Run existing persistence tests once (`tests/rest_api_persistence.rs`, `tests/settings_persistence.rs`, `tests/agent_threads.rs`); record output.
- [ ] 4.4 Hand the provisioning statements to the prometheus-skills-mini Docker services owner; verify the shared SurrealDB has the `uar_app` user and that 2.3 passes against it.
- [ ] 4.5 No tool-workflow files change; the Codex/Claude Code/Cursor/OpenCode validation rule does not apply.
