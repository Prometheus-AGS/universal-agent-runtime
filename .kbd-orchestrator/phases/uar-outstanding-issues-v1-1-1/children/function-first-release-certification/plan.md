# Plan — function-first release certification

Date: 2026-10-17. Stage: **Plan**. Parent: `uar-outstanding-issues-v1-1-1`.
Child phase ID: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Existing OpenSpec change: `uar-outstanding-issues-v1-1-1`.

Deliver a smaller release program that observes public prompt propagation,
explicit allowed-skill activation, truthful streaming/tool outcomes, owner-qualified
durability and the advertised authority restrictions. Implement the complete
revised program before requesting its exact execution commands. This document
does not mark any implementation or verification task complete.

## Authority, inputs and stop boundary

The inspected waypoint at source revision 3067 names this child, with Spec
COMPLETE and no registered child changes. The lead enters Plan through the
actual KBD CLI before accepting this plan; authoring this document does not
itself change canonical stage. Use the actual CLI for registration and
transitions, never edit generated waypoint, progress or task projections.

Inputs are child `goals.md`, `assessment.md`, `analysis.md`, `handoff-in.md`,
`review/assess-findings.md`, `review/analyze-findings.md`; parent `plan.md`,
`tasks.md`, `ownership.md` and the existing OpenSpec verification contract.
Source inspection covered the five release adapters, runner, support reader,
provider fixture, manifest schema/example and affected documentation. It is
source evidence only. The second Analyze review resolved the disabled-attachment
contradiction; its eligible-set and unique-dependency conditions are binding.

**Operator verification lock: CLOSED.** No tests, lint, format checks,
typechecks, Cargo checks, compiler/import probes, builds, packaging checks,
acceptance scripts, external gates, or `git diff --check`. Planning artifact
review is source-only and does not authorize completed-implementation review or
product verification. Before every command or delegation classify the operation
as source inspection, authorized implementation/documentation/memory writing, or
verification. Every execution packet repeats this lock. At each 120-minute
checkpoint and context handoff reconcile waypoint, complete ledger, active
writers and intervening commands; retain violations without acceptance credit.

Whole-phase completion and a new exact inventory are prerequisites, not gate
permission. Present them and exact proposed commands to the operator, then wait
for the instruction naming those commands. Any later implementation edit closes
the gate and requires renewed inventory and instruction. No standalone identity,
Docker, readiness, import or other preflight is implied by gate approval.

## Adopted scope and interface effects

Run all six reduced groups on the local candidate. Run exactly `streams` and
`recovery`, in that order, on each immutable published architecture child.
Keep the four existing native source supplements in the local command. Preserve
their distinction from installed-image evidence. Do not replace the pinned
libraries, alter vendored code, change runtime product behavior to fit fixtures,
or introduce a second certification runner.

The published reduction explicitly withdraws per-child re-observation of budget
refusal, compiler-profile restriction, credential migration and MCP failure
handling. Those remain local release blockers. Both published children still
exercise prompt/skill output, stream/tool completion and durable owner-qualified
restart, including the retained recovery controls. Native versus emulated
execution and companion architectures remain disclosed. This amendment must
appear in the OpenSpec proposal/design, verification contract and release docs
before implementation adopts the smaller inventory.

No Rust public API, provider contract, database schema, UI boundary or dependency
pin changes are selected. Interfaces changed are certification scenario IDs,
the opt-in streaming observation API, controlled-provider trace fields,
stage-specific manifest group selection and release claims. Runtime defects
discovered later need a separately recorded exact-file assignment and join the
same full-phase boundary; they are not permission for speculative source repair.

## Ordered task ledger

The following are exact **proposed registration IDs**, not claims that KBD has
already registered them. Identity is the full child phase ID plus existing
change ID plus the backend ID. Preserve parent task identities and completed
history; append child work and amend current acceptance wording deliberately.

