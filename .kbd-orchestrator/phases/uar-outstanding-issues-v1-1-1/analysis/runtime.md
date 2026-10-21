# Runtime design analysis

Phase: `uar-outstanding-issues-v1-1-1`  
Stage: Analyze  
Scope owner: `uar-runtime`  
Evidence type: source and assessment evidence only. This is not a runtime
reproduction, implementation plan approval, or verification receipt.

## Decision summary

Three bounded runtime changes belong together:

1. Keep budgeted runs fail-closed, but retain a typed, secret-safe origin when
   root-host model binding cannot be captured.
2. Make the ordinary `RunManager` path require an observed orchestrator
   `NormalizedEvent::Done` before it can settle a run as `Done`.
3. Add a local Anthropic SSE framing adapter before the pinned Liter decoder so
   native Anthropic accepts CR, LF, and CRLF framing while keeping malformed
   known events fatal and harmless future event kinds ignorable.

No dependency update, Liter change, skill-system submodule change, durability
attestation change, provider-routing change, retry change, or kernel write path
is selected. In particular, the `InheritedLlmSettings` path remains the source
of resolved timeout and retry values, preserving issue #384.

## Source findings and design

### 1. Typed, secret-safe model-binding refusal

`RunManager` builds `ModelCallBudget` at
`src/uar/runtime/manager.rs:5296-5324` and calls durable accounting before
`RunModelBindings::capture`. An explicit token ceiling therefore reaches
`ModelCallBudget::with_durable_accounting` even when `cost_tracking` is false.
That host operation currently returns opaque `anyhow::Error` values, including
`durable_accounting_unsupported` and
`accounting_verified_principal_required` at
`src/uar/runtime/cost_budget.rs:1432-1438`. The following catch-all at
`manager.rs:5357-5403` maps every non-team failure to `Run model binding
failed` and publishes only `orchestrator_start_failed`.

This masks the observed bounded-remote-Surreal refusal before a provider client
is constructed. It must not be repaired by dropping the limit, changing the
remote durability rule, or logging arbitrary source errors: provider
construction can still own credentials that are outside the captured scrubber.

Introduce one private `RunBindingFailure` classification at the root-host
composition seam. It should carry only an allowlisted `stage` and `category`,
not an arbitrary source error, endpoint, provider response, model
configuration, credential, or formatted error chain. The proposed stages and
categories are:

| Stage | Category | Origin that must be classified at source |
| --- | --- | --- |
| `budget` | `invalid_configuration` | `ThreadBudgets::from_artifact` or `ModelCallBudget::for_run` rejects the stated limit. |
| `accounting` | `unsupported_backend` | `with_durable_accounting` sees no store or one without the durable capability. |
| `accounting` | `verified_principal_required` | Durable accounting has no middleware-verified accounting subject. |
| `accounting` | `storage_operation_failed` | An accounting initialization/read/write operation fails after eligibility is established. |
| `team_profile` | `unsupported`, `route_mismatch`, `revision_conflict`, or `reasoning_unsupported` | Existing explicit team outcomes retain their current meaning. |
| `inherited_binding` | `unavailable` or `policy_rejected` | An inherited run binding cannot satisfy the captured root policy. |
| `budget_admission` | `rejected` | An otherwise constructed bound model cannot reserve/admit the approved budget. |
| `client_construction` | `rejected` | `RunModelBindings::capture` cannot create the primary client. |

The categories must remain internal representation details except for a stable
allowlisted public `orchestrator_start_failed` message/code extension and a
fresh diagnostic reference. The event must tell an operator that a run was
refused during its stated stage and identify the reference. It must not state
the source error. Structured logs may include the reference, stage, category,
`accounting_required`, `durable_backend_supported`, and
`verified_principal_present`. They must not include formatted source errors,
credential values, endpoint URLs, provider response bodies, or the full
effective model configuration.

