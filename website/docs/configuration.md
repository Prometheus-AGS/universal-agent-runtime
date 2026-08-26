---
sidebar_position: 10
title: Configuration Authority
description: Resolve UAR settings by source, precedence, lifecycle, secret, persistence, and profile boundary.
source_records:
  - docs/PROVIDER_CONFIGURATION.md
current_authority: /docs/configuration
---

# Configuration authority

## Boundary statement

**The effective configuration is the validated merge held by the running
process, not whichever YAML file or environment variable a reader inspected.**
Source precedence, runtime settings persistence, reload behavior, and feature
composition decide whether a value takes effect.

## File selection

UAR selects at most one YAML file in this order:

1. `--config <path>` or `CONFIG_FILE=<path>`;
2. `./config.yaml`, when present;
3. the user's `.uar/config.yaml`, when present;
4. no file, leaving compiled defaults and other configured sources.

The selected local file is watched. Content changes are debounced and rebuilt
through the same configuration loader. An explicit `POST
/.well-known/uar-config/reload` triggers the same reload and requires the admin
header when settings mutation protection is enabled.

## Precedence

For settings represented by CLI flags, the documented priority is:

1. explicit CLI argument;
2. structured `UAR_...` environment value;
3. a supported legacy environment value;
4. provider-specific shortcut environment key, where applicable;
5. selected YAML file;
6. compiled default.

LLM settings follow the explicit chain documented in current source: CLI,
`UAR_LLM__*`, legacy `LLM_*`, provider shortcut keys for credentials, YAML, then
defaults. A provider entry persisted through the settings/provider APIs has its
own database reconciliation rules and must not be mistaken for a process env
override.

## Structured environment names

Nested keys use `UAR_SECTION__KEY`: one underscore after `UAR`, then two
underscores between path components.

```bash
UAR_SERVER__PORT=1906
UAR_LLM__MODEL=provider/model-name
UAR_PERSISTENCE__PROVIDER=surreal
UAR_PERSISTENCE__DATABASE_URL=surrealkv://./data/uar-db
```

Use placeholders in checked-in examples and inject secrets from the process
environment or a supported secret backend. The schema describes a secret
field's shape, not its value.

## Schema and exact build authority

`GET /.well-known/uar-config` returns JSON Schema generated from the running
build's `AppConfig`. Use it to inspect sections, field types, and build-specific
shape. It does not reveal the effective secret values.

The packaged settings UI and `/api/uar/settings` expose registered setting
types, current persisted values, source metadata, and drift. Mutating settings
requires the configured admin boundary. Not every value can recompose a server
that is already listening; a successful settings write is not proof that every
subsystem reinitialized.

## Provider and model configuration

The default model uses `provider/model`. A bare model in a request resolves
against that configured default provider. Keep provider credentials out of
committed YAML:

```yaml
llm:
  model: "provider/model-name"
  api_key_env: "PROVIDER_API_KEY"
  protocol: "auto"
  timeout_secs: 60
```

`api_key_env` names the environment variable to read. Well-known provider
shortcut variables can also populate the runtime provider-key map. The model
catalog is discovery metadata; a provider becomes callable only when its
configuration, credentials, network, and selected model all work.

See [Provider configuration](./providers/configuration.md) for the UI/API
workflow and [Credentials](./security/credentials.md) for per-user storage.

## Persistence and feature requirements

The packaged server default is SurrealKV at an on-disk `surrealkv://` location.
Example files cover loopback embedded SurrealKV, remote SurrealDB, and remote
PostgreSQL. Keep provider and URL explicit for a deployment whose data location
must not drift.

PostgreSQL requires the `postgres-backend` Cargo feature. `server-full` includes
the embedded Surreal backend through `minimal`; it does not add PostgreSQL.
`embedded-mobile` selects no built-in database and requires the host to supply a
`PersistenceLayer`.

Vector dimensions must match the embedding implementation that produced stored
vectors. Changing the dimension without a re-embedding plan makes existing
vector data incompatible.

## Security and secret handling

With JWT required, UAR rejects the compiled fallback signing secret. Configure
a deliberate secret or JWKS verifier plus any required issuer and audience.
Anonymous mode is only for a trusted local process bound to loopback:

```yaml
server:
  host: "127.0.0.1"
security:
  jwt_required: false
```

When `security.settings_mutation_auth_required` is enabled, configure a
non-empty `security.settings_admin_key` and send that exact value in the
`X-UAR-Admin-Key` header. UAR rejects startup if protection is enabled without
an admin key. The packaged loopback-only LaunchAgent disables this boundary;
do not copy that weaker setting to a network-exposed deployment.

Do not combine anonymous mode with a public or non-local listener. Per-user
provider credentials require a valid credential-encryption key; without it,
that service is unavailable and operator environment/configuration remains the
separate fallback.

## Reload boundary

The config manager atomically swaps a newly validated snapshot after a watched
file change or explicit reload. Existing request snapshots remain valid until
their readers release them. `--strict-config` or `UAR_STRICT_CONFIG=true`
rejects a reload whose effective snapshot differs from startup.

Reload acceptance proves only that the snapshot parsed and passed configured
guards. Listener host/port, feature-gated capabilities, database engine,
process-level crypto provider, and other composition-root resources can require
a controlled restart. Use the settings UI's source/drift information and verify
the affected behavior after any change.

## Profile limits

`minimal` and `server-full` load the server configuration and selected
persistence. Only `server-full` carries the complete release feature set.
`embedded-mobile` uses host construction rather than the server's YAML/CLI
composition and owns persistence, inference, transport, and lifecycle. No
configuration result transfers silently between profiles.

Next: [Installation](./installation.md).