| Order / stable | Backend ID | Owner | Dependencies | Required deliverable |
| --- | --- | --- | --- | --- |
| FC1 | `1.1` | uar-lead | Accepted Spec and Plan | Expand advisory child scope to the exact paths below, register ledger/ownership, reconcile OpenSpec and parent contracts, and record the assurance amendment. |
| FC2 | `2.1` | uar-runtime | FC1 | Runtime adapter, provider fixture and support timing reader; prompt/skill oracle, reduced stream/MCP/binding/combined observations and source handoff. |
| FC3 | `2.2` | uar-trust-tools | FC1; messages-facade fold waits for FC2's observation handoff | Credential, compiler and recovery adapters with all folded assertions retained; cross-domain recovery ownership is explicitly assigned to this one writer. |
| FC4 | `3.1` | uar-lead | FC2, FC3 | Runner/schema/example integration, closed prebuilt-image mode, scenario disposition, exact supervisor preparation runbook and required docs; consume writer handoffs. |
| FC5 | `3.2` | uar-lead | FC1–FC4 and every parent implementation item | New whole-phase implementation inventory, reconciled canonical state, concrete local selection/configuration, published manifest templates and exact preparation/operator packet. Gate remains CLOSED. |
| FC6 | `4.1` | uar-lead | FC5 and separate exact operator preparation instructions | Build/inspect frozen local image; stage fresh same-image qualified predecessor/replacement and physical boot; seal image/proof bindings and executable local manifest. Preparation is not acceptance. |
| FC7 | `4.2` | uar-verifier | FC5, FC6 and explicit operator authorization for completed-artifact review | Fresh-context artifact-only review of frozen artifacts and preparation receipts; findings, model/isolation limits and dispositions. No build writer is delegated. |
| FC8 | `4.3` | uar-lead | FC5–FC7 dispositions and exact operator local-gate instruction | Execute the local functional command against the unchanged prebuilt image, retain actual receipts, obtain operator timing disposition, and reconcile parent `6.2` / V2. |
| FC9 | `5.1` | uar-lead | FC1–FC8 or explicit blocked handoff | Child reflection and `handoff-out.md` with implementation, evidence and publication states separately recorded; parent P1–P3 remain accountable. |

FC1–FC5 are implementation/documentation work. FC6–FC9 are preparation, review, evidence and
handoff work and must not inflate implementation-complete counts. If the CLI
cannot express that distinction, preserve a separate explicit inventory without
falsely completing those tasks. A blocked handoff does not complete FC8 or claim
child acceptance. Local and published gates remain linked to parent V2/P2;
do not create a second independent acceptance authority.

## Exact single-writer ownership

The lead's task-specific assignment supersedes earlier ownership only for the
paths below after old writers have handed them back. Others share this checkout;
preserve their edits. Default execution is lead plus one specialist plus the
reserved independent-review role. Runtime and trust may overlap only when both
packets remain disjoint and total active agents including lead is at most four.
One specialist writes each file throughout its task. Review owns no production
file. The lead alone owns the build target and later gate processes.

| Writer | Exact paths |
| --- | --- |
| uar-runtime / FC2 | `scripts/release/uar_1_1_1_runtime.py`; `scripts/release/uar_1_1_1_support.py`; `tests/fixtures/uar_1_1_1_provider.py` |
| uar-trust-tools / FC3 | `scripts/release/uar_1_1_1_credentials.py`; `scripts/release/uar_1_1_1_compiler.py`; `scripts/release/uar_1_1_1_recovery.py` |
| uar-lead / FC4 | `scripts/release/uar_1_1_1_acceptance.py`; `scripts/release/uar_1_1_1_manifest.schema.json`; `scripts/release/uar_1_1_1_manifest.example.json` |
| uar-lead / FC1 and FC4 | `openspec/changes/uar-outstanding-issues-v1-1-1/proposal.md`; `design.md`, `tasks.md`, `verification.md` in that same change; its `specs/model-path-resiliency/spec.md` and `specs/knowledge-rag-product-certification/spec.md` for the functional and release-evidence deltas |
| uar-lead / FC1 and FC4 | Parent phase `plan.md`, `ownership.md`, `spec/runtime-contracts.md`, `spec/trust-contracts.md`, `spec/state-contracts.md`; `docs/operations/uar-1.1.1-acceptance.md`; `docs/releases/v1.1.1.md` |
| uar-lead / FC4 | `.prometheus/decisions.md` append-only decision memory for the adopted function-first assurance tradeoff. |
| uar-lead / FC1, FC4–FC9 | Child `scope.json`, `plan.md`, NEW `scenario-disposition.md`, NEW `supervisor-preparation.md`, NEW `implementation-inventory.json`, NEW `operator-gate-decision.md`, NEW `reflection.md`, NEW `handoff-out.md`, NEW `handoffs/FC1.md`, `handoffs/FC4.md`, `handoffs/FC5.md`, `handoffs/FC6.md`, `handoffs/FC8.md`, `handoffs/FC9.md`; NEW parent `implementation-inventory-function-first.json`; NEW parent `evidence/gates/function-first-local.json`, `function-first-arm64.json`, `function-first-amd64.json` |
| uar-runtime / FC2 | NEW child `handoffs/FC2.md` |
| uar-trust-tools / FC3 | NEW child `handoffs/FC3.md` |
| uar-verifier / FC7 | NEW `.agent-team/uar-core/reviews/function-first-release-certification/implementation-findings.md` and, only if needed within the review cap, `implementation-corrective-findings.md` in the same directory; this is the canonical `uar-project-review` findings location. Child records link it. |

