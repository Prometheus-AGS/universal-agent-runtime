# S1 brief — live model choice governs the next eligible response

Phase `uar-uiux-architecture-2026-11`, re-anchored value-first (`re-anchor-2026-11-19.md`).
Spec work only: no builds, tests, runs, lint or typecheck. Gherkin is authored, not executed.

## Read first

- `docs/uar-settings-user-goals/specification.md`: R-01, R-02, R-03, R-06, R-07 (lines 15-33) and V-01 (line 531), Definition of Done (line 555).
- Memory pack record 02 (requested/effective, origin, activation boundary, clear-to-inherit; receipt joins saved revision, admitted route and the response-producing model; in-flight turns stay bound; never report the requested candidate as served). The lead has quoted it in `re-anchor-2026-11-19.md`.
- Prior receipt to build on, not repeat: `.prometheus/verification/c33-rest-model-restart-macos-pass.json` (label B36, 2026-10-27, Mac, earlier build, real gateway, two models) and `scripts/behaviors/features/rest-model-restart.feature`. It proves saved default plus session override routing, restart survival and refusal of stale/invalid writes over REST. It does NOT prove the current tree, the browser journey, a live change without restart, clear-to-inherit or in-flight binding.
- Existing browser path: `tests/bdd/features/chat-model-routing.feature` and `tests/bdd/steps/*.steps.ts`, run by `pnpm test:bdd` (playwright-bdd 9.2.1, stub-llm `/_stub/requests` as the served-model truth). Reuse its steps' vocabulary.

## Deliverables (you own these files only)

1. In `openspec/changes/uiux-system-model-live-default/design.md`, a new section
   `## S1 value rules` with, for each rule: actor, condition, outcome, value to the person,
   exact example values, exclusions and unknowns. Rules to cover:
   - S1-R1 Live default: an authorized operator changes the system default A→B through the dedicated live route; confirmation shows only after the runtime acknowledges; no restart.
   - S1-R2 Next eligible turn: a conversation with no override answers its next turn on B; the transcript shows the model that actually answered, its origin (system default) and when the change took effect.
   - S1-R3 Override holds: a conversation with an explicit override C keeps answering on C after the default change; its effective value shows origin "this conversation".
   - S1-R4 Clear to inherit: clearing the override previews the inherited value (B) before confirming; the next turn answers on B.
   - S1-R5 Isolation: changing or clearing one conversation's override does not change another conversation.
   - S1-R6 In-flight binding: a turn already generating when the default changes finishes on the model it started with.
   - S1-R7 Truthful failure: an unavailable or disabled choice is refused with a usable reason; the previous effective value stays and is shown; nothing reports the requested model as served.
   - S1-R8 Agent pin: an agent with an explicit model keeps it (operator decision: agent wins) and is listed as pinned when the default changes.
   Identity: S1 uses the deployment's operator authority only. No My Defaults (U scope) in S1. Tests never mint or send credentials (external identity proxy).
2. In the same design.md, `## S1 scenario set`: map each rule to the scenarios that prove it.
   Reuse existing SYS scenario IDs where they already prove a rule (e.g. SYS-001, SYS-003, SYS-006, SYS-007, SYS-010) and add new IDs for gaps (override holds, clear to inherit with preview, isolation, in-flight binding, served-model receipt in the UI). Mark which are mandatory for S1. Keep the set as small as proves the rules. Each new row needs the same matrix fields as the existing rows, with oracles that read the stub request log or a store read-back, never the system under test, and rejecting negative controls; record the expected pre-phase verdict (baseline f2c9e36e) for each.
   List the supporting scenarios from other changes S1 truly depends on (for example the settings API rejecting `llm.default_provider`, one provider-qualified Model identity, the picker), by ID, without editing those changes.
3. Author `tests/bdd/features/live-model-choice.feature`: readable Gherkin for S1-R1..R8 in the voice of the person doing the job (operator, chat user), reusing the vocabulary of `chat-model-routing.feature`. Synthetic data only (stub providers and models). One Feature, a short Background, scenarios named after the job. Do not write or edit step definitions now; list the new step phrases you need in design.md under `## S1 step phrases` so the execute stage can implement them.

Run `validate --strict` on the change after editing. Reply under 250 words: rules written, scenario IDs (new and reused, mandatory flags), supporting dependency IDs, the feature file path and scenario count, and anything UNVERIFIED.
