# Runtime plan — `uar-outstanding-issues-v1-1-1`

Phase: `uar-outstanding-issues-v1-1-1`  
Stage: Plan  
Scope owner: `uar-runtime`  
Plan status: proposed execution ledger; no production edit, fixture run, build,
formatter, validator, test, or review has run.

## Scope and source boundary

This plan registers only the stable OpenSpec tasks `R1`–`R5` and runtime's
integration contribution to `T2`. It does not renumber, split, or add tasks.
The full-phase implementation boundary remains closed. Future test and
acceptance paths named below are implementation inventory only; they are not
authorized to run until lead-owned `L4` records every phase implementation item
complete.

The source trace starts at `RunManager::execute_request_inner` in
`src/uar/runtime/manager.rs`. It resolves a root configuration, captures it
once with `RunModelBindings::capture`, and only then enters context assembly,
ordinary orchestration, graph execution, or a delegated child. The current
kernel in `src/uar/runtime/thread/kernel.rs` obtains a policy-constrained clone
through `RunModelBindings::for_policy`; it does not construct a provider. Its
cancellation branch cancels then awaits the captured execution. `ThreadService`
retains reservation and durable-write ownership. The changes below must preserve
those facts.

Graph evidence was used to identify candidate paths, then exact source was read.
The graph does not establish runtime behavior. No runtime result is claimed.

## Binding-caller inventory for T2

`T2` must close every root selection route before `RunModelBindings::capture`.
The one decision source is trust's nonserializable `CredentialSourceDecision`:
it carries an allow/deny result, authorized connection material, and safe
provenance. Runtime consumes it and does not inspect policy internals, deserialize
it from a caller, synthesize a bearer grant, or turn denial into another
fallback. Each selected primary and fallback candidate requires its own decision,
including a supplied primary driver.

| Binding caller or route | Source evidence | Runtime treatment and boundary |
| --- | --- | --- |
| HTTP, AG-UI/full-harness resume and ordinary API request | `src/uar/api/routes.rs` constructs `RunExecutionRequest`; `RunManager::execute_request_inner` owns root resolution | The API stays lead/trust-owned. Its authenticated capture and SDK inputs reach the one runtime decision/capture seam; runtime rejects a denied candidate before provider construction or tool admission. |
| OpenAI-compatible and ACP SDK paths | `src/uar/api/openai/routes.rs`, `src/uar/api/acp/handler.rs` call `RunManager::execute_request` | They inherit the same root seam. No transport-specific credential resolver or fallback is added. |
| Embedded SDK | `src/embedded.rs` constructs `RunManager` and calls `start_run`; the builder currently need not install `ProviderService` | The absent encrypted service still produces a policy decision for the configured connection. `RunManager` remains the enforcement consumer; embedded construction changes, if any, are a lead-owned composition handoff. |
| Ordinary primary and ordinary fallback | `manager.rs` currently calls `apply_credential_layer` for the selected root and uses `RunModelBindings::capture` to construct fallback clients | Replace both implicit fallback behaviors with per-candidate decision evidence. `capture` must reject a supplied primary or internally constructed fallback without the corresponding decision. |
| Team profile and team fallback | `manager.rs` builds `team_profiles`, resolves profile fallbacks, then captures the root binding | Preserve existing `TEAM_*` public meanings. Trust decides every profile/fallback source; runtime retains only approved profile connection material and does not convert a team denial into ordinary config fallback. |
| Graph run | `manager.rs` selects graph execution after root binding and separately preserves the graph terminal guard | Graph work uses the same already-captured root catalog. It cannot re-resolve a credential or replace a root connection. No graph state-machine rewrite belongs to this task. |
| Summarization/context assembly | `manager.rs` states capture occurs before summarization and passes `summarization_driver` into `turn::builtin::ContextStage` | Summarization uses only the captured root driver/candidate selected under the decision. It must not create a second connection or invoke a resolver. Its final-wire contract remains unchanged. |
| SDK-supplied `run_credentials` | `src/uar/api/routes.rs::attach_host_resources` validates `RunCredentials`; `manager.rs` handles `run_credentials` during root resolution | A host-supplied connection remains an input, not proof of source authorization. It needs the per-candidate decision before `capture`, and its scrubber/provenance stay host-owned. |
| Inherited/delegated child | `thread/kernel.rs` calls `models.for_policy`; `RunModelBindings::for_policy` narrows the captured catalog | No trust lookup or provider construction occurs in a child. It may select only the captured decision-backed bindings, intersected policy and budget. Cancellation still joins the child execution before the kernel returns. |
| Credential-free configuration | `apply_credential_layer` currently returns unchanged config when `ProviderService` is absent | Trust supplies an explicit allow/deny decision even without encrypted service. Runtime retains the compatible configured connection only when that decision allows it. |