Relative child and parent entries above resolve against their full phase
directories. Canonical task/status outputs are written only by the real KBD CLI
under lead coordination; the generated `tasks.md` is never hand-edited. Source
identity/receipt files generated during approved gates belong to the lead's
runner-owned evidence directory, not a concurrent teammate. No composition root
(`src/server.rs`, `src/lib.rs`, `Cargo.toml`) is selected for editing. If needed,
the lead assigns exactly one specialist before a task starts; `versions.toml`
stays read-only. No workflow or workflow-invoked build/package script edit is
selected. Such an edit would encounter the mandatory policy-validator rule and
must stop for reconciliation with the closed operator gate.

FC1 widens the child's present artifact-only scope to these concrete files before
any outside-child implementation. The scope file is coordination, not a native
permission boundary. No foreign checkout, customer deployment, registry or
external issue is writable by this child assignment.

## FC2 implementation sequence and contract

1. Add an opt-in incremental observation path to `Http.stream` or a dedicated
   method beside it, without changing existing string-returning callers. Consume
   bytes as they arrive, preserve strict decoding, finite whole-stream deadline,
   partial evidence and cleanup. Record first **parsed content delta**, not first
   byte or heartbeat. Agree the concrete signature in FC2's handoff before FC4
   consumes it. Do not run an import probe.
2. Extend the existing controlled provider mode and trace with a request-bound
   correlation ID, unique user marker, unique enabled-skill overlay marker,
   deterministic answer derived from both, and peer receive/first-write/last-write
   timestamps. Preserve redaction; synthetic fixture data only. The response
   oracle must fail if either marker is absent rather than blindly echo a fixed
   sentinel. Do not expose credentials in traces.
3. Through existing `POST /api/uar/skills/` and `POST /api/uar/runs`, register an
   enabled skill and submit its ID as an explicit `skill_attachments` attachment.
   Use the existing source-declared request shapes. Assert the actual admitted
   skill overlay reaches the provider, fully assembled public SSE and AG-UI
   answer text each equals the expected derived answer exactly, no activation
   failure occurs, terminal status succeeds and persisted answer text equals the
   same expected answer with matching status. Reject extra, missing, reordered
   or duplicated answer text; marker containment alone is insufficient.
   A second distinct prompt must change the
   derived answer so the receipt proves dependence rather than fixture identity.
4. Register a disabled skill that remains in the effective eligible set and has
   unique tool dependencies. Its attached run must report the exact `disabled`
   activation failure, exclude its overlay/dependencies and preserve policy.
   Ordinary provider contact is allowed: runtime continues the ordinary prompt.
   This is not a zero-contact denial oracle and does not widen tool permissions.
5. Remove the release-only exhaustive partition/fixed-field loops. Retain the
   semantic outcomes and positive controls in the disposition table below.
   Preserve ordinary regression source cases; do not run them or claim their
   continued coverage has been verified. Fold runtime labels only after moving
   their assertions, including distinct-provider `/v1/messages` coverage to the
   trust writer via a written observation contract, not a shared-file edit.

