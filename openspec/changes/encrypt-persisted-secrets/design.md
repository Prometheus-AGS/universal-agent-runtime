## Context

See `proposal.md` for why. What exists:

- **One writer.** Every settings write goes through `SettingsManager` and ends in `self.persistence.upsert_setting` (`src/uar/settings/manager.rs:176,226,316,376,523,783,849`). Nothing else in `src/` writes settings rows (checked by searching for `upsert_setting` callers). Reads go through `get_setting`/`list_settings` into an in-memory cache.
- **Schema already names secrets.** Settings types mark secret fields `x-sensitive: true` (`manager.rs:1083,1351,1656,1728,1756,2193,2195,2214,2535`); the settings API masks them on read (`src/uar/api/settings.rs:1-5,361-375`). The `mcp` type (`manager.rs:2837-2855`) declares `servers.additionalProperties: {"type": "object"}` and marks nothing sensitive.
- **Encryption primitive exists.** `CredentialEncryption` does AES-256-GCM with a random 96-bit nonce, output `base64(nonce || ciphertext)` (`src/uar/security/credentials/encryption.rs:81-118`), key from `CREDENTIAL_ENCRYPTION_KEY` only (`:49-79`). No associated data today.
- **Credential store exists but does not fit.** `CredentialStore` is keyed by `(scope, scope_id, provider_id)` (`store.rs:98-127`) and resolved `session → agent → user → system` (`resolver.rs:55-98`), and `ProviderService` is active only when `CREDENTIAL_ENCRYPTION_KEY` is set (`src/server.rs:972-992`). It cannot hold MCP `env` maps or the other sensitive settings.
- **API keys are already safe.** UAR API keys are stored as Argon2 hashes (`src/uar/security/api_keys.rs:33-39`); out of scope.
- **Storage engines keep history.** Overwriting a row does not erase earlier bytes in an append-only or MVCC store until compaction. This matters for "no plaintext findable after migration".

## Goals / Non-Goals

**Goals:**
- No readable secret in any file UAR writes, for every writer, including after migrating old data.
- The key cannot be recovered from the data directory, the database or a backup alone.
- A named, per-OS key source and a stated behavior when none exists.
- Hosts can show which secrets need re-entry.

**Non-Goals:**
- Key rotation and re-encryption to a new key (follow-up; the envelope carries a key id so it is possible later).
- Changing `ProviderService` / multi-tenant credentials.
- Run-scoped credentials from D1 (separate change; they are never persisted).
- Scrubbing secrets from logs or backups that earlier versions already wrote outside the data directory.
- Protecting secrets in process memory.

## Decisions

### 1. Threat model
**Covered:** someone who obtains a copy of the data directory, the SurrealKV/RocksDB files, a Docker volume or snapshot of the remote SurrealDB, a Postgres dump, or a backup of any of these — without also obtaining the user's unlocked OS credential store or the operator's environment key. Also covered: other OS user accounts on the same machine (the credential stores are per user), accidental disclosure (a database attached to a bug report, a data directory synced to cloud storage).

**Not covered:** malware or a script running as the same OS user. On Linux any same-user process can read an unlocked Secret Service item; on Windows any same-user process can call `CredRead`; on macOS a same-user process can at minimum read UAR's memory or replace its binary. Root/Administrator. Secrets in UAR's memory, in outbound provider requests, or in the-boss's own storage. Old plaintext versions inside a remote database engine UAR does not control (Decision 7). Physical recovery of deleted blocks from a disk.

### 2. One mechanism: sealed values inside settings, schema-driven
A value is secret iff its schema node is `x-sensitive`. The `mcp` settings type schema is extended so each `env` value under a server entry is `x-sensitive` (via `additionalProperties`). The exact JSON path follows `McpServerEntry`'s serde shape inside `StoredMcpServer` (`src/uar/api/mcp_admin.rs:21-26`), which this session did not inspect; the schema must match it, and the byte-search test catches a mismatch. A walker visits `properties`, `items` and `additionalProperties`.

