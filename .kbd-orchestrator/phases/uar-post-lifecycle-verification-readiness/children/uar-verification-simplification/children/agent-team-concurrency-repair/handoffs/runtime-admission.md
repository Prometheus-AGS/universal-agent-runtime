# Runtime admission handoff — task 1.1

Changed paths:

- `src/uar/runtime/skills/service.rs`
- `src/uar/runtime/manager.rs`

The startup registers configured roots as `filesystem-skills-{index}`, while
the prior reconciliation only searched `fs-skills`. Reconciliation now includes
those configured filesystem providers, writes their durable records with the
stable `fs-skills` ownership, and recognizes either identity when preserving
existing records or tombstoning removed configuration. API-owned `dynamic/`
skills remain excluded, and existing global/scoped enabled settings are copied
from the durable record before an updated source record is written. Later
configured roots retain the initialization overwrite order for duplicate IDs.

The run-admission client error remains the existing safe
`skill_selection_unavailable` category. The manager now records the internal
selection error category in a warning, without exposing storage detail to the
client.

Scoped baseline before this repair already contained unrelated edits in all five
assigned source files. This task added only the configured-provider helpers and
reconciliation change in `service.rs`, plus the error-category warning in
`manager.rs`; it did not edit `src/server.rs`, `registry.rs`, or
`service/selection.rs`.

No verification commands ran: the product verification gate remains CLOSED.
The source-only implementation has not established compilation, migration
behavior against the retained store, or TEAM-01 behavior. The exact historic
inner selection error is still unavailable; the diagnosis is the observed
provider-identity mismatch and missing durable selection rows.
