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

## 2. Make the matrix a mandatory local gate

- [x] 2.1 Require this exact command locally before each change is considered
      complete and before its commit is pushed:
      `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked
      --no-default-features --features server-full --test integration
      live::capability_cases -- --test-threads=1`
      `--test-threads=1` is mandatory: every booting case is `#[serial]` and
      `harness.rs:283-289` records 7/16 failures under concurrency.
- [x] 2.2 Do not add this gate to GitHub Actions. Repository policy reserves
      Actions for deployment and deployment validation; all development checks
      in this set run locally.
- [x] 2.3 Runtime budget: the matrix took 194.70s locally plus build. Run it
      serially without sharding because sharding breaks `#[serial]`.

## 3. Prove the gate fails (not just that it passes)

- [x] 3.1 Introduce a deliberate one-line break in a **named** case.
- [x] 3.2 Run the pinned command locally. Confirm it exits non-zero **and the
      output names that specific case**. A compile error proves nothing about
      the gate.
- [x] 3.3 Revert the break. Confirm the pinned local command is green. Record
      **both** command results — the red and the green — in
      `.kbd-orchestrator/phases/uar-spec-conformance-2026-08/verification.md`,
      one row each, in the format defined by `EXECUTION-CONTRACT.md`:
      `| case | capability | evidence level | result | evidence | timestamp |`.
      Two rows, because a gate observed only passing is indistinguishable from a
      gate that cannot fail.

## 4. Verification

- [x] 4.1 Local: matrix 20/20 with the exact command in 2.1. The final
      unchanged-source retry passed all 20 cases in 196.61s.
- [x] 4.2 Local gate: one red result naming the case and one green result, both
      recorded as evidence.
- [x] 4.3 No runtime source changes in this change and no non-deployment GitHub
      Actions workflow is added or used.