Envelope, stored in place of the string:
```json
{"$uar_sealed": 1, "kid": "<16 hex>", "ct": "<base64(nonce || ciphertext || tag)>"}
```
or, when no secret is stored:
```json
{"$uar_sealed": 1, "state": "needs_reentry"}
```
Associated data = `uar-sealed:v1\0<setting key>\0<JSON pointer>`, so a ciphertext only decrypts at the location it was written for. `kid` = first 8 bytes of `SHA-256("uar-kid:v1" || key)`, which identifies the key without revealing it and lets a wrong key be reported as `needs_reentry` instead of a decryption error. `CredentialEncryption` gains `encrypt_with_aad` / `decrypt_with_aad`.

`SettingsManager` gets one private `write_setting` that seals every sensitive plaintext string before `upsert_setting`, and one `unseal` step on load that puts plaintext into the cache and records per-pointer state. All seven call sites use `write_setting`.

Alternative rejected — move provider keys into `CredentialStore` (System scope) as D4's wording suggests. That covers one of the ~10 secret fields and would need a second mechanism for MCP `env` and the rest; the resolver also only runs when `ProviderService` is active. D4's intent — "encrypted or not persisted" — is met with one mechanism built on the same `CredentialEncryption` primitive. This deviation from D4's literal wording is deliberate.
*Operator decision 2026-09-23: approved* — one sealed format inside UAR settings, built on the existing `CredentialEncryption` with keys from the OS credential store (or the operator's environment key), instead of UAR's per-provider `CredentialStore`. Also accepted: rolling back to an older UAR requires re-entering secrets.

### 3. Key sources, per OS
Resolution at startup, first match wins, recorded in status:

1. **`env`** — `CREDENTIAL_ENCRYPTION_KEY` (32 ASCII bytes or 64 hex, existing parser `encryption.rs:49-79`). For servers and containers, where the orchestrator's secret store supplies it. The key lives in the process environment, not the data directory.
2. **`os_keystore`** — one generic secret per data store, service `universal-agent-runtime.secrets`, account `data-key/<store_id>`:
   - **macOS (x64, arm64):** login Keychain generic-password item. Protected by the user's login password; readable without a prompt by the application identity that created it. Access through `keyring` (Apple backend) or directly through `security-framework`.
   - **Windows (x64, ARM64):** Credential Manager generic credential (`CRED_TYPE_GENERIC`), local-machine persistence. Credential Manager stores the blob DPAPI-encrypted under the user's logon secret. A native UAR Windows service (native-service-deployment) runs under a service account; whether its Credential Manager store is usable without a loaded profile was not verified this session — if it is not, it falls to `none` and the status route says so.
   - **Linux:** Secret Service over the D-Bus session bus (GNOME Keyring, KWallet with Secret Service support, KeePassXC), item in the default collection with attributes `{service, account}`. Requires a session bus and an unlocked collection.
3. **`none`** — no key (Decision 4).

`store_id` is a random UUID stored as the non-secret setting `secrets.store_id` on first use. It keys the credential-store entry so two data directories on one machine get two keys, and a copied database on another machine finds no key.

Key creation: on the first secret write with no existing key, generate 32 bytes from the OS RNG, write the entry, read it back and compare before sealing anything. If the write fails, fall to `none` for this process and log `secrets.key_source_unavailable` with the backend name and error kind (no values).

Crate: `keyring` gives one API over the three backends. It is not in `Cargo.lock`; the exact version and feature names (Apple, Windows and a Secret Service backend that does not need `libdbus` at build time) are chosen at implementation under `versions.toml` rules. This session did not verify its feature names. If `keyring` is unsuitable, direct `security-framework` (already in the lock at 3.7.0 transitively), `windows-sys` Credential Manager calls, and a Secret Service client are the fallback plan.

Rejected: a key file in or next to the data directory (copy tools and backups take both; this is the round-3 CRITICAL); deriving the key from machine identifiers (guessable, and it breaks on hardware change); the-boss passing the key over stdin (makes standalone UAR different from the sidecar and moves key persistence into the-boss for no gain — the OS store is already per user).

### 4. No key source: session-only, then re-entry
A new secret is kept only in the in-memory registry/cache and the persisted entry holds the `needs_reentry` marker. The response and status report `session_only`. This matches D4's "encrypted or not persisted" and keeps behavior honest on a headless Linux box.

### 5. Migration
Runs in `SettingsManager::initialize` (`manager.rs:133`) after settings types exist and before provider/MCP hydration (`server.rs:1040-1048`), so the registry never loads plaintext:

1. Scan all rows; walk sensitive nodes; classify each as `plaintext`, `sealed`, `marker`, or empty.
2. `plaintext` + key → seal. `plaintext` + no key → marker. Write through `write_setting`.
3. If any row changed and the store is embedded SurrealKV: export the database with the SDK's `export` (present in `surrealdb` 3.2.4, `method/mod.rs:1410`) to a temporary file inside the data directory, import into a fresh sibling directory with `import` (`:1441`), swap directories atomically (rename), delete the old directory and the export file. The export happens after sealing, so it holds no plaintext. Whether SurrealKV 3.2.4 offers a compaction that provably drops old versions was not checked this session; if it does, it may replace the export/import swap. The byte-search test decides either way.
4. Log one line: `secrets.migration sealed=N stripped=M already_sealed=K failed=F rewritten=<bool>`.
5. A row that cannot be parsed is left untouched, counted in `failed`, and startup continues; the affected provider/MCP server reports `needs_reentry` only if its value is not plaintext. A `failed > 0` result is logged at `error`.

### 6. Reporting surfaces
- `ProviderView` (`src/uar/api/providers.rs:463-490`) gains `credential_state`; `credential_configured` stays and is true only for `stored` and `session_only`.
- MCP public views (`mcp_admin.rs:137-173`, and the `admin/mcp.rs` list) gain `envKeysNeedingReentry`.
- The settings API response for a type with sensitive fields gains `secret_state: { "<json pointer>": "stored" | "session_only" | "needs_reentry" }`; values stay masked.
- `GET /api/uar/secrets/status` returns `{ key_source, backend, store_id, counts }`. Behind the same authentication as other `/api/uar` routes (and the sidecar guard in sidecar mode).

### 7. Old plaintext outside embedded storage
- **Remote SurrealDB (Docker):** UAR cannot compact the server's storage; superseded plaintext versions may remain in the server's volume until the server compacts. The migration logs `secrets.migration.remote_history_not_purged` once. Operator remedy: export and re-import the namespace. For the-boss this exposure is small: D1 never calls the providers API, and `sidecar-launch-security` disables global MCP servers in sidecar mode.
- **Postgres:** dead tuples and WAL may keep old values until `VACUUM`/WAL recycling; migration runs `VACUUM FULL` on the settings table once when it changed rows. Archived WAL and dumps are outside UAR's control. Documented, not tested by this change.

## Risks / Trade-offs

- [macOS Keychain prompts after an update] → a Keychain item's ACL trusts the creating application's code identity; an unsigned or ad-hoc-signed `uar-sidecar` gets a new identity each build, and macOS asks the user to allow access after every update. Mitigation: sign release binaries with a stable Developer ID; the-boss release pipeline must sign the sidecar. Unsigned development builds will prompt. Not verified this session.
- [Linux desktop without an unlocked keyring] → `none`; secrets are session-only. The status route makes it visible; the-boss's settings UI (D3) must show it.
- [Keystore entry deleted or user profile reset] → every secret becomes `needs_reentry`. Availability loss, not a leak.
- [Export/import swap fails midway] → the old directory is deleted only after the new one opens and a row count matches; on any failure the old directory stays, the log reports `rewritten=false` and an error, and the byte-search property is not met for that run. Startup still succeeds.
- [Older UAR reads an envelope] → breaking data format (proposal). Rollback requires re-entering secrets. Accepted by the operator on 2026-09-23.
- [Same-user malware] → not covered (Decision 1). This is the honest limit of any scheme without a user-entered passphrase.
- [Seeding re-imports a secret from env/YAML on every boot] → seeding writes only rows that do not exist (`manager.rs:767-769`), and it now seals; the env value itself is the operator's responsibility.

## Migration Plan

1. Ship sealing, key sources, migration and status together.
2. First start after upgrade: migration seals (or strips) and rewrites embedded storage; one log line with counts.
3. Rollback: an older binary cannot read envelopes. Before rolling back, the operator re-enters secrets after rollback; UAR offers no downgrade path.

## Open Questions

1. Should `session_only` secrets survive a UAR restart when the host (the-boss) re-sends them? That is the D1 run-scoped credential path, not this change; recorded so the two do not get conflated.
