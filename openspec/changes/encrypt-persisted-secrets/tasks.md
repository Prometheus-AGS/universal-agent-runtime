# Tasks — encrypt-persisted-secrets

Contract tests come first. Each test in section 2 is run once against the unmodified branch and must **fail** (record the output under `openspec/changes/encrypt-persisted-secrets/evidence/`), then pass after section 3. *Regression guard* tests pass before and after. All tests run locally, never in GitHub Actions.

Tests use an injectable key source: a `TestKeySource` (in-memory, test-visible key) for automated runs, the real OS credential store only in the per-OS tests (2.14–2.16), and `NoKeySource` for the no-key cases. Canary values are random per run (`sk-canary-<uuid>`, `env-canary-<uuid>`). "Byte search" means: every file under the data directory recursively (including SurrealKV segment, manifest and lock files), every file under the log directory (`UAR_LOG_FILE` set inside the temp dir), and captured stdout/stderr; each canary searched raw, base64 (standard and URL-safe) and JSON-escaped.

Falsifier map:

| Source | Falsifier / finding | Tests |
|---|---|---|
| D4 falsifier 3 | new writes: provider key and MCP env secret not findable; both still work | 2.1, 2.2, 2.4, 2.5 |
| D4 falsifier 4 | migration of a plaintext fixture: nothing findable; entries work (encrypted) or show re-entry (stripped) | 2.7, 2.8, 2.9 |
| D4 review r3, CRITICAL 2 / open risk 2 | key not beside ciphertext; secrets unrecoverable from the data directory alone | 2.10, 2.11 |
| D4 decision "logs counts without values" | migration log | 2.7, 2.8 |
| D1 falsifier 3 | run-scoped credential containment | **not this change** (D1 run-scoped credential change) |

## 1. Bootstrap

- [ ] 1.1 Confirm `openspec validate encrypt-persisted-secrets --strict` passes; record output in `evidence/validate.txt`.
- [x] 1.1a Operator acceptance of the deviation from D4's wording (`design.md` §2): one sealed format inside UAR settings instead of the per-provider `CredentialStore`, and rollback to an older UAR requires re-entering secrets.
  Operator decision 2026-09-23: approved — one sealed format inside UAR settings, built on the existing `CredentialEncryption`, keys from the OS store; rollback to an older UAR requires re-entering secrets.
- [ ] 1.2 Register the change in the-boss child phase `the-boss-universal-agent-runtime` progress; verify it is listed.
- [ ] 1.3 Choose the credential-store crate (`keyring` first candidate), confirm its backends on macOS, Windows and Linux without a build-time `libdbus` requirement, pin it following `versions.toml` rules; verify `cargo tree -i keyring` resolves once and the default build succeeds on macOS.
- [ ] 1.4 Add test support `tests/support/secrets_fixture.rs`: builds a SurrealKV data directory containing plaintext `provider.<id>` rows with `api_key`, an `mcp.servers` row with `env` secrets, and plaintext `security.jwt_secret` / `unstructured.api_key` rows, by writing through `PersistenceLayer::upsert_setting` directly (bypassing `SettingsManager`); verify the canaries are findable by byte search in the fixture (positive control).
- [ ] 1.5 Reuse the test-only `uar-env-probe` binary from `sidecar-launch-security` task 1.4; if that change has not landed, add the probe here under the same name and feature; verify it builds only with the test feature.

## 2. Contract tests (write first; each fails before section 3 unless marked)

All in `tests/encrypt_persisted_secrets.rs` unless noted.

New writes (D4 falsifier 3):
- [ ] 2.1 `provider_api_key_absent_at_rest_after_create_and_update` — `POST /api/uar/providers` with canary A, `PUT` with canary B, restart; byte search finds neither A nor B.
- [ ] 2.2 `mcp_env_secret_absent_at_rest` — `PUT /api/uar/mcp/servers/probe` (`enabled: false`, stdio command `uar-env-probe`, `env: {PROBE_SECRET: <canary>}`), restart; byte search finds no canary.
- [ ] 2.3 `seeded_and_initialized_secrets_absent_at_rest` — fresh data dir; start with `OPENAI_API_KEY`-style provider key, `UAR_SECURITY__JWT_SECRET`, and an `unstructured.api_key` in config, all canaries; after startup byte search of the data dir finds none. (Covers `seed_providers_from_registry` and `initialize`.)
- [ ] 2.4 `provider_key_works_after_restart` — provider `base_url` points at a `wiremock` server; save key, restart with the same `TestKeySource`, call `POST /api/uar/providers/{id}/test` (or route a chat request); asserts the mock received `Authorization: Bearer <canary>` and the provider reports `credential_state: stored`. *Passes before on the "works" half; fails before on `credential_state`.*
- [ ] 2.5 `mcp_env_secret_delivered_after_restart` — after 2.2 and a restart, enables the server; `uar-env-probe` is spawned (MCP handshake is expected to fail) and its JSON output contains `PROBE_SECRET=<canary>`. *Regression guard for delivery; must keep passing after sealing.*
- [ ] 2.6 `ciphertext_does_not_decrypt_at_another_location` — copies provider A's sealed `api_key` into provider B's row via direct persistence write, restarts; B reports `needs_reentry`, and no request to B's mock carries A's key.