## Task ledger and exclusive runtime surfaces

The paths below are the execution ownership map. “Existing” means the plan
assigns a change to an observed file. “Proposed new” identifies a required file
not currently present. A path listed as lead, trust, or state-owned is an input
or handoff boundary, not runtime write authority.

### R1 — typed root-host binding diagnostics

**Runtime writer owns existing:**

- `src/uar/runtime/manager.rs` — replace the current opaque binding-error
  collapse at the root composition seam with an allowlisted private
  stage/category/reference projection. Keep ordinary and `TEAM_*` public codes,
  `orchestrator_start_failed` wire shape, secret scrubbing, and refusal before
  provider or tool effects.
- `src/uar/runtime/cost_budget.rs` — expose typed, nonformatted budget/accounting
  origin to the root seam. Preserve configured limits, durable reservation and
  unknown-liability behavior.
- `src/uar/runtime/thread/policy_intersection.rs` — preserve a typed artifact
  budget failure origin needed by R1; do not relax intersection or child policy.
- `src/uar/runtime/turn/bindings.rs` — preserve typed client-construction and
  budget-admission origin through the one root capture boundary; no kernel
  rebinding API.
- `docs/operations/durable-accounting.md` — add the safe diagnostic meaning,
  attestation prerequisite, and the statement that it does not identify erased
  deployment history or grant remote durability.

**State handoff (state writer):** a captured safe persistence eligibility
classification through the existing persistence capability boundary. Runtime may
map only this typed classification; it must not query an endpoint, infer
durability, or log a source error chain.

**Trust handoff (trust writer):** safe authenticated owner/principal context is
input to durable accounting; runtime must continue to treat a missing verified
principal as `verified_principal_required`, with no provider contact.

**Ordering:** accept S1's interface contract first; implement the source
origins and root classifier next; then wire the existing error emitter. R1 must
finish before R5's diagnostic fixture/docs integration.

### R2 — ordinary manager completion truth

**Runtime writer owns existing:**

- `src/uar/runtime/manager.rs` — add an ordinary-loop-local observed
  `NormalizedEvent::Done` condition. EOF without it and without confirmed
  cancellation becomes the existing host error projection with an allowlisted
  missing-completion code before successful settlement.
- `docs/operations/provider-stream-completion.md` — describe manager completion
  proof, EOF failure, and the unchanged cancellation/join/cleanup precedence.

The graph terminal guard and provider terminal guard are already separate. R2
does not consolidate state machines, publish a synthetic `Done`, execute a
partial tool call, rewrite a receipt, or change the cleanup order: terminal
operation completion, actor-root shutdown, sandbox completion, approval
settlement, then interrupted-history/checkpoint and final status.

**Ordering:** R2 may proceed independently of R1/R3 source edits, but R5 must
integrate its controlled EOF/cancellation scenario after R2's source and docs
are complete.

### R3 — native Anthropic byte framing

**Runtime writer owns existing:**

- `src/llm/anthropic_driver.rs` — insert the local adapter only before the
  pinned `liter_llm::SseDecoder`; keep event/payload agreement, known-event
  validation, `message_stop`, exactly one decoder finish, and post-terminal
  failure semantics.
- `src/llm/anthropic_types.rs` — only if the existing known-type boundary needs
  a helper to distinguish an unknown string type from malformed known data.
- `docs/operations/provider-stream-completion.md` — record initial-BOM and
  CR/LF/CRLF normalization, partition invariance, incomplete-frame limits, and
  forward-compatible unknown-kind handling.

**Runtime writer owns proposed new:**

- `src/llm/anthropic_sse.rs` — an incremental byte framer that accepts one
  initial BOM and normalizes CR, LF, and CRLF without creating EOF dispatch.
  It emits the normalized LF immediately for a CR and remembers only whether
  to swallow a following LF; a complete event cannot wait for another byte.
  It maintains bounded incomplete-frame accounting compatible with Liter's
  1 MiB bound and supplies
  only canonical bytes to the existing decoder.

**Lead handoff:** `src/llm/mod.rs` is the shared module declaration surface.
The lead adds the one private module declaration after R3's file lands; runtime
does not take a second writer on that root.

