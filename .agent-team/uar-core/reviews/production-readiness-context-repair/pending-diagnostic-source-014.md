# Request014 A: pending-tool diagnostic source review

Verdict: **APPROVE for this bounded source assessment. No concrete blocking source defect found.** Graph implementation remains **OPEN**. The diagnostic changes do not establish a graph repair, whole-phase source completeness, compilation, or runtime acceptance.

## Scope and isolation

Reviewed only the fixed `target/production-readiness-476/repair-014-review/artifact.patch`, its `requirements.txt`, and necessary baseline source for tool identity, stream accumulation, host pending persistence, fixture admission, and graph diagnostics. Applied the existing repository/tool boundaries and rc.2 vendored dependency context. No old whole-phase finding hunt was reopened.

No producer rationale, histories, handoffs, memories, or other reviewers' reports were read. This reviewer retains its own earlier corrective context. This native same-family/non-specialist assessment is weaker than an independent-family specialist review.

## Source dispositions

### Native first-completion observation

At `src/llm/orchestrator.rs:3041`, the new branch records whether this stream index already has an accumulator before entry. It still merges the incoming ID through the existing immutable-ID rule at line 3043 before emitting anything. The new event at line 3055 is a `ToolCallDelta`, not a completed call or effect authorization.

The synthetic partial event is emitted only for a first completion at a previously unseen index and only without caller client-tool input (`src/llm/orchestrator.rs:3054`). Existing delta-plus-completion input therefore does not emit the full arguments a second time. The existing later event match still suppresses provisional `ToolCallComplete` events, and terminal EOF plus whole-batch identity validation still precede completed-call publication, approvals, or effects. Caller tool metadata remains private on this branch.

The host consumes partial deltas into `partial_tool_calls` (`src/uar/runtime/manager.rs:9583`), removes the partial on an admitted completion, and on interruption copies unresolved identified partials into the pending assistant turn before durable session saving. The new completion-only observation supplies the same existing retention path without changing host persistence or admission.

The expanded `native_identity_failed_or_incomplete_streams_discard_pending_completions` fixture covers completion-only and delta-plus-completion input, caller/native modes, normalized/typed failures, terminal-then-failure, and incomplete EOF. It retains zero completion/result/client-handoff/Done publication, zero effects, zero approvals and one driver request assertions. Its added observation assertion expects exactly one native partial with intact arguments and zero caller partials. These are authored assertions, not observed outcomes.

### Ordinary pending-history fixture

`tests/context_history_integrity.rs:806` installs a real `ProviderRegistry` populated by `tests/support/harness_context.rs:11`. The fixture's metadata explicitly enables tools for `openai/gpt-4o`; the artifact then selects that provider/model rather than relying on default policy selection. The synthetic stream remains an explicit test driver. No production capability bypass, fabricated exact-budget contract, or production mock evidence is added.

The verified owner, owner-derived session key, local durable store, complete pending assistant history, `MissingResult` assertion, durable/in-memory equality, second-run refusal and final exact-one-provider-request assertion remain. Added assertions require actual `RunStart`, the intended `PROVIDER_INTERRUPTED` event and one initial call (`tests/context_history_integrity.rs:833`), then `invalid_history` mentioning the pending call (`tests/context_history_integrity.rs:892`). A route/admission failure cannot satisfy the intended interruption-stage assertions. The shared identity fixture still verifies identity and keeps the local-owner opt-in false.

### Graph diagnostics only

`tests/context_history_integrity.rs:2275` retains the three-second capture timeout. The loop also stops on disconnected capture or terminal Error/Cancelled and produces failure diagnostics containing run state, events, execution count and provider count. This reports the failure rather than widening the timeout or bypassing the graph.

Successful capture still requires one graph-node execution and zero provider calls before continuing. Raw/revoked checkpoint denial, legacy admission rejection, unchanged execution/provider counters, and unchanged warm history assertions remain after capture. No graph production implementation is changed by this patch. The underlying graph cause is unresolved and cannot receive repair or acceptance credit from this review.

## Execution and limits

All shell operations were source inspection using `cat`, `sed`, `rg` or `nl`, with explicit workdir `/Users/gqadonis/.claude/worktrees/production-readiness-476`. A few exploratory reads named absent graph source files; the relevant graph type was subsequently located and read. These were inspection misses, not executed checks.

**Executed checks: zero.** No Cargo, tests, compiler/import probes, check/lint/fmt/typecheck, builds, API/container calls, machine screening, `git diff --check`, or delegation occurred. The only write is this assigned report. Product source, other checkouts and services remained untouched. No unsolicited implementation or guards were added.

Compilation, fixture outcomes, pending durable persistence, resume refusal, malformed stream behavior and the graph diagnostic result remain unverified. This review grants zero compiler/runtime/provider-qualification credit and does not authorize another gate.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE for the bounded request014 source artifact. No concrete blocking defect found; graph implementation remains OPEN and runtime acceptance remains unverified.
