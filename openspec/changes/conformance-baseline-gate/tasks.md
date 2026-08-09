## 1. Correct the miscalibrated assertions

- [x] 1.1 `capability_cases.rs:356` — `l3_c04_credentials_listing` asserts
      `status == 200` against an endpoint that guards unconditionally. Either
      mint a Bearer token (the harness exposes `HARNESS_JWT_SECRET` at
      `harness.rs:50` for exactly this) and assert 200, **or** assert 401 as the
      documented contract. Prefer the token: an authenticated 200 exercises more
      of the capability than an unauthenticated 401. Note that the harness config
      sets `jwt_required: false` (`harness.rs:187`), so the 401 originates in the
      credentials handler itself, not in global auth — confirm which before
      choosing.
- [x] 1.2 `capability_cases.rs:511` — `shape_only_c13_sessions` asserts
      `status == 200` against a deliberately retired route. Assert the retirement
      contract instead: status 404 and body `code == "legacy_route_disabled"`.
      Rename to `absent_c13_sessions_retired` so the prefix matches what it now
      asserts.
- [x] 1.3 Each corrected assertion's failure message names the contract it
      asserts, so a future failure reads as a contract change rather than a
      mystery.
- [x] 1.3b **Discriminator.** Both corrected cases must assert a discriminator
      proving the real handler answered, not merely a status code. For C-13 that
      is the body field `code == "legacy_route_disabled"` — a bare 404 is
      indistinguishable from the `/api/{*path}` catch-all, which returns
      `code: "api_route_not_found"`. This requirement binds every case in
      `spec-conformance-measurement`, including these corrections; see
      `EXECUTION-CONTRACT.md`.
- [x] 1.4 Full matrix passes 20/20.

## 2. Make the matrix a blocking CI gate

- [x] 2.1 Add a dedicated job that runs:
      `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked
      --no-default-features --features server-full --test integration
      live::capability_cases -- --test-threads=1`
      `--test-threads=1` is mandatory: every booting case is `#[serial]` and
      `harness.rs:283-289` records 7/16 failures under concurrency.
- [x] 2.2 The job carries **no** `continue-on-error`. Do not add it to
      `live-integration.yml`, which still has two `continue-on-error: true`
      steps at `:121` and `:127` — a new file or a clearly separate job.
- [x] 2.3 Runtime budget: the matrix took 194.70s locally plus build. Confirm
      the job completes inside the workflow timeout; if it does not, raise the
      timeout rather than sharding, because sharding breaks `#[serial]`.

## 3. Prove the gate fails (not just that it passes)

- [x] 3.1 Introduce a deliberate one-line break in a **named** case.
- [x] 3.2 Push. Confirm the job goes red **and the log names that specific
      case**. A job that reddens on a compile error proves nothing about the
      gate.
- [ ] 3.3 Revert the break. Confirm green. Record **both** run URLs — the red
      and the green — in
      `.kbd-orchestrator/phases/uar-spec-conformance-2026-08/verification.md`,
      one row each, in the format defined by `EXECUTION-CONTRACT.md`:
      `| case | capability | evidence level | result | run URL | timestamp |`.
      Two rows, because a gate observed only passing is indistinguishable from a
      gate that cannot fail.

## 4. Verification

- [ ] 4.1 Local: matrix 20/20 with the exact command in 2.1.
- [ ] 4.2 CI: one red run (named case) and one green run, both linked.
- [ ] 4.3 `git diff` touches only `capability_cases.rs` and workflow files. No
      runtime source changes in this change.
