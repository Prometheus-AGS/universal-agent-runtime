# A2A mixed caller projection source review

Task 13.20, Execute. Reviewed frozen `project_settled_client_tools` in `src/uar/api/a2a/thread_service.rs`, the latest mixed-projection handoff, durable batch validation, and actual orchestrator transcript construction. This report is the only reviewer edit.

## Disposition

No blocking source finding in the scoped correction.

The orchestrator appends executed host-tool results to canonical history, while its separate visible history ends with an assistant message containing `caller_calls` only. Reading the latter fixes the demonstrated last-canonical-message assumption without changing private execution history.

The projection requires an array ending in an assistant message with a tool-call array. Every projected call must have a string ID. Set equality against durable `pending_call_ids` rejects missing, extra or foreign IDs, and the separate length comparison rejects duplicate visible calls hidden by set construction. Durable batch validation independently requires unique nonempty pending IDs. Public metadata uses these visible calls, visible history and captured caller schemas; it does not read canonical host calls/results or host descriptors. Canonical history remains untouched for continuation.

Malformed public evidence returns a static `TaskError::Host`, rather than `TaskError::Conflict`. Thus the observer's immediate CAS-conflict retry branch cannot repeatedly retry this deterministic projection error. Error strings do not contain raw calls, results, database details or schema payloads. Existing real producer settlement and durable handoff acknowledgment still precede InputRequired publication; no successful handoff is fabricated.

## Evidence and limits

Only source reads (`sed`, `rg`, `tail`) and this report write ran. No executable checks, tests, builds, imports, formatting, runtime or network commands ran. No production file was edited. The reviewer gate remains CLOSED and the lead owns its scoped local functional loop. Compilation, mixed-call HTTP projection, race/disconnect scenarios and subsequent teardown remain unverified by this reviewer. Separate negative-stream fixture interpretation changes are outside this report. Same-family isolated source review is not cross-model adversarial evidence.
