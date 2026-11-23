# B5 repair round 3: plan

Operator approval: "Approve round 3" (2026-11-22). Base: c8eb9e23. Status: **implementing.**
Budget amendment adopted as 03de9cf8. The operator chose rate-limit option A (burst 100).

## Rules

| Rule | Actor and condition | Outcome | Scenario |
|---|---|---|---|
| R3-1 | An operator opens Add model with the full 8,171-row catalog behind it | Opening, typing and closing the picker cause no main-thread task of 200 ms or more | PICK-002 |
| R3-2 | A person opens UAR in a new browser profile and sends a first message | History loads and the request is sent, without a reload | SYS-001 and the other chat scenarios |
| R3-3 | A credential-free loopback browser, on a test server that opts in | The browser is the local owner; tests carry no credential | The ten chat scenarios |
| R3-4 | An operator loads or reloads a console page a few times in a row | The console's own requests are not refused with 429 | SET-003 |

## Root causes and fixes

### 1. Picker freeze (R3-1). Writer: uar-console

- Cause, measured on the committed build without tracing: `models-page.tsx` mounts all 8,171 rows
  (110,723 DOM elements). Opening the picker caused tasks of 350, 384 and 428 ms; typing caused 304 ms.
  With those rows removed from the same page (577 elements), opening took 81 ms and typing caused no
  long task. The 8.5 s tasks in the boundary run were mostly trace-snapshot cost on that same DOM.
- Fix: window the catalog list in `models-page.tsx` with the installed `@tanstack/react-virtual`.
  Keep `models-count` (the filtered total), the `compare-checkbox-<key>` test ids, keyboard access
  and the visual design.
- Residual risk: an entity-persistence timer task of 150-172 ms was seen, close to the limit.

### 2a. First chat fails in a new browser (R3-2). Writer: uar-state

- Cause: `UarDb.open()` loads `pglite-seed-v3` and then skips `runMigrations`, so migration 4
  (`run_id`, `client_turn_id`, `history_position`) never runs until a reload. This is a product bug,
  already in `.prometheus/gotchas.md` (2026-11-17), and it explains the SYS-001 history error.
- Fix: in `frontend/src/platform/pglite/client.ts`, always call `runMigrations`. It already skips
  applied versions.
- Noted, not fixed: `pnpm pglite:seed:check` looks for a v4 seed that does not exist.

### 2b. No principal in the chat harness (R3-3). Writer: uar-runtime

- Cause: the BDD server does not opt in to the existing local owner.
- Fix: `UAR_SECURITY__LOCAL_LOOPBACK_OWNER: 'true'` in `tests/bdd/playwright.config.ts`. The product
  default stays off. A request with an Authorization header, even an empty one, gets no local owner.

### 3. 429 on the console's own requests (R3-4). Writer: uar-runtime. Operator decision

- Cause: one process-wide bucket with burst 20. One Providers page load makes 14 API requests within
  one second, so a second load can exhaust it. SET-003's frozen config refills at 5 requests/s.
  `example.config.yaml` ships burst 10, less than one page load.
- Option A (recommended): default burst 20 to 100 in `src/config.rs`,
  `src/uar/settings/resilience_policy.rs`, `resilience-preview.ts` and `example.config.yaml`
  (10 to 100). The sustained rate and the frozen fixture are unchanged. Trade-off: an exposed server
  admits up to 100 requests at once before throttling.
- Option B: switch the limiter off in the test-server configs only. The scenarios stop seeing 429;
  users still can.
- Noted, not fixed: the Providers page fetches `/api/uar/providers` five times per load.

## Sequence

1. Sign and adopt `amendments/repair-round-3-budget.sh`.
2. Implement 1, 2a, 2b and 3, one writer per file. No test or build during implementation.
3. Independent diff and step-definition review, then one local commit (no push).
4. Regenerate the inventory; the independent critic records PASS.
5. Follow the Repair rule in full: bind, prepare-bdd, remove and rebuild both mutant worktrees (the
   round 2 exception was for c8eb9e23 only), both baseline checks, then commands 23-49. Every earlier
   PASS is invalidated, including IDN-001, because `src/**` and `frontend/src/**` change.
6. Ask the operator before the builds whether Codex is building #539.

## Limits

- Expected boundary time is 4.5 to 6 hours, from the last run's durations.
- Nine of the ten chat scenarios have never run past setup, so more faults may appear.
- A pass would accept S1 on this branch only. It would not cover merging main, a version bump or a tag.

## Added 2026-11-22 06:03 CST after the r3 chat-path review

| Rule | Actor and condition | Outcome | Scenario |
|---|---|---|---|
| R3-5 | An operator switches a provider's default model, with or without that provider's key in the environment | The provider's verification proofs stay current, so the next turn runs on the chosen model and not on the configured fallback | SYS-001, 011, 013, 014, 017 and the others that switch |
| R3-6 | The baseline check runs on f2c9e36 | Each scenario gets through Background, so an expected FAIL is the oracle rejecting the old build | baseline-check-bdd (012 must fail at the preview step) |

- R3-5 cause A: reads add a default execution profile, the console writes it back, the digest changes.
  Fix: `prepare_provider_update` keeps absent when the write equals the default.
- R3-5 cause B: the update route enriched the key from the environment before deciding whether the
  operator sent one, so every keyless save re-stored it and rotated the credential revision. Fix: decide
  and carry the saved key over first. A keyless save no longer copies an environment key into the store.
  No scenario exercises cause B (the test server blanks the key). Check it at image qualification.
- R3-6 fix: S1 fixtures first on stub A; burst 100 stated in the BDD server environment.
- This widens round 3 beyond the three named repairs. Reason: S1-R1 and S1-R2 fail without it.
- Not changed: burst 10 in docs/config/home.uar.embedded-openai.example.yaml (its limiter is off).

## Corrected 2026-11-22 06:29 CST after the r4 review

- **R3-6 as written above is wrong and is replaced.** On f2c9e36 each scenario gets through Background and
  teardown, and then fails inside the scenario. SCN-SYS-012 fails in its second Given, because the old console
  has no session-config-model-trigger, model-option or session-config-clear. It is NOT_RUNNABLE in the
  contract's vocabulary. No S1 BDD oracle is shown rejecting the old build. The controls and the two mutants
  test discrimination on the candidate only; none runs on the old build, and 012 has no mutant. The combined
  amendment (not yet written) will propose that the owner change the 012 expectation from FAIL to
  NOT_RUNNABLE. Until signed, the frozen file is untouched. All of this is predicted from source: the first
  r5 baseline receipt must show no teardown 404, no 429 and no probe error.
- Teardown: dispose tolerates a 404 from /_stub/release; the scenario step stays strict.
- R3-5 cause B, corrected: without a protected store, r4 would have written an environment key to the settings
  row in plaintext, where the old code returned 503. In r5 a keyless update does not persist the key taken from the
  provider's catalog environment variable, in either mode. A saved key still carries over. Keys from
  UAR_LLM__API_KEY or api_key_env, and boot seeding, can still reach a store-less row (predates round 3).
- Limit of R3-4: native installs keep their existing config.yaml, so an upgraded install keeps burst 20 until
  the operator edits resilience.burst_size. For the release notes.
- Recorded risk: SCN-SYS-014 depends on a 15 s stream-start timeout.
