## Why

UAR writes secrets in plaintext into its settings database. `POST`/`PUT /api/uar/providers` persist the whole `ProviderConfig`, including `api_key` (`src/uar/api/providers.rs:47-57,131-200`; `SettingsManager::upsert_provider_config`, `src/uar/settings/manager.rs:820-850`; `ProviderConfig.api_key`, `src/llm/registry.rs:46-47`). `PUT /api/uar/mcp/servers/{name}` persists MCP definitions with their `env` values under the settings key `mcp.servers` (`src/uar/api/mcp_admin.rs:120-135,185-231`; the transport-free twin `src/uar/admin/mcp.rs:100-118,184-216`). Two more writers do the same without an API call: first-boot provider seeding copies env/YAML keys into the database (`manager.rs:749-800`), and settings initialization copies every configured value, including fields the settings schema itself marks `x-sensitive` (`security.jwt_secret`, `persistence.surreal_pass`, `unstructured.api_key`, `mistral_ocr.api_key`, the memory embedding keys, `llm_failover` fallback keys; `manager.rs:1077-1083,1351,1656,1728,1756,2192-2214,2535`). Anyone who copies the data directory, a Docker volume or a backup gets every key. D4 requires that this stop, and the round-3 review blocked D4 because it did not say where the decryption key lives.

## What Changes

- Every value whose settings-schema node is marked `x-sensitive` is stored as a sealed envelope (AES-256-GCM, bound to its setting key and JSON path) instead of plaintext. Provider `api_key` is already marked; MCP server `env` values become marked. One write chokepoint in `SettingsManager` seals before every `upsert_setting` call (`manager.rs:176,226,316,376,523,783,849`), so API writes, seeding and initialization are all covered.
- The data-encryption key never sits in the data directory. It comes from, in order: the operator's `CREDENTIAL_ENCRYPTION_KEY` (existing variable, `src/uar/security/credentials/encryption.rs:43-79`, for server deployments), else the OS credential store — macOS login Keychain, Windows Credential Manager (DPAPI-protected), Linux Secret Service — else none.
- With no key source, secrets are **not persisted**: a new secret works for the running process only (`session_only`) and the stored entry is marked `needs_reentry`. There is no key-file fallback.
- A startup migration finds existing plaintext secrets, seals them when a key source exists or strips them when none does, marks stripped entries `needs_reentry`, rewrites the embedded store so old plaintext versions do not survive on disk, and logs counts only.
- APIs report secret state without values: `credential_state` on providers (`none` / `stored` / `session_only` / `needs_reentry`), env keys needing re-entry on MCP servers, a per-setting secret-state map on the settings API, and `GET /api/uar/secrets/status` (key source and counts) so a host such as the-boss can show "re-enter this key" and "secrets cannot be saved on this machine".
- **BREAKING (data format):** settings rows written by this version hold envelopes, not strings. An older UAR reading them sees an object where it expects a string and fails to load that provider or setting. Rollback requires re-entering secrets.
- **Deviation from D4's wording (operator decision 2026-09-23: approved):** one sealed format inside UAR settings, built on the existing `CredentialEncryption` with keys from the OS credential store, replaces D4's suggestion of moving provider keys into UAR's per-provider `CredentialStore`. The operator also accepted that rollback to an older UAR requires re-entering secrets. Reasoning in `design.md` §2.

## Capabilities

### New Capabilities
- `secrets-at-rest`: which persisted values are secret, the sealed format's guarantees, key sources per OS and the no-key behavior, the plaintext migration, and how secret state is reported without values.

### Modified Capabilities
- None. `provider-model-settings-certification` already requires that stored secrets are never returned in plaintext; that stays true and is not reworded.

## Impact

- **Code:** `src/uar/settings/manager.rs` (write chokepoint, read-side unseal, migration, `mcp` schema marks `env` sensitive), a new `src/uar/security/secrets/` module (envelope, key sources per OS, status), `src/uar/security/credentials/encryption.rs` (AAD variants of encrypt/decrypt), `src/uar/api/providers.rs` (`credential_state`), `src/uar/api/mcp_admin.rs` and `src/uar/admin/mcp.rs` (re-entry view), `src/uar/api/settings.rs` (secret-state map), `src/server.rs` (key-source resolution at startup; status route), SurrealKV rewrite after migration.
- **APIs:** additive fields and one new read-only route. No secret value is ever returned (unchanged).
- **Runtime UX:** providers or MCP servers whose secrets could not be migrated show "needs re-entry" instead of failing silently. On Linux without a Secret Service, keys entered in the UI last until restart; the status route says so.
- **Provider compatibility:** requests to providers are unchanged; the in-memory `ProviderConfig` still carries the decrypted key (`registry.rs:46-47`).
- **Realtime state:** settings change events still fire; payloads carry envelopes or masked values, never plaintext.
- **Dependencies:** an OS credential-store client. The candidate is the `keyring` crate (not in `Cargo.lock` today); its version and features are pinned at implementation under `versions.toml` rules. `security-framework` 3.7.0 and several `windows-sys` versions are already in the lock file transitively.
- **Relation to other work:** D1 makes the-boss pass provider keys as run-scoped credentials and never call `POST /api/uar/providers`; this change protects standalone UAR users and any the-boss user who configured UAR directly. The existing multi-tenant `ProviderService` (`server.rs:972-992`) keeps its own behavior.
- **KBD workflow state:** yes — list this change in the-boss child phase `the-boss-universal-agent-runtime` progress.