Timing receipt fields must include client-before-POST, admission response, first
parsed content delta and terminal-status observation, plus controlled peer
receive/first/last-write. Compare durations within each clock domain; do not
subtract unrelated monotonic epochs. Report the residual and client/network/
polling uncertainty, never pure runtime overhead. Timing is diagnostic, with no
invented numeric SLO. Before V2 acceptance, present the observed client/peer
durations, uncertainty and any identified cause to the operator for an explicit
release disposition: accept the measured result or require investigation/fix.
The author cannot declare material slowness acceptable; a missing disposition
leaves timing/release acceptance pending even when functional assertions pass.
A numeric threshold requested by the operator becomes an explicit requirement.
Controlled responses prove UAR
propagation and explicit attachment activation, not external-model reasoning,
model-selected skill quality or real-provider latency.

## Complete scenario disposition and consequential operation cuts

FC4's `scenario-disposition.md` expands the following exact prefix sets to every
old ID and names its destination assertions, actual production route, negative
control, backend and evidence fields. No ID may disappear as a silent skip.

| Existing ID or exact set | Destination / retained observation |
| --- | --- |
| `binding-qualified-durable-control` | Fold into `binding-16000-catalog-reservation-refusal`: adequate envelope succeeds with provider contact, accounting settlement and successful post-restart run; actual 16k catalog refuses. |
| `binding-v1-messages-policy` | Fold into `messages-facade-policy` in credentials; keep distinct provider paths, allowed native stream, exact denial and zero denied contact. FC3 consumes FC2 handoff. |
| `binding-unattested-refusal`, `binding-16000-catalog-reservation-refusal`, `binding-shared-alias-and-credential-free`, `binding-fallback-policy`, `binding-timeout-inheritance`, `binding-team-source-policy`, `binding-summary-capture-policy`, `binding-invalid-budget-refusal` | Retain all eight IDs and their authority/effect assertions. |
| `streams-native-partition-matrix`, `streams-sse-fields-fixed-partitions` | Replace with NEW `streams-prompt-skill-output-timing`. Exhaustive split/newline/BOM/field combinations become ordinary regression coverage rather than mandatory release observations. |
| `streams-eof-matrix` | Retain text-without-terminal and pending-tool EOF; require failure and zero pending-tool effect. Remove duplicate empty/metadata/framing permutations from release execution only. |
| `streams-invalid-matrix` | Retain one known malformed-data failure, overflow, invalid UTF-8, post-terminal data and a valid unknown-event control. Do not collapse these semantics into one failure. |
| `streams-tool-terminal-control`, `streams-cancel-cleanup`, `streams-graph-inherited-control` | Retain successful tool loop, cancellation/next-run cleanup and approved host graph delegation. |
| `mcp-application-error`, `mcp-explicit-success-and-omitted-status`, `mcp-protocol-and-timeout`, `mcp-uncertain-effect-preserved` | Retain all four IDs and six underlying outcomes. Keep cold receipt immutability/later success on application failure and uncertainty; remove repeat restart/later-success work for the already-successful and omitted-status controls only. Native incidental `isError` remains a source supplement. |
| `combined-valid-control` | Fold into `combined-failed-tool-incomplete-stream`: later success is a distinct run/receipt, original failed receipt unchanged, exactly two effects. |
| `identity-isolation`, `local-installation-stability`, `shared-source-grants`, `absent-service-policy`, `store-decrypt-failures`, `single-owner-compatibility`, `local-quarantine`, `messages-facade-policy`, `response-correction-policy` | Retain all nine common credential IDs, with expanded messages-facade assertions from FC2. |
| For each `local`, `surreal`, `postgres`: `-preview-authority-pagination` and `-secret-redaction` | Fold into that backend's `-legacy-conditional-copy`; preserve ordinary-user denial, complete keyset counts/pagination, stale cursor, and preview/receipt/raw-log secret exclusions. Common malformed-query forms may run once on local; backend decisions remain separately observed. |
| For each backend: `-preview-status-inventory` | Fold assertions into matching drift/conflict/rotation/historical cases, retaining exact HTTP and journal statuses for missing, quarantined, source-changed, already-applied and conflicted records. |
| For each backend: `-metadata-upgrade`, `-legacy-conditional-copy`, `-fixed-local-conditional-copy`, `-revision-drift`, `-mapping-drift`, `-destination-conflict`, `-concurrent-replay`, `-lost-response-retry`, `-immutable-rotation-deletion-restart`, `-historical-receipt` | Retain all thirty IDs; actual local/Surreal/PostgreSQL transaction and journal implementations stay mandatory. |
| `legacy_principal_namespace_cold_readback` | Fold into `attested_upload_restart_owner_isolation`; both supported backend cold boots retain legacy inventories and deny cross-owner retrieve/retry/adopt. |
| `unattested_remote_upload_refusal`, `attested_upload_restart_owner_isolation`, `joined_quiesce_reclaim_controls`, `qualified_supervisor_reclaim_controls` | Retain four recovery IDs, real durability and qualified supervisor physical exclusion/proof/replay controls, unknown liabilities and owner isolation. Missing genuine qualification is blocked, never waived. |
| `compiler-restart-preserves-existing-resources` | Fold full spec/report/session/registered-artifact unchanged readback and restored local operations into `compiler-local-single-owner-control`. |
| `compiler-local-single-owner-control`, `compiler-remote-two-principal-denial`, `compiler-local-shared-denial` | Retain three IDs. Initial denial coverage keeps both admitted remote principals, both REST aliases and every registered operation in restricted profiles. Keep representative spoofed headers and malformed bodies with exact profile-denial code. After restart run representative read/write/session denials plus full state comparison instead of repeating the entire cross-product. |