`RunModelBindings::capture` is already root-host-only
(`src/uar/runtime/turn/bindings.rs:194-198`), and child policy selection in
`RunModelBindings::for_policy` only narrows captured clients and budgets. Keep
the classifier on that trusted host path. Do not grant a kernel the ability to
rebind a provider, mutate accounting, or write a diagnostic receipt.

`src/uar/runtime/thread/kernel.rs` confirms the required boundary: it captures
an owner-qualified live root, freezes MCP bindings, reads a narrowed budget,
rechecks policy and live-root state, and has the thread service retain
admission reservations and durable mutations. Its child cancellation awaits the
captured execution after cancellation rather than detaching it. The model
binding diagnostic must not alter those captured-authority, reservation,
checkpoint, tool-result, cancellation, or join contracts.

### 2. Manager terminal invariant

`Orchestrator` already requires a provider-terminal event for each driver
attempt (`src/llm/orchestrator.rs:2438-2502`) and emits `NormalizedEvent::Done`
when its complete logical orchestration is finished. The ordinary manager loop
at `src/uar/runtime/manager.rs:7024-7041` breaks on `stream.next() == None`.
It maps `NormalizedEvent::Error` to `run_failed`, but leaves `Done` in the
catch-all event arm. Its settlement then marks the run `RunStatus::Done` when
neither cancellation nor another error was recorded
(`manager.rs:7438-7528`). The manager therefore does not independently prove
that its orchestrator completed.

Add one ordinary-loop-local `orchestrator_completed: bool`, initialized false.
Set it only after the manager observes `crate::normalized::NormalizedEvent::Done`
from the orchestrator stream. Do not use `RuntimeStepKind::Finished`, a provider
SSE `message_stop`, a client stream closure, an API `RunDone` envelope, or a
tool result as substitute proof.

When the stream exhausts without that observation and the run was not cancelled,
mark the run failed and emit the existing host error projection with a new
allowlisted code such as `orchestrator_completion_missing`. Do this before the
normal success-history branch and final `RunStatus` settlement. Then leave the
existing terminal cleanup order intact:

1. terminal operation completion;
2. actor-root shutdown;
3. sandbox completion;
4. approval cancellation settlement;
5. interrupted history/checkpoint projection and final status.

Cleanup errors continue to convert a nominal cancellation to error because they
represent unconfirmed effects. Cancellation continues to win only while
cleanup remains confirmed. The new missing-completion error must not suppress
a later cleanup error, publish a second synthetic `Done`, execute pending
tools, or rewrite a tool receipt. A valid completed text turn and valid
completed tool loop must still reach the original `Done` path.

### 3. Native Anthropic framing and forward-compatible unknown events

The native driver already routes bytes through the pinned
`liter_llm::SseDecoder` (`src/llm/anthropic_driver.rs:375-452`). It validates
payload JSON, verifies a nonempty SSE `event` name agrees with payload `type`,
rejects malformed known data, requires `message_stop`, calls `decoder.finish`,
and only then publishes `NormalizedEvent::Done`. The current pin is Liter
2.1.1 at `e446f5ef91003bbec5fce6b2f45fb94559dff335`; changing it is outside
this repair.

The source assessment found the pinned decoder recognizes LF framing but does
not establish lone-CR support. Put a small UAR-owned byte framer before the
Liter decoder, preferably as `src/llm/anthropic_sse.rs` called only by the
native Anthropic driver. It owns byte-boundary normalization, not SSE field or
payload semantics:

* accept one optional leading UTF-8 BOM;
* normalize CR, LF, and CRLF into the canonical LF stream expected by Liter;
* retain a trailing CR until the next byte establishes whether it starts CRLF;
* preserve bytewise partition invariance, including splits in CRLF, UTF-8, and
  JSON escapes;
* do not manufacture a blank-line dispatch at EOF; call Liter's existing
  `finish` exactly once after the framer has completed its own EOF validation;
