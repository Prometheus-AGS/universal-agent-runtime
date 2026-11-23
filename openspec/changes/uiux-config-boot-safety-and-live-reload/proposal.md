## Why

A bad edit to `config.yaml` does not brick UAR when it is made, but it bricks the next
start. `ConfigManager::reload` already loads, validates and rejects a bad file while keeping
the current configuration (`src/config_manager.rs:103-130`). Boot has no such safety net:
`ConfigManager::load` returns the error and `main.rs:94-99` ends the process with "Failed to
load configuration". There is no last-known-good copy on disk. A desktop or sidecar operator
who cannot fix the file has an application that does not start.

Two more boot paths stop startup on bad stored values:

- the persisted `llm.embedding.*` rows. `get_typed::<usize>(..).await?` and an
  `anyhow::ensure!` on the dimension at `src/server.rs:1251-1264` end startup on a
  wrong-typed or out-of-range stored value;
- a malformed `provider.{id}` row. `load_provider_configs_from_db` fails the whole batch on
  the first undecodable row (`src/uar/settings/manager.rs:1168-1187`), so every provider
  hydration is skipped and only a `tracing::error!` line says so (`src/server.rs:1173-1180`).

Operator decision 1 (2026-11-19, `.prometheus/decisions.md`) says the database wins over
YAML for operator-edited values. The provider seed does not implement that. It skips every
existing provider row (`src/uar/settings/manager.rs:975-978`), so a model added to a YAML
provider never reaches an existing deployment, and it stamps `updated_at: Some(now)` on a
row it just seeded from YAML, so the next boot cannot tell that row from an operator edit.
Hot reload is wider than the seed: a successful reload swaps `ConfigManager.current`, but
`server.rs:490` took a boot snapshot and nothing re-reads the swap, so the reload endpoint
(`server.rs:3724-3752`) reports success with no effect on settings or providers.

The risk this change must not introduce (unresolved review finding critical 4): a
last-known-good fallback that boots an older, weaker security posture because a tightened
one failed validation. Boot validation includes deliberate security refusals
(`src/config.rs:931-946`, `src/config.rs:895-902`). The fallback must never override one,
and what it saves to disk must never contain environment- or vault-merged secrets
(`AppConfig` derives only `Deserialize`, `src/config.rs:243`).

## What Changes

Build items 7, 8 and 9 of the analysis.

- **Last-known-good (LKG) copy.** After a boot reaches readiness, save the exact YAML source
  bytes that boot loaded, plus a SHA-256 digest record, to a UAR state directory with
  owner-only permissions and an atomic write. Nothing else is saved: no environment,
  command-line, vault or defaulted value.
- **Fail-closed boot fallback.** Fall back to the LKG bytes only when the configured YAML
  file cannot be read or parsed into `AppConfig`. A configuration that parses and then fails
  any validation, refusal or vault step never falls back. The LKG run must pass the full
  validation pipeline with the same command-line and environment layers or the original
  error stands. A remote deployment profile never uses the fallback. Boot reports the
  fallback through a log line and a new admin status route.
- **Tolerant persisted values.** A malformed or out-of-range persisted embedding value, and
  a malformed `provider.{id}` row, are skipped individually with a visible status entry. The
  stored rows are never edited or deleted. Governance fail-closed handling and the local-only
  provider audit are unchanged.
- **Live config reconcile.** After a successful reload, one reconciler re-runs the same
  per-key decision the boot pass uses, so config-owned rows follow YAML and operator-edited
  rows keep their value and are reported as drift. A reload reports what it applied, what
  needs a restart and what drifted.
- **Provider seed aligned with DB-wins.** Seeded rows are config-owned. YAML models missing
  from an existing row are merged in. Operator-edited fields are retained and reported as
  drift. No data migration; existing rows stay on the conservative side.
- **Drift resolution.** Admin routes list drift with both values and resolve it by adopting
  the config value on an explicit request.

