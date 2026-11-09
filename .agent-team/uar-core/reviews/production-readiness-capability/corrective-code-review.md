# Corrective capability-dispatch source review

Phase: `production-readiness-476-capability-dispatch`, Execute. Scope: corrective artifact-only inspection against source base `629f5702`, including `src/llm/inline_image.rs`, the OpenSpec deltas and directly related callers. This is one fresh-context native same-family review, weaker isolation than an independent-family review. Prior `code-review.md` and `artifact-critic.md` were read as review artifacts only. Producer handoffs, generation history and self-assessments were not consulted.

## Findings

No remaining actionable Rust defect was identified with sufficient confidence in the inspected corrective source. This is source approval only; compilation and runtime behavior remain **NOT_EXERCISED**.

## Corrective dispositions

- **R1 resolved in source:** `src/llm/mod.rs:243` declares `formatter`, and line 263 now calls `formatter.debug_struct`. The prior unbound `f` expression is gone. The payload-free Debug projection remains. This is lexical evidence, not an observed compiler result.
- **R2 resolved in source:** `src/llm/router.rs:309–317` excludes catalog supplementation whenever the exact model identity has a runtime registration, including a disabled, insufficient-context or context-unknown registration. Line 319 also excludes unregistered catalog supplementation for a requested context minimum. Exact model/provider preference is considered only after this candidate construction at lines 338–357. Final destination admission still checks live registered context at lines 441–446. The authored 4096-versus-8192 regression scenario uses an actual embedded catalog entry and asserts no route through ordinary or preferred selection; it was not executed.
- **Output-limit repair preserves the private contract:** `AdmittedDestination` and its fields remain crate-private (`src/llm/router.rs:87`). `restrict_output` at lines 95–106 lowers `output_token_cap` using `min`, and lowers an existing budget contract's output allowance using `min`; it neither constructs a contract nor raises a limit. Ordinary retry/fallback admission and the graph driver invoke it after destination preparation and before the leaf stream. The corrective routing change does not widen the public `RouteRequirements` API or inject a synthetic accounting contract.
- **R3 inspected:** the focused helper registers full fetch/body-consumption promises, uses a shared 100-second case deadline, aborts and joins owned client work, and refuses later cases after uncertainty. Cleanup has a separate 100-second lifetime and 180-second outer hook. Client settlement does not prove server-side cancellation. See the scope update below for the concurrent durable-record correction, which is outside this report's inspected snapshot.
- **R4 inspected:** `scripts/behaviors/steps/capability-dispatch.steps.mjs:233–276` requires an actual `uar.request-budget.v1` final-provider receipt, approximate counting, no fit guarantee, no installed exact reserve, enforced output ceiling, positive counts/bytes and a digest. Removing final receipts while retaining attempt manifests must fail the same comparator. The destination comparison matches this fixture's explicit-base-URL raw wire model: `registry.rs:626–638` supplies that driver model and `liter_driver.rs:838` records it. None of these assertions were executed.
- **R5 / peer compatibility inspected:** `safe_chat_observation` maps absent/null tools to an empty observational list without changing the original request. Existing `success` and `provider_failure` response branches remain outside tool advertisement processing. The supplementary Python probe authors both null-tools outcomes, ordered image hashes, safe observation and oversized-arrival accounting; the supplementary JavaScript probe authors the exact helper's real deadline and poisoned-next-case behavior. Both remain **NOT_EXERCISED**.

## Preserved safeguards and uncomfortable limits

The inspected changes retain #388 destination-bound preparations, final-wire enforcement and existing output/reasoning handling. The graph wrapper retains its captured-contract equality check before installing the existing contract. Native-adapter construction and inherited binding paths remain present. Ordinary typed-image adapter admission is derived from captured driver facts, not model catalog vision flags. Canonical message projection preserves supported inline image content while generic state and tool arguments retain secret projection; checkpoint owner, binding revalidation and protected envelope persistence remain in the host path. These are inspected mechanisms, not regression acceptance.

All runtime/provider/decoder behavior, actual zero-send refusals, retry/fallback and graph dispatch, protected-history/checkpoint continuation, finite-accounting behavior, warning-free compilation and Linux amd64 artifact delivery remain unverified. Deterministic byte capture cannot establish genuine visual understanding. No runtime or release certification is granted.

The lead stopped further review because a concurrent corrective JavaScript report identified a durable ownership-record gap and its author began a final fixture edit. This report does **not** review or accept that later edit. The Rust source was reported unchanged since inventory 002; the inspected working tree was not independently frozen or hashed by this reviewer. A renewed inventory must identify the final candidate. This source review neither opens a gate nor replaces the lead's authorization reconciliation.

## Commands and ownership

Every shell command used explicit cwd `/Users/gqadonis/.claude/worktrees/production-readiness-476`. Source inspection used `pwd`, `cat`, `rg`, `rg --files`, `sed`, `nl`, `head`, `tail`, and ordinary `git diff`/`git diff --stat`/`git diff --name-only`, including staged and unstaged diff inspection. Outputs supplied the cited source and diffs. A few candidate-path reads returned file-not-found errors; no executable was launched through them. The older Compass snapshot described in the repository map was not treated as current evidence or rebuilt.

No tests, checks, lint, format, typecheck, Cargo, compiler, builds, imports, Docker, network/service or acceptance commands were run. Rust skill verification suggestions were deferred under the task's source-only restriction. No production guard or unsolicited implementation was added. Only this requested review file was written; all other writers' edits were preserved.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE — inspected Rust corrective source only. The later fixture edit and every execution claim remain outside this approval.
