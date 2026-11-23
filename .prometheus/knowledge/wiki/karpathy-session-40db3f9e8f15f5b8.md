---
type: SessionRecord
id: karpathy-session-40db3f9e8f15f5b8
title: Karpathy session 40db3f9e8f15
tags:
- karpathy
- session-learning
sources:
- resource: session:a1da1304-ba13-46ed-9018-32c6d7cfa804
generated:
  by: pk/1.11.0
  at: 2026-11-20T01:49:38.523986+00:00
created_at: 2026-11-20T01:49:38.523986+00:00
updated_at: 2026-11-20T01:49:38.523986+00:00
revision: 0
---

## Delta

S1 is specified, but the contract isn't adopted yet. The review hit its two-round cap with three critical findings left in how pass and fail are judged. Under VLBV rule CR-30 those can't be accepted at the cap, so the next step is your call.

**What S1 covers**
- **Nine rules, R1 to R9.** An operator changes the system default live, through the new picker, and the next reply answers on the new model. A conversation override holds, and clearing it shows a preview before inheriting. Other chats are unaffected, and a reply already being written finishes on its original model. The receipt names the model the provider reported, where the choice came from, and when it changed. A refused choice never claims a served model, and an agent with its own model keeps it and is listed.
- **Readable Gherkin:**
  - `tests/bdd/features/live-model-choice.feature` has 10 scenarios.
  - `tests/bdd/features/live-model-choice-controls.feature` has 9 negative controls.
- **Contract:** 13 mandatory scenarios run through the existing `pnpm test:bdd` path. The stub's request log is the evidence of which model answered. Each scenario runs its declared samples with no retries, and the run is compared against expected pre-phase results. The 89-scenario draft is saved as `adopted-contract.full-draft-r3.json`.
- **Earlier evidence:** the C33/B36 REST receipt from 2026-10-27 is recorded as prior evidence, not as proof for the current tree.

**Review:** round 1 found 7 critical problems. Round 2 confirmed 5 of them fixed: the receipt-time window, the held reply for the in-flight check, the pre-phase expectations, excluding the old per-provider button, and real sample counts. Three remain:
1. **The deliberately broken build can "fail" for the wrong reason.** It's the only control proving the receipt names what the provider reported. It's built from committed code, so it would lack this phase's uncommitted work. It names no exact patch, and any failure at all counts as the expected rejection.
2. **The pre-phase comparison for the three supporting scenarios also counts their negative-control tests.** So an empty main test can still match the expected FAIL.
3. **"Confirmation appears only after the runtime acknowledges" has no real check.** A console that confirms on click would pass.

Each fix is narrow and stays in the spec. The record is in [spec-review.md](.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/spec-review.md), and the runtime's next work is updated to match.

**Still open:**
- **Your decisions:** the 20 items in `spec-open-decisions.md`, the navigation layout, and the identity mode for My Defaults.
- **Not committed:** nothing is committed, built or run.

Should I run one more narrow round to fix these three, or show you the contract first?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: a1da1304-ba13-46ed-9018-32c6d7cfa804
- Captured: 2026-11-20T01:49:23.467212Z
- Project: /Volumes/T7/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