* keep Liter's existing bounded-frame and field/data handling authoritative.

The adapter must preserve Liter's cumulative 1 MiB incomplete-frame budget.
The pinned decoder counts buffered data, event-name bytes, pending bytes, and
the whole incoming chunk; feeding it arbitrarily smaller fragments must not
make an oversized incomplete frame acceptable. The adapter therefore needs its
own cumulative bound accounting across normalized output and withheld CR/BOM
state, with the same failure behavior at or before Liter's effective limit. It
may consume exactly one initial BOM before the first field; a later BOM is
ordinary data. This explicitly follows the SSE framing contract rather than
silently changing payload behavior.

The framing rule is grounded in the WHATWG event-stream parsing algorithm:
<https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream>.
This is local adaptation around the pinned integration, not a claimed behavior
change in Liter.

The current `StreamEvent` enum deserializes only known variants, so a valid
future type falls into the same malformed-event error as an invalid known one.
At the native-driver boundary, inspect the valid JSON `type` before full enum
deserialization. For an unknown top-level string type whose explicit `event` name is
empty or agrees with that type, ignore the frame without a normalized event or
state mutation. A known type that fails its expected shape remains a fatal
`Malformed Anthropic SSE event`; a missing/non-string `type`, event/payload
type disagreement, invalid JSON, data after `message_stop`, incomplete frame,
or EOF without `message_stop` also remain failures. `ping` remains ignorable.
This matches Anthropic's documented forward-compatibility guidance for added
event types:
<https://platform.claude.com/docs/en/build-with-claude/streaming#other-events>.

Do not add the unpinned `eventsource-stream` candidate. Its 0.2.3 release date
(2022-02-17) and a replacement of the pinned decoding path would expand
dependency and regression scope without resolving the ownership boundary.

### 4. Bundled skill missing-version startup observation

This observation is legitimate triage, but no runtime or compiler production
change is selected from the current evidence.

`src/uar/runtime/skills/builtin_loader.rs:67-77,444-504` parses builtin
SKILL.md `version` as `Option<String>` and assigns `"0.0.0"` when absent.
It can warn that a builtin skill failed to load, but absent per-skill version
alone is not such a failure in this loader. `PackProvenance` similarly states
that a missing pack manifest/version must load as unknown
(`src/uar/runtime/skills/provenance.rs:27-83`). The compiler's
`metadata.version` requirement is for an agent descriptor at
`src/uar/compiler/stages/s01_frontmatter.rs:34-42`; it is not the builtin
SKILL.md startup path.

The current startup warning may instead come from a selected external root,
another pack consumer, a deployment-catalog `SkillRef`, or a loader failure
whose error text contains a separate version requirement. The next bounded
evidence is the sanitized failing path and exact error together with the
existing `skills.pack.resolved` source/root/version/skill-count record. Do not
add version fields across the pinned skill-system submodule, alter its gitlink,
or weaken descriptor validation before that evidence identifies the consumer.

## Proposed ownership and changed-path candidates

| Owner | Candidate path(s) | Responsibility |
| --- | --- | --- |
| `uar-runtime` | `src/uar/runtime/manager.rs`, `src/uar/runtime/cost_budget.rs`, `src/uar/runtime/turn/bindings.rs` | Typed run-binding classification, secret-safe public projection, and manager-observed completion. |
| `uar-runtime` | `src/llm/anthropic_driver.rs`, new `src/llm/anthropic_sse.rs`, `src/llm/anthropic_types.rs` if an internal raw/known event split is needed | Local CR/LF/CRLF framing and unknown-kind handling around pinned Liter. |
| `uar-runtime` | `tests/bauar_full_harness_cursor.rs` or a dedicated full-harness runtime target; `tests/stream_terminal_contract.rs`; `tests/anthropic_driver_tests.rs`; a narrow byte-stream fixture under `tests/support/` or `tests/fixtures/` | Real-path manager terminal cases and native Anthropic framing scenarios. Do not create unit-only substitutes for the required harness path. |
| `uar-runtime` | `docs/operations/provider-stream-completion.md`, a new bounded runtime-diagnostics/operator document if no existing document fits | Public code/reference meaning, diagnostic non-disclosure rules, verified durability prerequisites, and skill-root triage receipt guidance. |
| `uar-state` (coordination required) | Accounting persistence error source only if `with_durable_accounting` needs to distinguish initialized-store failure from backend ineligibility | Preserve `AccountingError`, durability semantics, reconciliation and durable-mutation ownership. Runtime owns the host mapping. |
| `uar-providers` (coordination required) | Provider-facing diagnostic wording only if a public provider status contract is shared | Preserve explicit endpoint routing and #384 inherited timeout/retry values. |