The adapter does not replace Liter, add a dependency, change its pinned
revision, reinterpret SSE fields, or accept malformed known events. A valid
unknown top-level string type with an empty/matching explicit event is ignored;
missing/non-string type, disagreement, invalid JSON, malformed known data,
post-stop data, incomplete frame, and EOF without `message_stop` remain errors.

**Ordering:** R3 has no source dependency on R1/R2. R5 receives the source
adapter interface and documents/fixtures its byte-partition matrix.

### R4 — MCP outcome scenario completion

**Runtime writer owns existing scenario surfaces:**

- `tests/support/truthful_tool_outcome_peer.rs`
- `tests/support/truthful_tool_outcome_process.rs`
- `tests/truthful_tool_outcomes.rs`
- `tests/tool_call_protocol.rs`
- `docs/operations/truthful-tool-outcomes.md`

The inspected production path already distinguishes application failure,
transport failure, and `OutcomeUnknown`: `src/mcp/registry.rs` translates
`isError=true`; `src/mcp/runtime.rs` preserves timeout/disconnect certainty;
`src/uar/runtime/tool_admission/lifecycle.rs` projects the typed terminal
outcome; and `manager.rs` blocks unknown outcomes from checkpoint continuation.
Those files have no runtime change assigned unless fixture work exposes an
observed mismatch. R4 supplies real-peer fixtures for true, false, omitted
`isError`, protocol error, timeout, post-dispatch disconnect, native incidental
`isError`, and a later independent recovery. It must retain the original
canonical receipt and never replay an uncertain effect.

**Ordering:** R4 is source-independent from R1–R3. R5 composes its failed-MCP
and incomplete-stream case using R4's peer helper. Trust owns MCP authorization
and state owns receipt persistence; R4 does not alter either contract.

### R5 — full-harness integration and operator documentation

**Runtime writer owns existing:**

- `tests/anthropic_driver_tests.rs`
- `tests/stream_terminal_contract.rs`
- `tests/bauar_full_harness_cursor.rs`
- `tests/support/stream_terminal_peer.rs`
- `tests/fixtures/stream_terminal_https.py`
- `tests/fixtures/truthful_tool_outcome_mcp.py`
- the R4 scenario surfaces above
- `docs/operations/durable-accounting.md`
- `docs/operations/provider-stream-completion.md`
- `docs/operations/truthful-tool-outcomes.md`
- `docs/skill-pack-installation.md`

**Runtime writer owns proposed new:**

- `scripts/release/uar_1_1_1_runtime.py` — a deterministic runtime group adapter
  exposing `run_group(context) -> result` for the R1 refusal/qualified control,
  R2 EOF/cancellation, R3 framing/partitions, R4 outcomes, and their combined
  negative control. It configures only runner-owned provider/MCP peers and
  returns structured observations. It contains no process lifecycle, source
  build, credential value, manifest interpretation, or acceptance verdict.

NEW `scripts/release/uar_1_1_1_acceptance.py` is lead-owned under L3; the
historical v1.1.0 runner and receipts remain unchanged.
Runtime hands it the adapter import contract, required private inputs, and
structured observations; the lead alone integrates six phase gate groups and
manifest/receipt handling, including common `support.py` lifecycle. T4 and S4
provide their own `run_group(context) -> result` scenario adapters by their
documented handoff interfaces. R5's skill observation records only the resolved
root, sanitized path, and error category in `docs/skill-pack-installation.md`.
It does not change the pinned skill-system submodule, assume a missing skill
version is causal, or alter compiler behavior.

**Ordering:** R5 begins after the relevant R1–R4 source and documentation
interfaces are settled and after trust/state publish their T4/S4 adapter
contracts. It completes before L3/L4, but none of its fixtures execute before
the full-phase boundary.

## T2 runtime integration

**Runtime writer owns existing:**

- `src/uar/runtime/manager.rs` — replace `apply_credential_layer` with a single
  root-host adapter that requests `CredentialSourceDecision` for every selected
  primary/fallback candidate, including ordinary, team, SDK-supplied,
  credential-free, and supplied-primary candidates. Carry nonserializable safe
  provenance through binding capture, emit a policy-denial outcome before any
  provider contact, and retain inherited retry/timeout values.
- `src/uar/runtime/turn/bindings.rs` — require a decision-backed candidate at
  capture time, preserve it in the captured catalog, and reject supplied drivers
  or internally constructed fallbacks lacking evidence. `for_policy` may narrow
  this catalog only; it does not resolve again.
