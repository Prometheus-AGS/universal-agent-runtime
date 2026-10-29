# Bounded formatting repair r14

Stage: Execute, source implementation only. Assessment and analysis use the
retained exact-approved format failure; verification remains CLOSED. The user
explicitly requests a bounded owned-versus-baseline plan and implementation,
with r13 evidence preserved and no tests/builds before complete integration.
This plan is recorded before producer edits.

## Observed failure and boundary

The exact cargo fmt --all -- --check exited1 and retained4579534 bytes with4165
diff hunks across376 normalized paths (381 textual spellings/aliases). Exactly22
are owned producer files from r13;6 are unchanged controls and348 are outside the
frozen source inventory. No new formatter, compiler or baseline check is needed
to classify that retained result. Full path/identity partition is in
.agent-team/uar-core/handoffs/kubo-native-identity/format-partition-r14.json.

The boundary is file ownership: format all retained proposed changes within the
22 listed owned files, including their pre-existing style. This does not grant
control files, foreign modules, baseline files or the whole workspace. The354
baseline/control findings remain separately tracked and unresolved. Scoped repairs
cannot turn the mandatory whole-workspace format failure into a PASS.

## Complete scoped source batch

One implementation writer owns exactly these22 Rust files:

- `src/llm/liter_driver.rs`
- `src/llm/mod.rs`
- `src/llm/orchestrator.rs`
- `src/llm/tool_call_identity.rs`
- `src/server/tests.rs`
- `src/uar/api/routes.rs`
- `src/uar/api/sse.rs`
- `src/uar/persistence/providers/memory.rs`
- `src/uar/rag/ingestion_admission.rs`
- `src/uar/runtime/cost_budget.rs`
- `src/uar/runtime/manager.rs`
- `src/uar/runtime/presentation_history_tests.rs`
- `src/uar/runtime/skills/service.rs`
- `src/uar/runtime/thread/approvals/recovery.rs`
- `src/uar/runtime/thread/service_tests.rs`
- `src/uar/runtime/tool_admission/mod.rs`
- `tests/stream_terminal_contract.rs`
- `tests/support/approval_recovery_process.rs`
- `tests/support/fresh_process_resume_process.rs`
- `tests/tool_admission_integration.rs`
- `tests/tool_call_protocol.rs`
- `tools/seed-minimax-verification/main.rs`

The lead explicitly grants these cross-domain paths for formatting only. Others
share the checkout; preserve all other edits. No functions/assertions/literals,
contracts, defaults, visibility, dependencies, feature flags, schemas, warning
suppression, behavior fixes or test selection may change as adjacent cleanup.
Only the retained rustfmt presentation proposals may be adopted. Import or item
layout/sorting already proposed by that formatter requires explicit source review;
do not represent it as a runtime equivalence proof.

## Editing procedure; no formatter execution

Do not run cargo fmt, rustfmt in any mode, tests, Cargo/check/compiler, syntax or
import probes, lint, receipt validators, runtime/model/Docker probes, generators
or authored product/gate/check code while this batch is incomplete. Tier examples
in skills do not override the operator lock. No automatic workspace reformat.

Use the already-retained formatter output as an edit blueprint. An authorized
patch-authoring procedure may read that log and these exact source files, match
old-context hunks, and write only the22 allowed producer paths plus its handoff
metadata. This is editing, not a new formatter/check execution. Bind each current
preimage to its retained r13 bytes before replacement; preserve exact preimages
and original failed log under the handoff root. Construct all proposed candidates
before writes. Duplicate module/path renderings must yield the same candidate;
ambiguous, missing, overlapping or inconsistent context stops the authoring
procedure before any write, for source assessment. Do not guess/fuzz hunks or
silently skip a retained owned finding. Preserve UTF-8, existing LF line endings and file modes. Adopt an EOF final-LF
addition only when explicitly proposed by the retained formatter output; it is
formatting within the owned file, not an arbitrary line-ending conversion. No product,
compiler or formatter code is imported or executed by this authoring procedure.

