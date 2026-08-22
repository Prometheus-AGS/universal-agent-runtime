## Why

GAP-05 (`docs/SPECIFICATION.md:445`) says `register_builtins` is called "only
from `server.rs`" and that embedded boots with "an empty skill registry" at
"0% capability." **Grounding on current `main` shows all three claims are now
wrong, in different ways.** The spec was written before commit `fdd69a2f`
(*"persist builtin skills when no embedder is configured"*).

What is actually true, verified in code rather than from doc comments:

| Spec claim | Verified |
|---|---|
| Called only from `server.rs:436` | **Two** sites: `server.rs:454` and `:517`. Neither line number matches |
| Builtins are not persisted | **They are.** `registry.rs:69-99` — `register` writes through `db.save_skill` |
| Embedded registry is empty | **Only on a fresh database.** `embedded.rs:365-371` registers a `DatabaseStorageProvider` and calls `initialize()`, so it loads any persisted builtin |

The real defect is narrow: **`embedded.rs` never calls
`discover_builtin_skills()` / `register_builtins`**, so on a *fresh* embedded
database no builtin ever enters the system. An embedded host that has never run
the server against the same database has no built-in skills.

This matches the 2026-08-09 judge ruling: the registry is empty of *built-ins*
always, empty *overall* only on a fresh device.

## What Changes

- Call the builtin loader from the embedded runtime bootstrap
  (`src/embedded.rs`, alongside the existing `DatabaseStorageProvider`
  registration) so builtins are discovered and registered — and therefore
  persisted — on a fresh embedded database.
- Register **before** `initialize()` would re-read them, or make registration
  idempotent, so a second boot does not duplicate rows.
- Honour `seed_defaults`: the embedded builder already gates
  `seed_builtin_agents` and `ensure_default_knowledge_base` on it
  (`embedded.rs:355-358`). Builtin skills follow the same switch rather than
  introducing a second convention.
- Amend `docs/SPECIFICATION.md:445` to the verified wording: builtins are absent
  on a fresh embedded database, not "0% capability".

## Capabilities

### New Capabilities
- `skill-builtin-availability`

## Impact

`src/embedded.rs`, `docs/SPECIFICATION.md`.

**No storage change.** Persistence already works; this change only ensures the
embedded path reaches it.

## Non-goals

- Scoped enable/disable — `skill-scoped-governance` owns that.
- Config-to-database reconciliation — `skill-config-reconciliation` owns that.
- Any change to how builtins are discovered on the server path, which works.
