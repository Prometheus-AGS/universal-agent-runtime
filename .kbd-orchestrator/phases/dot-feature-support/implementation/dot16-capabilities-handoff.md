# Dot16 source handoff

Dot Execute, 2026-10-12. Source implementation only. No tests, compiler, build,
lint, formatter, validator, probe or product review gate executed.

## Changed files and wiring

- `src/uar/api/assistant_capabilities.rs`: authenticated per-operation installed
  inventory at `/api/uar/assistant-capabilities` and owner-scoped stored tool
  outcome projection at `/api/uar/runs/{run_id}/tool-outcomes`.
- `src/uar/api/capabilities.rs`: additive link to that inventory.
- `src/uar/api/mod.rs` and `src/server.rs`: module/route registration with the
  actual schedule/followup controllers, current AppState and successfully
  registered supervisor presence. Schedule/followup startup and native tools
  are attached before observer start/listening.
- `src/uar/persistence/mod.rs` and Postgres/Surreal providers: explicit durable
  receipt support flag, authored by the A2A worker during its shared writer
  window. Default false; Postgres true; Surreal uses its existing attested/local
  durable predicate.
- `src/uar/persistence/agent_threads.rs`, `src/llm/orchestrator.rs`: optional
  full principal provenance on newly acquired canonical receipts, included in
  immutable equality. Existing storage partition keys stay unchanged.
- `docs/operations/assistant-capabilities.md`: API examples, profile/entrypoint
  limits, authority, operational recovery limits and uncertified status.
- `tests/fixtures/dot16_capability_inventory.py`: authored real-host discovery,
  operation coverage helper and exact-principal receipt isolation assertions.

The receipt endpoint queries the existing subject partition, then authorizes
each row by the newly persisted verified issuer/tenant/subject key. A historical
row without that key requires an exact live run owner; it is not exposed cold.
Absent historical outcome is unknown, never success. Raw result bytes, inputs,
owner provenance and authority are excluded. This is a named disclosure trust
boundary, not a rewrite of execution storage. Optional JSON provenance requires
no table migration and never upgrades a historical row's authority.

## Availability contract

Controller `supported()` methods call exactly the operation store predicates.
Schedules require catalog, durable instances, observers and accounting; followups
require catalog, instances and observers. Memory service availability is separate
from the runtime catalog and reports only registered memory tools. Native entries
are intersected with actual descriptors. Unsupported MCP/embedded methods are
explicitly unavailable. HTTP runtime approval history retains its existing
`durable:false` indication on a volatile host; this inventory advertises only the
durable contract, not that preexisting live-only behavior.

Supervisor routes are advertised only after configured qualified-controller
registration. Native reboot and Kubernetes affected-node inhibition are explicit
limits. Discovery never bypasses current principal, workspace, grant, revision,
approval, consent, budget, epoch or storage checks, and never certifies a release.

## Deferred acceptance

Dot17 must supply concrete real-path scenarios to `exercise_inventory`; missing
advertised entries raise rather than skip. The helper alone is not completion
evidence. Cold-restart receipt checks require real externally observed tool calls
and a second authenticated issuer/tenant with the same subject. Both deployed
profiles, negative unsupported backends, native/MCP paths and installed-artifact
identity remain unexecuted. No direct embedded parity is claimed.

No unrequested UI, queue, scheduler or execution authority was added. The
uncomfortable thing: this integration composes concurrently authored Rust and
has not compiled; runtime claims remain unverified until the entire phase's
production inventory is complete and the serial real-path gate runs.

## Integration corrections
Native memory_capture_policy is read-only by design; policy.update advertises HTTP/MCP only, never that native read tool. src/uar/tools/memory.rs disabled-service branches previously returned successful empty/disabled payloads; they now return explicit memory_disabled errors, satisfying the operation unsupported contract without changing enabled behavior. This was observed in source while matching actual native method semantics to discovery. No gates ran.
