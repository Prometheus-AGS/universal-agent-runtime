# Published image AG-UI cleanup failure diagnosis

Execute, diagnosis only. Verification CLOSED. This teammate used source/evidence
inspection and wrote this handoff; no production edits, tests, checks, builds,
imports, network calls, database engine/query probes or runtime commands ran.

## Artifact and observed event

Published immutable 1.2.2 digest
`048d9c8ef71002514b5419fa6f56b351bf874027e6a37cc28c0361a1a76ba2e1`
from source `c065` is reported by the lead as core 94/95, with clean shutdown.
The one failed scenario is `agui:reject:partial` setup; the partial result
rejection itself was never reached. Native gate017 and both platform builds
passed before this published-image run, which does not erase this failure.

Evidence root:
`target/client-tools-image-bundle-v1.2.2/target/client-tools-published-image-001/`.
Matrix event for wire run `9aa09a19-193b-44f5-adbf-737c84c2538f`, thread
`3e7d79ea-10af-471b-80d4-c2d2fa490c8b`, emits two caller tool calls in batch
`594edbdd-94dd-44ec-b789-f9dbbc15f630`, then RUN_ERROR
`host_cleanup_unconfirmed`. Exact caller IDs match candidate native run
`f936321b-4e89-4b15-aafc-3b035e6c1830` at 23:12:39.016 and warning
23:12:39.026763, category `caller_tool_settlement_failed`. The provider stream
completed at 23:12:39.010851. An initial nearest-warning correlation to native
`7fcf1355-d842-4776-9f6c-5720496e3286` was incorrect; exact batch/call-ID matching
supersedes that inference. The earlier warning is a distinct execution.

## What source establishes

`api/agui.rs::reconcile` sees committed ClientToolsRequested and calls
`seal_frontend_handoff`. That function retrieves the authenticated native run,
then calls `api/client_tools.rs::confirm_handoff_for_run`. Any returned error
is collapsed to only two labels: exact string `run_settlement_unconfirmed`, or
generic `caller_tool_settlement_failed`. The observed generic label does not
retain the underlying error or stage. It then deliberately seals the observed
fail-closed terminal, rather than reporting successful cleanup.

`confirm_handoff_for_run` can fail while loading/validating the exact durable
batch; requiring Pending state; joining the root run; requiring actual Done;
joining an optional executing child; writing the immutable settlement CAS; or
reloading/validating the CAS winner and requiring `source_settled`. Therefore
the public code is not proof of a physical producer cleanup failure specifically.

`manager::await_settled_run_status` actually joins the retained producer and
graph root before accepting a terminal run status. No evidence here warrants
weakening that requirement. The Surreal continuation CAS can return a real
storage error or false; false triggers one exact reread and only an already
settled row is accepted. There is no demonstrated reason to invent a retry,
disable confirmation, or accept a pending row.

## Uncomfortable evidence limit and minimal proposal

The exact underlying cause remains **unknown**. Candidate logging discarded it.
The retained embedded database contains binary VLOG/SST files, not a textual
backend error log establishing what the failing operation returned. This
diagnosis did not open its engine or execute a query. Its final row could not
alone establish the transient error returned at 23:12:37 either.

The minimal source proposal is diagnostic retention, not a speculative semantic
repair: attach typed, field-free stage context at the existing confirmation
operations, and have the AG-UI warning log that static stage plus safe typed
storage kind/subkind when present. Retain the underlying error chain, exact
authentication/correlation, actual joins, CAS ownership, and fail-closed terminal.
Do not format database messages, SQL, tool content, histories, credentials, or
arbitrary backend detail fields. An internal diagnostic reference can correlate
the warning with the native run already retained.

This would require scoped ownership of `api/client_tools.rs` and
`api/agui.rs`; no manager semantic change is currently justified. It requires a
new 1.2.3 source/artifact/digest if built. Do not overwrite or retag immutable
published 1.2.2. Lead must reconcile the new inventory and obtain the applicable
verification instruction before running any gate. Until richer evidence exists,
neither a source root-cause repair nor published-image acceptance is established.

## Additional read-only narrowing

Exact batch-ID inspection of retained VLOG bytes finds a serialized version of
this batch with `source_settled:true`. Raw VLOG data does not by itself prove
that version is the current committed row or that a caller observed successful
acknowledgment; it may retain historical/uncommitted values. No database engine
was opened to interpret that state, and this finding is not acceptance evidence.