Write a reusable exact owned-only patch and source handoff under the native
identity handoff root. Source inspection/plain diff and identity inventories are
allowed; no diff --check or new validation program. If the retained proposal
cannot be reconstructed safely, stop and report the concrete gap before choosing
another procedure. Do not replace it with an unapproved formatter probe.

## Baseline findings: preservation and ownership routing

Read-only control findings (6):

- `build.rs`
- `src/config.rs`
- `src/config_manager.rs`
- `src/main.rs`
- `tests/approval_recovery.rs`
- `tests/fresh_process_resume.rs`

The other348 exact paths are the baselineOutsideInventory list in the partition.
All354 paths stay unchanged by this repair. Track them as a baseline formatting
backlog requiring separate ownership/implementation approval, rather than expand
this repair or weaken the whole-workspace rule. Their classification is based on
retained output and the inventory, not a fresh independently executed baseline.
Never change vendor/submodule, pin/toolchain/config or formatter policy to hide
findings. Do not add ignores, skip annotations or suppression.

## Lead documentation, integration and complete boundary

The lead owns this plan and the existing OpenSpec tasks.md only, plus partition,
backlog/preimage/patch/source manifest, canonical KBD, board and review metadata.
Existing50 producer paths become51 by adding this plan. All27 controls and all
other existing producer identities remain as r13 except the22 formatting targets
and the existing tasks.md. Stage explicit paths only; never git add -A. No commit,
push, package install or publication is part of this repair.

Keep the entire r13 inventory/patch/review/approval/backend outcome, retained
customer binary and failed formatter log immutable. Prior10/24/83x249/34/build
results remain tied to r13, not re-executed or promoted to a new source identity.
Do not alter old baseline/r11/r12/r13 gate scripts to accept formatting churn.
Future execution needs its own exact approved command and source boundary.

After all22 edits and required docs are integrated, stop every source writer.
Inspect source integration, obtain separate artifact-only product and security
reviews under the usual bounded review policy (one initial and max one corrective
pass), and capture full51 producer/27 control source identities/preimages/tree,
backlog and reviewed binding. Reconcile the entire scoped canonical task ledger.
Review is source/manual only: no machine screening or verification commands.
No runtime test is required merely to mirror reversible formatting; the meaningful
future confirmation is an exact owned-file formatter check after complete source,
reviews/refreeze and fresh operator approval. This plan does not authorize it.

## Exit and uncomfortable limits

Source exit: all retained proposed owned formatting edits applied, no baseline
or semantic change, all docs/integration complete, writers stopped, independent
source reviews disposition recorded and renewed inventory/canonical state agree.
No passing format/runtime claim may be made from source editing or old receipts.
The broader workspace format gate remains blocked by354 preserved paths even if
these22 edits later pass their focused check. Baseline-wide repair, node hook
bootstrap, warning disposition, release/package-version decision, signed source
commit/immutable publication, actual consumer pin/runtime/refreeze, cumulative
acceptance and PR13 readiness remain separate holds. PR13 stays draft; no merge.


## Source-only authoring refinement: unterminated EOF

Before any producer write, the patch author correctly stopped because the owned
seed utility has no final LF, whereas diff display terminates every displayed
line. Retained final section tools/seed-minimax-verification/main.rs:245 includes
an added empty line token. The scoped repair explicitly includes the formatter's
final-LF normalization for this owned file. This refines the earlier ambiguous
newline-preservation phrase; do not preserve a known retained owned finding.

Reconstruct diff content as exact logical line tokens, distinguishing display
line delimiters from source EOF, and preserve the source's terminal empty token
where it exists. An explicit retained added terminal empty token authorizes the
single final LF represented by join on those tokens. Do not append an extra
blank line merely because the display itself is LF-terminated. Other line-ending
or literal/content changes remain prohibited. Retain original raw preimages and
fail before all writes for any remaining ambiguous/inconsistent proposal. No
formatter, syntax/compiler probe or test is needed to author this correction.
