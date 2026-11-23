# S1 revision brief — round 1

Source: `review/s1/findings-round1.json` (BLOCK 7/9/1). Spec and Gherkin authoring only; no runs.
Lead verified C5 (legacy button `providers-page.tsx:408-410`; fixture provider default
`tests/bdd/support/api.ts:115` is already the target) and C4 (pre-phase stub panics on unknown
fixture kinds, `src/bin/stub-llm.rs` parse with `panic!`).

## Writer: uiux-system-model-live-default + the two S1 feature files

C1 Receipt time can fail. The step captures a time window around the confirm action
(before click, after the live route's acknowledgement) and asserts the receipt's change time
lies inside it and is not the Background's change time. Scenario 6 (in-flight) must assert
the receipt shows A's change time (from the Background), not B's. Freeze the comparator
(window, not minutes) in `s1-oracle.json`.

C2 Served differs from requested. Add a stub fixture where the provider's response reports
a different served model id than the request (e.g. request `claude-5-haiku`, response model
`claude-5-haiku-20261101`). Add a scenario: the receipt names the served id reported by the
provider and never the requested candidate as served. SYS-015's negative control becomes
this scenario run against a build that labels the receipt from the request (state how that
mutant is produced), not a copy of SYS-016.

C3 In-flight without a race. Replace the fixed delay with a held reply: the stub holds the
response until released (e.g. a hold fixture plus a release call). The steps check the reply
is still held after the default change is acknowledged, then release it. Name the stub
extension as an implementation task.

C4 Baseline expectations that a correct runner can meet. The operator-change step drives
the new system-default picker; on the baseline that control does not exist, so every
scenario using that step is NOT_RUNNABLE there, including SYS-013 (change its expectation).
Baseline runs use only fixture kinds the baseline stub can parse; scenarios needing new
kinds are NOT_RUNNABLE on the baseline. Update `baseline-verdicts.json`.

C5 S1-R1 cannot be satisfied by the legacy button. Choose a target whose provider's
`default_model` differs from the target model (e.g. set bdd-provider-b's default_model to
another model in the fixture), so only the new picker path (save model, then live route)
produces the expected served model. State whether the per-provider button stays (it sets a
provider default, a legitimate path) and make sure no S1 oracle accepts it as S1-R1.

W fixes in the same pass:
- Tag every sent message with a unique marker and assert on the stub request carrying that marker, plus a successful turn (not just the last logged request).
- Scenario 7 (refusal): assert no stub request used the refused model and the stored default is unchanged; set the YAML fallback model (`UAR_LLM__MODEL`) to a sentinel outside the S1 set so a persisted-but-refused default cannot hide behind it.
- Scenario 5 must match S1-R5 and the frozen SYS-013 fixture.
- SYS-003: assert the exact pinned set from the fixture, not membership.
- Add assertions for "confirmation only after acknowledgement", the requested value, and the activation label.
- Provider truth: give each synthetic provider its own stub instance or port so the request log identifies the provider.
- Align control 012's "unavailable" preview with what the runtime would actually serve.

## Lead / contract (after the writer)

- C6 baseline-check-supporting compares each report with the expectations file (jq), mismatch ⇒ runner rejected.
- C7 samples are collected: each scenario command uses `--repeat-each=<samples>` and `--retries=0`; all samples must pass; task 8.17 aligned (no pass-on-retry).
- Baseline worktree: add `git submodule update --init --recursive` before builds.
- Remove the external identity proxy claim for BDD runs; state that BDD runs use the deployment's auth-off test configuration and send no credentials.