The proposed unordered-map/date-format explanation is unsupported by the
snapshot's source shape. `ClientToolBudgetSnapshot` contains fixed structs,
vectors and scalar/optional values; its timestamps are integers, and
`RunUsageSnapshot`/`ThreadBudgets` contain no unordered map. The row's graph
state is null for this ordinary case. Monetary fields are f64, and the backend
CAS compares serialized JSON strings; an exact numeric roundtrip mismatch is
not established by these facts alone. No precision or serialization fix is
proposed without stronger evidence.

AG-UI's retained observer and stream both invoke reconcile, so concurrent
confirmation is a real source path. The common confirmation method already
accepts a CAS winner only after rereading the exact row and validating
`source_settled`. No observed error establishes whether the failure occurred
before that CAS, inside the backend, or in the reread. Source-independent critic
analysis is pending. The diagnostic proposal above is **not implemented or
approved**; no production source changed during this diagnosis.

## Authorized diagnostic retention implementation

After the diagnosis above, lead authorized a diagnostic-only source change.
`api/client_tools.rs` now attaches a field-free typed context at each existing
confirmation failure boundary: LoadBatch, ValidateBatch, PendingState, JoinRoot,
RootStatus, JoinChild, ChildStatus, SettlementCas, ReloadBatch,
ValidateReloadedBatch, and SettlementWinner. Context retains the original
error chain. The same joins, status requirements, validation, CAS, winner reread,
and acceptance conditions execute in their original order. No retry was added.

`api/agui.rs` logs only the optional typed stage in the existing native-run
warning. The stage enum has no data fields, so Debug emits only fixed enum
variant labels. Existing coarse `run_settlement_unconfirmed` classification now
checks the retained chain to preserve its behavior after adding stage context.
It does not log any arbitrary error text. No shared field-free storage helper
is available here; the budget module's storage-detail helper is private and
was not exported or duplicated for this bounded change.

Exact production ownership: `src/uar/api/client_tools.rs` and
`src/uar/api/agui.rs` only. This handoff is the only documentation edit. The
observed missing provenance is the sole reason for the new diagnostic type.
No semantic fix, retry, backend relaxation, cleanup bypass or scope expansion
was added. No execution or verification ran; compilation and actual diagnostic
output remain unverified pending lead's authorized gate. Published 1.2.2 stays
immutable; this source is subsequent work.

## Display-preserving diagnostic correction

Source review identified that anyhow stage contexts changed the existing
`error.to_string()` observed by OpenAI/ACP/A2A callers. The context implementation
above is superseded before execution: `confirm_handoff_for_run` now has a small
local map_err closure which logs only the field-free stage and existing run ID,
then returns the **original anyhow error unchanged**. All eleven stage labels
remain available at their original failure sites. The enum is private and only
derives Debug; it no longer wraps errors. Original Display, downcast behavior,
error chain, acceptance rules and error text are preserved.

`api/agui.rs` is restored to its original coarse category and warning fields;
the correlated stage warning now originates from shared confirmation itself.
No raw backend error/message or tool data is logged. No execution or
verification ran after this correction. Exact source touched remains
`api/client_tools.rs` plus restoration of the prior diagnostic edit in
`api/agui.rs`; the final added diagnostic behavior is in client_tools.rs only.

## Additional retained correlation and dependency source

The earlier native7fcf warning maps by exact call ID to the passed AGUI mixed-host/caller case, whose other observer published successful RUN_FINISHED. It is not an expected negative. This establishes competing confirmation paths can disagree in one source run; it does not yet establish which operation fails. Build016 passes with fixed-stage diagnostics; gate018 is running under the operator finite repair authority.

Potential CAS visibility mechanism, still an inference pending stage output: pinned SurrealKV0.21.4 commit source publishes conflict-oracle entries before memtable apply outside its write mutex. See https://github.com/surrealdb/surrealkv/blob/v0.21.4/src/commit.rs and the source reference to PR378 (https://github.com/surrealdb/surrealkv/pull/378). A conflict is not itself proof that the winner is already read-visible. This is not a claim of an upstream defect. Required Firecrawl developer-index lookup was attempted; installed CLI rejects the developer subcommand. Pinned primary source was read directly instead. No dependency changes are proposed.
