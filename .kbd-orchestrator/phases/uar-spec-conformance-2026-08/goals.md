# Goals

Phase: **uar-spec-conformance-2026-08**
Opened 2026-08-09. Predecessor context: `uar-uiux-full-migration-2026-08`
(running, 47/72) is a UI phase and does not measure the runtime against its spec.

## The goal

Produce a per-capability conformance result for every capability in
`docs/SPECIFICATION.md` (C-01..C-27) that is:

- **(a) current** — measured against `main` as it stands, not inherited from a
  run that predates the commits since;
- **(b) reproducible** — a named command, a pinned backend mode, and a recorded
  profile, such that a second person gets the same numbers;
- **(c) honestly labelled** — each result carries an evidence level that reflects
  what was actually exercised, and capabilities that cannot be exercised are
  **published exclusions**, never silent passes;
- **(d) enforced** — the matrix is a mandatory local completion gate, with a
  demonstrated red path and durable evidence for each completed change.

Published exclusions are in scope for (a)-(d). An honest "cannot be measured with
this harness" is a successful output of this phase. A pass count is not the
deliverable; a truthful matrix is.

## Explicit non-goal

**This phase does not claim, and cannot support, that the runtime is done.**

Even at 27/27 the strongest defensible claim is: *on `server-full`, against a
stub LLM, in a throwaway database, each capability returns a correctly-shaped
response.* Four structural limits stand outside this phase's scope and are not
closed by coverage:

| Limit | Closed here? |
|---|---|
| No L4 (write→reboot→read) | partially — C-05 only |
| No semantics (shape, not content) | no |
| One profile (`server-full` only) | no |
| No real-provider behaviour | no |

The phase must publish the per-capability matrix with evidence levels as its
top-level output. **It must not publish an aggregate percentage or a
runtime-level verdict.** A stakeholder reading "27/27 conformance" will hear "the
runtime is done," and an earlier method was killed by adversarial review for
exactly that failure.

## Why this phase exists now

Measured 2026-08-09:

- `docs/SPECIFICATION.md`: 718 lines, 27 capabilities, 39 GAP references.
- `capability_cases.rs`: 20 tests covering **19 of 27** capabilities.
- Durable local completion records for that matrix: **zero**.
- Baseline run: 18 passed, 2 failed — and **both failures are miscalibrated
  assertions, not runtime defects** (see `baseline-2026-08-09.md`).

The instrument exists and works, but its error rate currently exceeds the
runtime's and no durable local completion record proves its failure path. That
is the gap this phase closes.

## Success criteria

1. A committed baseline naming its command, backend mode, and profile. **[MET
   2026-08-09]**
2. The two miscalibrated assertions corrected, so the instrument stops
   generating false defects.
3. A mandatory local gate whose non-zero failure is proven by a named case.
4. All 27 capabilities carry either a result at a stated target evidence level,
   or a published exclusion with a reason.
5. The evidence-label taxonomy defined and applied consistently across all cases.

## Ownership

C-01, C-01b and C-02 are executable in this session — one command and two
one-line test corrections. No runtime changes or GitHub Actions work.

**C-03, C-04 and all of C-05 hand off to Codex at kbd-execute.** They are
bounded, test-shaped, multi-file work.

> **Corrected 2026-08-09.** This section previously held C-05(a) back, claiming
> "a shutdown hook on `start_server` is a boot-path change." Reading
> `src/server.rs` refuted that: graceful shutdown already exists — a
> `CancellationToken` at `1386`, a signal handler at `1388-1420`, a
> `shutdown_future` at `1425-1438`, and both listeners wired through
> `.with_graceful_shutdown(...)` at `1441`/`1453`. The token is merely created
> internally, so the seam is a caller-supplied parameter on a function
> (`start_server_sidecar`, `1357`) that already accepts a caller-supplied
> `oneshot::Sender`. Additive, not a redesign.

Codex deliverables are re-reviewed against the four structural findings before
this phase closes. This phase exists because a prior instrument silently
overstated its own evidence; handing the fix off without a review gate would
reproduce the enabling condition.