No shared root, manifest, dependency pin, vendored Liter tree, or
`crates/prometheus-skill-system` file is a candidate path for this design.

## Required fixtures and documentation acceptance inventory

The eventual phase plan needs all of these production items before its one
verification boundary opens:

1. A full-harness ordinary-manager fixture that covers empty EOF,
   metadata-only EOF, text EOF, partial tool JSON EOF, and completed tool-call
   EOF without the orchestrator terminal; each must end as error, never as
   `RunStatus::Done` or tool execution. Include cancellation, a valid text
   completion, a valid tool loop, and fallback controls.
2. A binding fixture through the run admission path for invalid budget,
   unattested/unsupported durable accounting, missing verified principal, and
   client construction. Assert only allowlisted public failure code/message and
   diagnostic reference; inspect test logging/serialized events to ensure no
   credential, endpoint, raw configuration, or opaque source message appears.
3. An authenticated real-path configuration case retaining a per-agent explicit
   endpoint plus operator `timeout_secs` and `max_retries`, proving the #384
   inheritance fields remain through registry resolution and captured bindings.
4. Native Anthropic byte-stream cases for every two-way split and representative
   fixed partitions across LF, CRLF, lone CR, BOM, multiline data, UTF-8,
   escaped JSON, comments/ping, unknown valid kinds, malformed known kinds,
   type/event disagreement, trailing data after `message_stop`, oversized or
   incomplete frame, and EOF before `message_stop`. Equivalent wire content
   must normalize to equivalent event sequences for the three newline forms.
5. Operator documentation that says remote durability attestation follows
   verified infrastructure durability and does not create it; it must preserve
   ledger reconciliation requirements. Include skill troubleshooting that
   records the resolved root and sanitized failing file/error rather than
   instructing users to mass-edit bundle versions.

## Unresolved choices and limits

* The live persisted agent configuration and original inner binding error were
  not read. The leading cause is source-supported, not a reproduced fact.
* The exact source of `storage_operation_failed` needs the accounting owner to
  identify an existing typed persistence boundary. Do not parse `anyhow` text
  to derive it.
* The eventual public diagnostic shape needs an OpenSpec delta and compatibility
  decision: preserve existing `orchestrator_start_failed` while adding safe
  structured detail, or introduce a versioned diagnostic field. This analysis
  does not choose a wire change by itself.
* The ordinary completion guard must be applied only to the confirmed ordinary
  manager path. Graph/other paths need separate source mapping before any
  shared abstraction is introduced.
* Source inspection confirms the builtin loader's absent-version fallback, but
  no retained startup warning included the selected file path and exact nested
  error. The bundle issue remains unassigned implementation work.
* The local Compass graph query failed because the checked-in snapshot declares
  `compass.store.graph-index/2` while the installed CLI expects version 1.
  Direct source inspection supplied the findings above; no graph-derived
  relationship is claimed as runtime proof.

## Verification status

No test, build, Cargo command, formatter, linter, typecheck, model probe, or
review gate ran. The active phase's production implementation boundary remains
closed. All fixture and documentation items above are planned implementation
work, not acceptance evidence.
