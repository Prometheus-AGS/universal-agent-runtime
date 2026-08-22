# B5 deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Provenance: PASS after operator-approved correction. Files under reserved `skills/dynamic/` reload as `api`; configuration files outside it reload as `fs-skills`; only API-managed skills are written into the reserved directory.
- Reconciliation: PASS. Configuration additions and changes are persisted, absent `fs-skills` records are tombstoned without deletion, and reappearance clears the marker.
- Cold restart: PASS. Separate seed, change, remove, and restore child processes opened one SurrealKV directory and preserved the durable row and scoped agent disable.
- Visibility: PASS. Tombstoned skills are absent from default lists, keyword/vector candidates, and matching, but remain retrievable from durable storage for audit and restore.
- Origin safety: PASS. A skill created through the real API service path and a built-in survived empty-source reconciliation. Ignoring `provider_id` produced exit 101 at the API survival assertion.
- Empty-source fail-safe: PASS. Config, API, and built-in records remained active when the filesystem source was empty. Removing the guard produced exit 101.
- Attribution: PASS. The tombstone log emitted `skill_id=config-removed reason="absent_from_configuration"`.
- Negative-control restoration: PASS. Both inversions restored to service SHA-256 `96920a864e47f267126849b245d77c1cfd8ff52b2fc99d5cc790de5b05a74472` and diff SHA-256 `21e890e94eb60b7a0b731e9b3e86f6ee745a7a65d26e62a68b7a4a7be2b0eb6d` before 11 service tests passed.
- Tier 0: PASS. Package check and package/library/no-deps Clippy exited 0; Clippy returned to the 573-warning pre-B5 baseline.
- OpenSpec: PASS. `openspec validate skill-config-reconciliation --strict` exited 0.
- Scope: PASS. Behavioral edits remain within the operator-amended Track B surface.
- Tier 2: NOT RUN. The phase command remains deferred until B5 artifact review and commit.

Literal evidence is retained in:

- `openspec/changes/skill-config-reconciliation/evidence/positive-verification.md`
- `openspec/changes/skill-config-reconciliation/evidence/negative-controls.md`

Uncomfortable result: the original plan's safety discriminator was not durable.
API-created files and configuration updates could enter the same filesystem
namespace and reload with the wrong source. Reconciliation could not safely be
implemented until the reserved dynamic namespace became API-only on both read
and write paths.
