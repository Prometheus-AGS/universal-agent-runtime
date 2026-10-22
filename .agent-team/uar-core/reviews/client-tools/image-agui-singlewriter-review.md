# Task 13.30 independent source review

Disposition: ready for the lead's authorized production gate; no blocking source findings in the assigned diff against c065. This is not compilation or functional acceptance.

Phase: `uar-post-lifecycle-verification-readiness`, execute. Verification lock: CLOSED for this reviewer. Commands were source inspection only (`cat`, `sed`, `rg`, `git diff c065 -- <assigned paths>`). No tests, builds, checks, lint, formatting, scripts, runtime probes, network, or publication ran.

## Inspected artifact

- `src/uar/runtime/manager.rs`: per-retained-run `Arc<Mutex<()>>`, initialized in all three `RunStreamState` constructors (3496, 4537, 4652), and crate-private lookup helper near 8939.
- `src/uar/api/client_tools.rs`: fixed-stage diagnostics and acknowledgment guard spanning durable load through settlement CAS/reread.
- `Cargo.toml` and `Cargo.lock`: matching package version change from 1.2.2 to 1.2.3, no dependency changes in the assigned diff.
- `docs/releases/v1.2.3-kubo.md`, caller-tool continuation spec delta, and `handoffs/image-agui-singlewriter.md`.
- Existing producer join implementation, AG-UI caller, and scoped persistence dispatch were inspected for interaction with the change.

## Source conclusions

The helper clones the run-owned Arc under `active_runs.read()` and returns it; the map read guard does not reach the caller's mutex wait or subsequent persistence/producer awaits. Distinct live runs own distinct guards. All three constructors initialize the field.

Confirmation acquires the guard before its durable scoped lookup. A subsequent cooperating observer therefore loads after the preceding non-cancelled confirmation has released the guard and can reuse durable `source_settled` evidence. The owner/tenant/agent/run arguments remain on both lookup paths. Producer and graph joins, child-source joins, settlement CAS, validation, losing-CAS reread, and fail-closed error returns remain intact. No emitted terminal substitutes for cleanup.

When the live run is absent, an already-settled validated row is accepted before any join. A pending unsettled row reaches `await_settled_run_status`, whose initial missing-run guard refuses it with `source_run_not_found`; the patch does not invent cold acknowledgment.

The diagnostic closure logs only run identity and a field-free stage enum, then returns the same `anyhow::Error`. It does not add backend text or tool payloads to the log and does not replace the returned error.

## Uncomfortable limit and remaining evidence

Cancellation drops the local guard; a scoped persistence operation already dispatched may continue beyond that drop. Consequently this patch serializes cooperating active confirmation futures for one retained live source, not every eventual storage effect after cancellation, and supplies no cross-process or exactly-once guarantee. The writer handoff states that limit explicitly. Durable CAS and fail-closed reread remain necessary.

Compilation, lints, formatting, cancellation behavior under execution, and the new patch's real production-path result are unverified by this review. Prior gate018 does not certify this patch, as the release contract correctly states. Product verification remains local under the operator's exact approved commands; no general CI acceptance is assumed. No new arbitrary suites are requested.

Only this report was authored by the reviewer. No production guards or unrequested implementation changes were added. The added run guard traces to the observed competing AG-UI acknowledgment failure documented in the assigned handoff; the precise original backend failure stage remains unproven.