The expected local inventory becomes 65 labels: runtime 19 (binding 8, streams
6, MCP 4, combined 1), credentials 39, recovery 4 and compiler 3. This is the
original 80 minus 16 old labels plus one new label. The runner's exact equality
requirement remains; label totals alone never prove assertion coverage.

Actual work removed: the six newline/BOM partition loops totaling
`Σ(len(wire)+1)` installed runs, the 12 fixed-field runs, duplicate EOF/framing
permutations, successful/omitted MCP restart-plus-later-run repetitions, duplicate
compiler post-restart cross-products, and eight published group executions.
Folding combined/migration labels alone removes no invocation where assertions
already share a journey. Local six plus two per child is ten group executions
instead of eighteen. These are source-derived counts; no wall-time saving has
been measured. Keep distinct backend startup/transaction and recovery operations.

## FC4 runner and documentation integration

The runner still accepts only a closed program: local full six groups;
published exact `streams,recovery`; or one previously failed/blocked/interrupted
group from that stage's declared program with a valid immutable parent receipt.
An empty `groups` receipt from pre-build failure cannot become a failed-group
repair parent. Preserve identity comparison, explicit source-change disclosure,
secret-safe receipts, owned cleanup, immutable published image/source/platform
binding, publication provenance and the ban on replacement binaries.

Update manifest schema/example and `read_manifest` together. Do not make arbitrary
subsets valid, treat omitted groups as passed, or credit native tests to images.
Receipts state their selected program and the omitted architecture-specific
assurance. Keep local four native cases exactly as declared in `NATIVE_CASES`:
binding refusal history, native incidental error, SDK supplied credentials and
in-memory HTTP atomic migration. Native source evidence is supplemental.

FC4 implements the NEW closed manifest option `image_preparation: "prebuilt"`
and required `prepared_image_id` plus `image_preparation_receipt: {path,digest}`.
These are proposed fields, not existing interfaces. Function-first local and
published manifests must select this mode. Keep legacy build behavior only for
existing legacy manifests; never fall back to a build/pull when prebuilt input
is absent or mismatched. In prebuilt mode the functional runner inspects the
already present image and requires the same ID, source/fixture identity and
platform bound by the hashed preparation receipt. It does not rebuild or pull
UAR. The supervisor identity receipt must name that exact image ID too.
Published mode retains existing registry/index/tag/workflow provenance checks,
and compares the locally prepared child digest without replacing its bytes.
Record selected image mode and preparation receipt in the final receipt.

The image receipt establishes byte identity, not supervisor qualification. FC4
also authors `supervisor-preparation.md` from the existing parent
`coordination/handoffs/native-team-staging-20261016.md`,
`coordination/handoffs/rc05-staging-helper.md` and retained-controller handoff.
It names actual disposable guest/controller controls, exact proposed commands,
private inputs and output receipts before FC5. Existing private helper paths are
historical references only until their current contents/interface are confirmed;
this child does not edit outside the repository. If a usable preparation
procedure cannot be authored from available evidence, FC4 remains incomplete.
No new preparation script or external write scope is implied by this plan.