Migration (D4 falsifier 4):
- [ ] 2.7 `migration_seals_plaintext_fixture_with_key_source` — start on the 1.4 fixture with `TestKeySource`; asserts: byte search finds no fixture canary; providers report `stored` and 2.4's mock check passes for one fixture provider; the MCP probe receives its env canary; exactly one `secrets.migration` log line with numeric counts and no canary or key name that carries a value.
- [ ] 2.8 `migration_strips_plaintext_fixture_without_key_source` — same fixture with `NoKeySource`; asserts: byte search finds no canary; providers report `needs_reentry` and `credential_configured: false`; `GET /api/uar/mcp/servers` lists `envKeysNeedingReentry: ["PROBE_SECRET"]`; the settings API's `secret_state` shows `needs_reentry` for `security.jwt_secret`; log line has counts only.
- [ ] 2.9 `migration_is_idempotent` — second start after 2.7: log reports `sealed=0 stripped=0 rewritten=false`; the SurrealKV directory's inode/creation time is unchanged.

Key not in the data directory (round-3 CRITICAL):
- [ ] 2.10 `secrets_unrecoverable_from_data_directory_alone` — after 2.1 and 2.2 with `TestKeySource` (key K known to the test): copies the data dir; byte-searches the copy for K (raw, hex, base64) → absent; starts UAR on the copy with an empty `TestKeySource` → every secret `needs_reentry`, the wiremock receives no key; negative control: starting the copy with the original K → `stored`.
- [ ] 2.11 `no_key_file_is_created` — lists every file created under the data dir, config dir and the process's working dir during 2.1; asserts none contains K and none is named like a key file (`*.key`, `*key*`) outside SurrealKV's own files.

No key source:
- [ ] 2.12 `new_secret_without_key_source_is_session_only` — `NoKeySource`; save provider key; asserts `credential_state: session_only`, the mock receives the key in this process, byte search finds no canary; restart → `needs_reentry`.
- [ ] 2.13 `secrets_status_reports_source_and_counts_without_values` — `GET /api/uar/secrets/status` under `TestKeySource`, `NoKeySource` and `CREDENTIAL_ENCRYPTION_KEY` set; asserts `key_source` is `os_keystore`/`none`/`env` respectively, counts match, and the body contains no canary and no ciphertext.

Per-OS credential store (local, `#[ignore]` by default, run manually on each OS and record in `evidence/`):
- [ ] 2.14 `os_keystore_roundtrip_macos` — creates `data-key/<random store_id>` under a test service name in the login Keychain, reads it back, seals/unseals a canary, deletes the item.
- [ ] 2.15 `os_keystore_roundtrip_windows` — same against Credential Manager on Windows x64 and Windows ARM64.
- [ ] 2.16 `os_keystore_roundtrip_linux_secret_service` — same against a Secret Service (GNOME Keyring in a dbus session); and `linux_without_session_bus_falls_back_to_none` with `DBUS_SESSION_BUS_ADDRESS` unset → status `none`.

Read surfaces:
- [ ] 2.17 `responses_never_contain_secrets_or_ciphertext` — after 2.1–2.2, reads `GET /api/uar/providers`, `/{id}`, `GET /api/uar/mcp/servers`, the settings API for `provider`, `mcp`, `security`, `unstructured`, and secrets status; asserts no canary and no `"$uar_sealed"`/`ct` value in any body. *Partly a regression guard (masking exists today); fails before on the new fields.*

## 3. Implementation

- [ ] 3.1 `CredentialEncryption::{encrypt_with_aad, decrypt_with_aad}`; envelope types; key id. Verify existing `encryption.rs` tests still pass.
- [ ] 3.2 `src/uar/security/secrets/`: `KeySource` trait with `EnvKeySource`, `OsKeystoreKeySource` (macOS/Windows/Linux), `NoKeySource`, `TestKeySource` (test-only); resolution order and `store_id`. Verify 2.13 and 2.14–2.16 on the available OS.
- [ ] 3.3 Schema walker (`properties`/`items`/`additionalProperties`); mark `mcp` env values `x-sensitive`; `SettingsManager::write_setting` seals before all seven `upsert_setting` sites; unseal on load into cache with per-pointer state. Verify 2.1–2.3, 2.6.
- [ ] 3.4 Session-only path when no key source. Verify 2.12.
- [ ] 3.5 Startup migration in `initialize` before hydration, with count-only log; SurrealKV export/import directory swap when rows changed (or a proven compaction); `VACUUM FULL` of the settings table on Postgres; remote-SurrealDB warning. Verify 2.7–2.9.
- [ ] 3.6 `credential_state` on `ProviderView`; `envKeysNeedingReentry` on both MCP views; `secret_state` on the settings API; `GET /api/uar/secrets/status`. Verify 2.4, 2.8, 2.13, 2.17.
- [ ] 3.7 Verify 2.5, 2.10, 2.11 with everything wired.
- [ ] 3.8 Add `secrets_at_rest` to the capabilities list served by `GET /api/uar/capabilities` (`sidecar-launch-security` design Decision 12) in the commit that completes 3.1–3.6; verify the route lists it.

## 4. Verification at the change boundary

- [ ] 4.1 Run `tests/encrypt_persisted_secrets.rs` once on macOS; record output.
- [ ] 4.2 Run 2.14–2.16 on macOS, Windows x64, Windows ARM64 and Linux; record per platform or record it as unverified.
- [ ] 4.3 Run the existing settings-persistence, credentials-API and providers tests once (`tests/settings_persistence.rs`, `tests/credentials_api_integration_test.rs`, `tests/test_provider_resolution.rs`); record output.
- [ ] 4.4 Confirm with the the-boss settings work (D3) that it reads `credential_state` and `GET /api/uar/secrets/status`.
- [ ] 4.5 No tool-workflow files change; the Codex/Claude Code/Cursor/OpenCode validation rule does not apply.
