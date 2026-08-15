# A2 deterministic verification summary

Profile scope: `server-full` only. These results transfer to no other profile.

- Verified identity: PASS. Tenant identity is populated from the verified principal and remains distinct from the subject; body, query, and header tenant values cannot override it.
- Task partitioning: PASS. Focused task ID, context ID, cancel, and gRPC assertions passed with paired exit-101 inversions.
- Required tenant: PASS. JWT-required HTTP and gRPC surfaces reject tokens with no verified tenant; removing either guard made its assertion fail.
- Live C-21: PASS. The published exclusion is gone. The pinned phase command observed `l3_c21_a2a_tasks_are_partitioned_by_verified_tenant ... ok` within 29 passing and 0 failed.
- Live negative control: PASS. Ignoring the tenant key in `TaskStore::get` made the exact C-21 case exit 101 at the cross-tenant read assertion (`left: Null`, `right: -32001`). Source SHA-256 restored to `8159395c...ebd4`; the restored diff is empty.
- Tier 0 and Tier 1: PASS as recorded in the A2 verification artifact. The final phase proof introduced no source change.
- OpenSpec: PASS. Strict validation is required again after the completed task ledger.
- Scope: PASS. A2 changes only verified claims/security and A2A task surfaces; runs, memory, and knowledge bases are unchanged.

Literal evidence is retained in:

- `openspec/changes/gap-03-a2a-tenant-partitioning/verification.md`
- `openspec/changes/gap-03-a2a-tenant-partitioning/evidence/fail-closed-negative-controls.md`

Uncomfortable result: the implementation had passed focused controls for a
day while the published live exclusion remained unresolved. Compilation proved
the replacement test existed; only the phase run proved the server behavior,
and only the inversion proved that assertion could detect lost isolation.