Reconcile the existing inventory `boundary: open` contract with the operator
lock: implementation readiness and operator authorization must be separate
named facts in documents/receipt data. An inventory cannot grant execution.
Any necessary manifest/schema addition must be specified before it is consumed;
do not invent CLI flags. All source/image/readiness checks remain internal to
the exact approved functional invocation, not standalone preflights.

Amend current acceptance wording in the existing OpenSpec change and parent plan,
preserving historical task completions and receipts. Add a narrow functional
prompt/attachment/output scenario to `model-path-resiliency` and amend release
evidence scope in `knowledge-rag-product-certification`; preserve production
parser/authority/durability requirements. Update all three phase contract
fragments so local-only credential/budget/compiler assurance is explicit.
Document the controlled peer, diagnostic timing and untested model selection.
No OpenSpec validation command runs while the gate is closed.

## Whole-phase inventory and operator decision

FC5 reads the current waypoint and complete parent and child ledgers, every
writer handoff and all amendments. The old L11 inventory is historical after
any source/fixture edit. The new parent inventory names every original and
subsequent implementation task, FC1–FC4 path-by-path, fixture/configuration,
existing migrations, operational settings and required documentation. Record
each item implemented, exact source/fixture identity, unresolved items and the
matching handoff. Unknown or conflicting status keeps implementation incomplete.

Record exact identity from the frozen saved source and existing metadata as
inventory documentation; never import `source_identity()` or invoke an identity
preflight while CLOSED. If the available evidence cannot bind the actual bytes,
record the missing identity and ask the operator for the exact needed command;
do not copy an old digest onto new source. Concrete manifests must have actual
identities and private owned paths; placeholders remain non-executable templates.

FC5 concretizes only the local candidate label, platform, source/fixture identity,
configuration and private resource paths. Fresh `prepared_image_id` and supervisor
receipt values cannot exist before the authorized FC6 preparation; explicitly
mark the local manifest non-executable until FC6 fills them from actual receipts.
The arm64/amd64 files remain clearly non-executable templates pending parent P1
publication and each child's fresh preparation. Never invent a future digest.

## FC6 authorized preparation: image, qualification and fresh proof

This sequence begins only after FC5's whole-phase implementation inventory. The
lead is its sole image/build/staging writer. Each numbered action requires the
operator's explicit instruction naming its exact commands and target; recording
the sequence or approving acceptance does not authorize preparation. Retain
separate sanitized preparation receipts and source/image identities. Commands
below are proposed and unrun from the UAR checkout.

1. Request the local candidate build and image inspection as their own named
   commands, using the frozen FC5 source and unique candidate label:

   ```text
   docker buildx build --load --platform linux/arm64 --tag uar:1.1.1-function-first --file Dockerfile .
   docker image inspect uar:1.1.1-function-first
   ```

   Record the actual image ID/platform, source/fixture identities, build result
   and logs in the preparation receipt. Both commands need explicit approval;
   approval of the build does not imply image inspection or Docker readiness.
   The selected local platform is linux/arm64; any change requires the rendered
   packet and command to be revised before authorization.
2. Separately request the exact controller/guest preparation commands authored
   in `supervisor-preparation.md`. Use a genuinely operator-qualified controller:
   retained `production_qualified: true`, controller source digest, mechanism,
   controller ID and qualification ID must match the actual fresh registration.
   Historical controller qualification may remain valid; a new runtime identity
   file cannot create or promote qualification. Missing/mismatched qualification
   blocks staging; obtain separately authorized real qualification before use.
3. Install/start the exact prebuilt candidate bytes in the existing disposable
   qualified environment, using a fresh scope and fresh predecessor registration
   from the real controller journal. Stage at least one queued and one active
   predecessor attempt through the real team APIs. Observe the provider trace
   for the active attempt under the specifically approved preparation command.
   Do not synthesize catalog rows, registration rows, fences or attempted work.
4. Obtain separate explicit authorization for the exact disposable guest physical
   boot transition and replacement startup commands. Reboot only the named guest,
   preserve its real durable state/journal, and start a fresh replacement using
   the same inspected image bytes. Capture the changed boot ID, exact predecessor
   registration/fence, distinct replacement fence, scope/attempt IDs and endpoint.
   A process restart alone does not prove native whole-boot exclusion.
