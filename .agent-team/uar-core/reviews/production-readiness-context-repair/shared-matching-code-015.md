# Gate015 A: bounded shared-matching source review

Verdict: **APPROVE. No concrete blocking source defect found in the selected artifact.** This is source-only assessment, not compiler/runtime/provider qualification or phase acceptance.

## Scope and isolation

Reviewed the fixed `target/production-readiness-476/repair-015-review/artifact.patch`, its `requirements.txt`, and necessary baseline source for matching capture, service selection, native partial observations, host pending retention and fixture assertions. Repository rules and rc.2 manifest/vendor selection remain applicable. No old whole-phase finding hunt was reopened.

No producer conversation, rationale, handoffs, memories, or other reviewers' reports were read. This reviewer retains its own previous corrective context; this native same-family/non-specialist review is weaker than independent-family specialist review.

## Matching setup disposition

The ordinary fixture creates a real `SkillService` with the same local durable persistence object (`tests/context_history_integrity.rs:772`), explicitly selects `Service`/`Keyword`, passes `Arc::clone(skill_service.registry())` into the manager at line 798, and attaches that service at line 811. The graph fixture performs the equivalent construction at line 2262 and its manager construction immediately below.

The production path supports this setup: `RunSkillBindings::capture` obtains `service.matching_snapshot()` and shares that snapshot's registry for matching/activation (`src/uar/runtime/turn/bindings.rs:177`). The snapshot reconciles selection, captures the configured method, and clones the real registry (`src/uar/runtime/skills/service/selection.rs:189`). With no settings-manager override, manager admission selects this captured service engine/configuration (`src/uar/runtime/manager.rs:5821`), and the Keyword branch performs local matching without an LLM/vector resource (`src/uar/runtime/skills/service.rs:915`). No classifier owner, fake vector or proof replaces production selection. Production defaults are unchanged.

Both fixtures now assert `requested_method` and `effective_method` equal `skill_service.keyword`, with null requested/effective model values (`tests/context_history_integrity.rs:839` and line 2318). These assertions correspond to the production receipt construction at `src/uar/runtime/manager.rs:6054`; they cannot silently accept the prior missing selected-matching resource. The supplied requirement identifies the observed `skill_matching_unavailable` issue; this review inspected its source path and does not claim to have reproduced or resolved its runtime outcome.

The ordinary fixture retains explicit `openai/gpt-4o` policy selection, real tool-capability registry admission and a captured synthetic response driver. It does not fabricate an exact budget, add a production mock capability, or bypass admission. The graph fixture still requires zero provider calls.

## Native partial-observer disposition

`src/llm/orchestrator.rs:3041` detects only an unseen accumulator index, merges the immutable ID before publication, and emits one native-only partial delta at line 3054. Existing delta-plus-completion input does not emit a second full argument payload. Caller client-tool input still suppresses the provisional metadata. Completed calls remain withheld until terminal EOF and whole-batch validation; approvals/effects remain downstream of that validation.

The host's existing partial-call accumulation and interruption persistence path remains intact. The expanded failed/incomplete-stream fixture retains zero completed calls/results/client handoff/Done, zero approvals/effects and one provider request, while checking one native partial and no caller partials. No change in this artifact publishes a completed call early.

## Preserved assertions and boundaries

- Ordinary fixture: verified owner, owner-key lookup, local durable store, pending call, empty assistant text, `MissingResult`, durable/in-memory history equality, RunStart/intended interruption, second `invalid_history` refusal and exactly one provider request remain.
- Graph fixture: the capture timeout remains three seconds; the capture node counter increments before sending state. Capture still requires execution count one and provider count zero. Diagnostic failures include run/events/counters without extending the timeout.
- Raw/revoked checkpoint denial, legacy admission refusal, unchanged execution/provider counters and warm-history equality remain after capture. Matching assertions add evidence requirements rather than weakening those controls.
- The shared verified-user fixture and explicit local-owner false setting remain; no production identity policy changes are introduced.

These are source observations about authored checks. None is reported as a passing test. The artifact contains fixture setup changes, not speculative graph/kernel production changes; it does not independently establish phase completeness.

## Execution limits

Only `cat`, `sed`, `rg` and `nl` source inspection ran, with every shell call explicitly rooted at `/Users/gqadonis/.claude/worktrees/production-readiness-476`. An exploratory search included an absent optional source path; matching capture was located and read in its actual source files. No verification was executed by that search.

**Executed checks: zero.** No tests, Cargo, check/fmt/lint/typecheck, compiler/import probes, builds, API/container calls, `git diff --check`, machine screening or delegation occurred. Docker and installed-artifact gates remain closed for this reviewer. Product source, other checkouts and services were untouched. The only write is this assigned report; no unsolicited implementation or guards were added.

Compilation, matching outcomes, pending durability/resume refusal, graph capture and denial outcomes remain unverified. This report grants no runtime credit and authorizes no subsequent gate.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE for the bounded Gate015 source review. No concrete selected-behavior defect found to trigger the blocking rule.
