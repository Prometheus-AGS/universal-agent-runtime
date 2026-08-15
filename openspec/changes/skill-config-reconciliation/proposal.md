## Why

Operator requirement, 2026-08-12: when the skill configuration files change,
those changes SHALL be merged into the database on restart, **including removing
skills that are not built-in and have been deleted from the configuration
files**. Today nothing reconciles: the filesystem provider
(`FilesystemStorageProvider`, mounted as `fs-skills` at `src/server.rs:472-476`)
loads what it finds, and a skill whose file was deleted simply stays in the
database forever.

### Removal is a tombstone, not a delete — operator decision, 2026-08-12

Reconciliation is the only irreversible operation in the skill subsystem, and it
is driven by *file absence*, which has innocent causes: a mis-mounted volume, a
partial checkout, a failed deploy. A hard delete on a wrongly-empty directory
destroys user data with no recovery.

Removed skills are therefore **tombstoned**: retained in storage, marked removed,
excluded from matching and from default listings, and restorable — either by
restoring the configuration file or by an explicit restore.

### The source discriminator must survive filesystem reload

Reconciliation must distinguish config-provisioned skills from user-created ones,
or it would tombstone every skill a user authored through the API. **No new field
is needed.** `Skill::provider_id` (`skills.rs:63-65`) records the source, but the
filesystem loader must preserve the distinction across a cold reload. API-created
skills are written beneath the reserved `skills/dynamic/` directory; files under
that directory load as `api`, while other files load as `fs-skills`:

| `provider_id` | Source | Reconciled? |
|---|---|---|
| `fs-skills` | Configuration files (`server.rs:472-476`) | **Yes** — in scope |
| `builtin` | Skill pack (`builtin_loader.rs:392`) | No — built-ins are never removed |
| `api` | Created via REST/UI (`service.rs:278`) | No — not config-managed |
| `wasm` | WASM runtime (`wasm_runtime.rs:326`) | No |

This is the whole safety argument for the change, so it is stated in the spec
rather than left to the implementer.

## What Changes

- On startup, after providers load, reconcile the configuration-file source
  against stored skills: skills present in config are upserted; stored skills
  whose `provider_id` marks them config-provisioned and which are **absent** from
  config are tombstoned.
- Skills with any other `provider_id` — built-in, api, wasm — are never
  tombstoned by reconciliation.
- Tombstoned skills are excluded from matching and from default listings, and are
  restored on the next startup where their configuration file is present again.
- Reconciliation logs every tombstone at info level with the skill id and the
  reason, so an operator can tell a deliberate removal from a mis-mounted volume.
- **Refuse to reconcile when the configuration source resolves to zero skills**
  while the database holds config-provisioned ones. That is far more likely to be
  a broken mount than a deliberate removal of everything. Log an error and skip
  rather than tombstoning the entire catalogue.
- Correct filesystem cold reload so the reserved `skills/dynamic/` API persistence
  directory reloads with `provider_id = "api"`; configuration files outside it
  continue to reload with `provider_id = "fs-skills"`.

## Capabilities

### New Capabilities
- `skill-config-reconciliation`

## Impact

`src/uar/runtime/skills/service.rs`, `src/uar/runtime/skills/storage/`,
`src/uar/domain/skills.rs` (tombstone marker), startup path in `src/server.rs`
and `src/embedded.rs`.

## Non-goals

- Reconciling agents, knowledge bases, or providers. Skills only.
- Hard deletion of any skill by reconciliation. User-initiated delete of a
  user-created skill is unchanged and remains a real delete.