5. Seal a new private supervisor identity receipt containing exact `image`,
   `platform`, `source_tree`, `fixture_tree`, `runtime_image_id`, isolated phase
   identity, runtime endpoint, predecessor registration/fence, replacement fence
   and queued/uncertain attempt IDs. Bind its hash and qualification record/hash
   into the private infrastructure file. Record journal access and credentials
   by private references only. Populate the image receipt fields and executable
   local manifest from these observed values; no acceptance has occurred yet.

The recovery gate itself requests the fresh generation-1 challenge, records real
physical exclusion and consumes its proof to reclaim. Each local/published run
and every affected recovery rerun needs one fresh physical boot, predecessor,
replacement and unconsumed challenge/proof sequence. A prior successful reclaim
or partially consumed attempt cannot be replayed as preparation. Never reuse the
old rc04/rc05 identity by merely replacing hashes. If a gate stops before proof
use, preserve that fact but do not assume safe reuse without an explicitly
authorized fresh-state preparation decision.

After P1, perform the same preparation separately for each published child:
request an exact `docker pull --platform linux/arm64 <actual immutable child>`
and `docker image inspect <actual immutable child>`, then separately stage its
same-image qualified environment and fresh physical boot; repeat with linux/amd64
and that actual child. These are non-executable command shapes until the lead
substitutes P1's observed digests in the operator packet. Do not run placeholders.
Each architecture's P2 command follows its own preparation and instruction;
local preparation or the other architecture's proof cannot qualify it. Published
manifest sealing occurs only after those receipts exist.

No source/fixture/configuration implementation edit is permitted between freeze,
preparation and acceptance. Such an edit returns to implementation, invalidates
the relevant image/source binding, and requires renewed FC5 inventory, explicit
preparation authorization, fresh same-image staging and gate authorization.

## Exact operator acceptance decision

The operator packet includes the complete inventory, source state, all remaining
tasks, actual model/reviewer limits, risk of reduced per-child assurance and
the following **proposed, unrun** commands. Working directory is the current UAR
checkout `/Volumes/T7/Projects/prometheus/universal-agent-runtime`:

```text
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-local.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-arm64.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-amd64.json
```

Request the local command only after FC6 and FC7 dispositions. Published commands are later parent P2 decisions
after authorized P1 supplies observed immutable digests and provenance. The local
command uses the FC6 prebuilt image and existing four serial native supplements;
approval must state those internal actions clearly. It does not build or pull
UAR and cannot invalidate the prepared supervisor image by replacement. This
child contains no The Boss deliverable, so no Boss customer build is selected.
UAR's customer artifact is the image produced by authorized preparation and used
unchanged by the functional command.

FC7 additionally requires an explicit operator decision authorizing the fresh
completed-artifact review dispatch under the lock. Supply only frozen artifacts,
requirements, ownership and actual receipts/absence of receipts; no producer
conversation or preferred verdict. Use `uar-project-review`, report critical/
warning/suggestion findings, judge identity, isolation and sycophancy-screening
availability. One review plus at most one corrective review; missing independent
judge means pending review. Existing exhausted parent review rounds are not
silently renewed: this new child review scope must be explicit in the decision.
An implementation fix after review requires a renewed inventory and operator
instruction before either corrective review or product command.

FC8 retains exact argv, start/end, exit code, prepared/used image identity, selected
scenario inventory, timings, source/fixture/config hashes, transcripts and
sanitized receipt paths. Failed, blocked, interrupted and unrun are distinct;
exit zero without required observations is not acceptance. Fix only observed
failures, renew the inventory/instruction, and rerun only the named failed gate.
Preserve passing and failing historical receipts. No broad checks or CI runs
are appended to a successful functional command.

## Parent handoff and exit criteria

FC9's `handoff-out.md` names changed paths, interface effects, current canonical
source identity, exact command/results, timing limits, unresolved findings and
remaining tasks. Reflection leads with the delta between this plan and delivery.
It explicitly states one of implementation incomplete, implementation complete
awaiting authorization/review, local accepted, or blocked with retained evidence.
None means published acceptance unless P2 has actually completed.

- Parent V2 (`6.2`): consumes the revised six-group local program and four native
  supplements, fresh inventory, separate preparation/review/gate instructions,
  same-image supervisor staging, actual FC8 receipt and operator timing disposition.
  Source-only child work cannot complete V2.
