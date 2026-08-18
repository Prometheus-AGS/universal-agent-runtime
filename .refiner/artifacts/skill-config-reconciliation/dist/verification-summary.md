# B5 deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Provenance: PASS after operator-approved correction and independent-review repair. The filesystem write boundary rejects non-API skills, and a real configuration source deterministically wins over any stale dynamic copy with the same ID.
- Reconciliation: PASS. Configuration additions and changes are persisted, absent `fs-skills` records are tombstoned without deletion, and reappearance clears the marker.
- Cold restart: PASS. Separate seed, change, remove, and restore child processes opened one SurrealKV directory and preserved the durable row and scoped agent disable.
- Visibility: PASS after independent-review repair. Tombstoned skills are absent from default lists, refresh responses, keyword/vector candidates, and matching, but remain retrievable from durable storage for audit and restore. Vector search requests enough ranked rows to filter tombstones before applying its visible top-five limit.
- Origin safety: PASS. A skill created through the real API service path and a built-in survived empty-source reconciliation. Ignoring `provider_id` produced exit 101 at the API survival assertion.
- Empty-source fail-safe: PASS after critic-required evidence repair. Config, API, and built-in records remained active when the filesystem source was empty, and the focused command observed the refusal at `ERROR` with `stored_config_skills=1`. Removing the guard produced exit 101.
- Attribution: PASS. The tombstone log emitted `skill_id=config-removed reason="absent_from_configuration"`.
- Negative-control restoration: PASS. The two original reconciliation inversions and four reviewer-found inversions all exited 101. After the test-only log subscriber, final source hashes are registry `9956eca1...c6a14`, service `97432069...ca0a`, filesystem `dbb88829...79585`, with combined diff `f6f9e872...ac72f`; the restored skills slice observed 46 passing and 0 failed.
- Tier 0: PASS. Package check and package/library/no-deps Clippy exited 0; Clippy returned to the 573-warning pre-B5 baseline.
- OpenSpec: PASS. `openspec validate skill-config-reconciliation --strict` exited 0.
- Scope: PASS. Behavioral edits remain within the operator-amended Track B surface.
- Tier 2: NOT RUN. The phase command remains deferred until B5 artifact review and commit.

Literal evidence is retained in:

- `openspec/changes/skill-config-reconciliation/evidence/positive-verification.md`
- `openspec/changes/skill-config-reconciliation/evidence/negative-controls.md`

Uncomfortable result: both the original plan and the first refined artifact
overstated the safety boundary. The first independent review proved tombstones
could still suppress vector results and leak through refresh. The judge then
proved the dynamic write boundary accepted non-API rows and stale upgrade files
could win by traversal order. Those were reachable defects, not speculative
concerns.

The original plan's safety discriminator was not durable.
API-created files and configuration updates could enter the same filesystem
namespace and reload with the wrong source. Reconciliation could not safely be
implemented until the reserved dynamic namespace became API-only on both read
and write paths.