- `src/uar/runtime/thread/kernel.rs` — consume the already-captured catalog
  only. This is a compatibility audit/change only if the explicit provenance
  type must be carried through `InheritedRunBindings`; it cannot become a
  resolver, mutator, reservation owner, or durable writer.

**Trust-to-runtime interface:** trust provides `CredentialSourceDecision` with
allow/deny, approved connection material, and nonserializable safe provenance.
It must reject unapproved source/provider/issuer/tenant/install combinations
without returning a fallback candidate. Runtime treats it as opaque authority,
using neither a public bearer representation nor raw policy inspection.

**Lead handoff:** `src/server.rs`, `src/embedded.rs`, API routes, provider-admin
and settings construction remain L2/shared composition surfaces. The lead wires
the trust resolver into server and embedded construction and verifies every
transport still funnels into `RunManager`; runtime does not take a second writer
on these files. If the embedded builder requires a new resolver injection, lead
owns that small construction change after the interface is frozen.

**State handoff:** no T2 runtime code writes source decisions, credential rows,
or journals. State persistence remains behind trust's resolver and existing host
interfaces. Captured provenance must be safe for in-memory authorization only,
not a durable receipt schema addition.

**Sequence:** T1 policy types and resolver contract first; T2 manager/binding
integration second; L2 construction wiring after both interfaces are frozen;
T4 fixtures consume the completed seams. R1 follows S1 independently for
durability classification, while R5 composes accepted T2 behavior only through
the lead-owned full harness.

## Preserved invariants and limits

- The trusted host owns policy intersection, candidate decisions, reservations,
  admission, checkpoints, tool-result receipts, and every durable mutation.
  Agent kernels receive captured, narrowed authority only.
- Cancellation retains its current owner and joins the captured child execution;
  it never detaches a kernel or turns an unconfirmed cleanup into success.
- Tool outcomes remain typed and immutable. An uncertain external effect blocks
  automatic replay and remains a liability across checkpoint/recovery paths.
- `versions.toml` remains unchanged: Liter is pinned at 2.1.1 and its recorded
  commit. No dependency, vendored source, skill-system submodule, generic
  provider route, retry policy, or timeout inheritance change is selected.
- A safe diagnostic explains only stage/category/reference. It cannot prove the
  reported deployment's remote store, recover an erased inner error, or repair
  customer data. The phase must not claim otherwise.

## Deferred verification inventory

No verification command was run for this planning work. At the later L4 boundary,
R5 supplies the runtime parts of the `binding`, `streams`, `mcp-outcomes`, and
`combined` L3 groups. The final runner must use real controlled peers, isolated
storage/ports, sanitized transcripts, and persisted result/receipt inspection.
Source tests and fixtures above are not completion evidence; the completed
phase's serialized real-path gate is the only planned acceptance claim.

## Unresolved source items for execution

1. The exact Rust shape and module path of trust's `CredentialSourceDecision`
   remain a trust-owned interface detail. Runtime requires the stated opaque
   capabilities, not a particular serialization or crate layout.
2. State must name the captured eligibility type and error mapping R1 receives.
   Runtime must not reverse-engineer it from persistence errors.
3. The existing release runner currently carries v1.1.0 text and gate inventory.
   L3 owns reconciling it to the v1.1.1 six-group phase contract; R5 only hands over
   its adapter interface and does not edit the shared runner.
4. The full two-way byte partition matrix and actual real-peer behavior remain
   unverified until the phase boundary opens. This plan makes no runtime-pass
   claim from source inspection.


## Lead consolidation — 2026-10-16

The canonical plan orders runtime work R3, T2, R1, R2, R4, R5 with one writer.
T2 waits for T1/T3 finalized source authority; R1 also waits for S1. References
to source independence above never permit two runtime tasks to edit manager.rs
or shared docs concurrently. CredentialSourceDecision is declared in the NEW
trust-owned credentials/policy.rs; S1 owns the typed eligibility declaration
in persistence/mod.rs. Concrete Rust signatures are recorded in those source
handoffs before consuming edits; no additional architecture decision is needed.

The runtime adapter's final name is uar_1_1_1_runtime.py. It owns binding,
streams, mcp-outcomes and combined group observations. L3 support.py owns
shared process lifecycle; runtime may request owned peers through that interface
and must report their handles for joined cleanup. R5 owns the existing full
bauar target and named peer fixtures above; trust/state supply helpers only.