Out of scope: the console surfaces that display status, drift and the reload report (change
`uiux-settings-truthful-status-and-removals` and the settings regroup, which consume the
routes added here); changing `llm.default_provider` or any model-selection behavior (change
`uiux-system-model-live-default`); deleting stored rows of namespaces removed from the UI;
any change to the vault feature; any edit to dependency pins.

## Limitations and unresolved decisions

- A remote deployment can still fail to start on a bad YAML: the fallback is never used for the
  remote profile. This contradicts "YAML changes never prevent startup" for remote deployments
  and is an explicit limitation, put to the operator in design.md under Unresolved decisions.
  Recommendation: narrow the release claim to local profiles.
- A YAML that is both unparseable and intended to tighten security boots the previous
  validated config on the local profile. Mitigations are in design.md (Risks).
- Rows stamped by the old seed are treated as operator-owned; no migration.

Frozen oracle fixture (written now, not editable by the implementer): `fixtures/cfg-oracle.json`.

Review round 1 additions: SCN-CFG-008 (drift, reload and resolve never expose a secret),
a second refusal edit and a second invalid-input control in the other scenarios, and the
reload report's `not_applied` list (keys with no reader; a YAML-at-boot key edited in the YAML is `restart_required`), and `failed_source_path` in boot status for the console's rejected-file banner.

## Capabilities

### New Capabilities

- `configuration-boot-safety`: last-known-good configuration, boot fallback eligibility,
  tolerant persisted values, live reload reconcile, drift and reload reporting.

### Modified Capabilities

- `provider-model-settings-certification`: `Provider bootstrap preserves persistent
  settings authority` gains seeded-row ownership, additive YAML model merge and drift
  reporting for operator-edited rows.

## Impact

Rust (single-writer assignment required before implementation; none of these paths is in the
`uar-state` ownership list): `src/main.rs`, `src/config.rs`, `src/config_manager.rs`,
`src/server.rs`, `src/uar/settings/manager.rs`, `src/uar/settings/provider_sync.rs`,
`src/uar/settings/schema.rs`, `src/uar/api/settings.rs`,
`src/uar/api/administration_capabilities.rs`, `src/uar/api/openapi.rs`.

Interfaces added (additive): `GET /api/uar/settings/boot-status`, `POST
/api/uar/settings/reload`, `POST /api/uar/settings/drift/{key}/resolve`, and additive fields
on `GET /api/uar/settings/drift`. `POST /.well-known/uar-config/reload` keeps its response
body and gains the real effect. No schema migration; no new Cargo dependency (`sha2` is
already declared at `Cargo.toml:631`).

Depends on `uiux-realtime-topics-and-replica-refresh` for one behavior only: after a
reconcile writes provider rows, the live registry is refreshed through that change's
`ReplicaState::refresh_now`.

## Related open changes

- `make-config-authoritative-on-boot` — **supersede, then archive.** Its R3 product decision
  ("env/YAML authoritative on every boot") was reverted deliberately; operator decision
  1 (2026-11-19) supersedes it. It has no spec deltas and its ledger describes reverted code.
- `config-hot-reload-vault` — **continue to closure, then archive.** This change builds on
  its `ConfigManager`; its only open tasks are 7.5/7.6 bookkeeping. It owns the `vault`
  feature, which this change does not touch.
- `config-rs-schemars-migration` — **continue.** Owns the `AppConfig` schema and secret
  types; this change depends on `AppConfig` staying `Deserialize`-only.
- `uiux-settings-truthful-status-and-removals` — **continue, coordinate.** It owns the console
  screens for boot status, drift with one-click resolution and the reload report, and the
  apply-class table this change's reload report uses; this change owns the routes and shapes.
  Agree the shapes in design D6-D8 before either is implemented.
- `uiux-system-model-live-default` — **continue, coordinate.** Both touch
  `src/uar/settings/manager.rs` provider and default-provider code; assign one writer.