- Parent P1 (`7.1`): remains the separately authorized immutable source/tag/image
  publication task after V2; no publishing, pushing, issue messaging or tag changes
  are authorized by this child plan.
- Parent P2 (`7.2`): separately authorized pull/inspection and fresh qualified
  same-image physical boot/staging for each child, then exact published arm64
  and amd64 commands serially with streams/recovery only. Explicitly report the withdrawn
  per-child budget/compiler/credential/MCP-failure assurance; no rebuild or native
  supplement substitutes for either child's runtime receipt.
- Parent P3 (`7.3`): release receipts/setup/compatibility and authorized issue
  dispositions reflect observed coverage. #387 remains restricted-profile
  mitigation, #385 customer recovery remains separate, #382 remains external,
  and v1.1.0 remains immutable.

Child acceptance requires all implementation delivered, no silent scenario loss,
fresh same-image qualified preparation, independent authorized review with no
unresolved critical defect, complete authorized local functional evidence and
explicit operator timing disposition. If authorization is pending, stop at the
durable implementation handoff and leave evidence/acceptance tasks pending.

## Task model assignments

All keys use the child phase ID and `uar-outstanding-issues-v1-1-1` change ID.
The 2026-10-17 project policy and native role tool descriptions supply these
configured routes; configuration is not measured inference quality or cost.
Plan is frontier work. Use fresh contexts and the native scheduler; no gateway
credential changes or model smoke probes. No alternate family is assumed usable.

| Backend / stable | Model and effort | Native role / route | Reason and handoff |
| --- | --- | --- | --- |
| `1.1` / FC1 | `gpt-6-astra`, high | uar-lead, Codex native | Scope, OpenSpec and canonical coordination; `handoffs/FC1.md`. |
| `2.1` / FC2 | `gpt-5.6-terra`, high | uar-runtime, fresh native teammate | Bounded runtime fixture/reader implementation; `handoffs/FC2.md`. |
| `2.2` / FC3 | `gpt-6-astra`, high | uar-trust-tools, fresh native teammate | Credential/compiler authority and explicitly assigned recovery evidence; `handoffs/FC3.md`. |
| `3.1` / FC4 | `gpt-6-astra`, high | uar-lead, Codex native | Closed runner contract and cross-domain integration; `handoffs/FC4.md`. |
| `3.2` / FC5 | `gpt-6-astra`, high | uar-lead, Codex native | Complete-phase boundary and exact operator packet; `handoffs/FC5.md`. |
| `4.1` / FC6 | `gpt-6-astra`, high | uar-lead, sole image/staging writer | Separately authorized image and genuine qualified preparation; `handoffs/FC6.md`. |
| `4.2` / FC7 | `gpt-6-astra`, high | uar-verifier, fresh artifact-only context | Independent judge; same-family fallback is weaker and disclosed; canonical project-review findings paths. |
| `4.3` / FC8 | `gpt-6-astra`, high | uar-lead, sole gate writer | Authorized real-path receipt, operator timing disposition and parent V2 reconciliation; `handoffs/FC8.md`. |
| `5.1` / FC9 | `gpt-6-astra`, high | uar-lead, Codex native | Truthful reflection and parent publication handoff; `handoffs/FC9.md`. |

The current author is the frontier lead assigned to Plan. Reviewer family
separation remains unverified unless a genuinely configured different-family
judge is selected; native fresh-context review is not claimed cross-family.
No unsupported cost ceiling, benchmark, model entitlement or token-control
promise is made. If a route is unavailable, report it and preserve pending work;
the producer never self-certifies.

## Uncomfortable limit and present evidence

The proposal certifies explicit user-selected attachment activation; it does not
answer whether a real model selects the best skill. It reduces architecture-
specific release evidence and cannot promise faster execution before measurement.
The controlled peer can expose delayed application delivery but cannot establish
external provider responsiveness. Real supervisor qualification remains required
even if it is inconvenient or blocks release.

Present evidence is source inspection and authored Plan only. No test, check,
build, compiler/import probe, gate or product verification ran while writing it.
No guard or product feature was added. Exact runtime compatibility, timing,
65-scenario execution and produced artifacts remain unverified until the named
operator-authorized gates actually run.
