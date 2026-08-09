# Execution contract — conformance change set

**Read this before executing any of the three changes.** It resolves the
cross-change questions an autonomous executor cannot safely infer. Applied after
adversarial review (MiniMax-M3 critic, Kimi k3 judge) returned INSUFFICIENT on a
set that was individually valid but ambiguous when read together.

## The set, and its order

Apply strictly in this order. Each change assumes its predecessors have landed.

1. **`conformance-baseline-gate`** — correct two miscalibrated assertions, make
   the pinned matrix a mandatory local gate, prove the command detects a named
   failure.
2. **`conformance-close-capability-gaps`** — define the label taxonomy, relabel,
   close the eight-capability hole.
3. **`conformance-l4-persistence`** — expose the shutdown trigger, allow a fixed
   DB path, write the round-trips.

All three add requirements to the same capability, `spec-conformance-measurement`.
Each delta applies on top of the previous change's result. **On conflicting
requirement text, the later change wins.**

Do not parallelise. 1 and 2 both edit `capability_cases.rs`; 2 and 3 both edit it
again. Running them concurrently produces conflicts with no stated resolution.

## The pinned verification command

Every change uses this command verbatim. It is quoted here so no change has to
resolve it from another change's task file.

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases -- --test-threads=1
```

Three parts are load-bearing and must not be altered:

- `UAR_LIVE_INTEGRATION_BACKEND=recorded` — the in-process stub. No API keys, no
  token spend. Results differ under `live`, so a result taken in another mode is
  not comparable to the baseline.
- `--features server-full` — the certified profile. Results transfer to no other
  profile.
- `--test-threads=1` — every booting case is `#[serial]`;
  `harness.rs:283-289` records 7/16 failures under concurrency.

Baseline for comparison: `.kbd-orchestrator/phases/uar-spec-conformance-2026-08/baseline-2026-08-09.md`
— 18 passed, 2 failed, 194.70s.

## Local enforcement covers the whole set

The gate defined by `conformance-baseline-gate` task 2.1 runs the entire
`live::capability_cases` module locally before each change is considered
complete and before its commit is pushed. **Every case added or renamed by
changes 2 and 3 is therefore executed and enforced by that gate** — neither
change may add a case that the pinned command skips.

GitHub Actions are reserved for deployment and deployment validation. Unit,
integration, conformance, lint, format, and other routine development checks
MUST NOT be added to or run by a GitHub Actions workflow.

If a case cannot run under the pinned local command, it is an `excluded_` case
with the reason named. It is never a case that silently does not execute.

## Discriminator scope

The discriminator requirement (`conformance-close-capability-gaps` task 3.7)
binds **every case in `spec-conformance-measurement`, including the two
corrected by `conformance-baseline-gate`.**

Concretely: the corrections in baseline-gate tasks 1.1 and 1.2 must each assert a
discriminator proving the real handler answered, not merely a status code. For
C-13 the discriminator is the body field `code == "legacy_route_disabled"` — a
bare 404 is indistinguishable from the `/api/{*path}` catch-all, which returns
`code: "api_route_not_found"`.

## What counts as satisfied

A capability is satisfied when it has **either**:

- a passing case at or above its stated target evidence level, **or**
- a recorded exclusion carrying an explicit reason.

`absent_` and `excluded_` cases are exclusions and count as satisfied **provided
they carry a reason**. `absent_c13_sessions_retired` is satisfied: its reason is
that the route was deliberately retired in favour of `X-UAR-Session-ID` with
`POST /api/chat/completion`.

A case with no reason recorded is not satisfied, regardless of prefix.

## Verification record — one file, one format

All three changes append to
`.kbd-orchestrator/phases/uar-spec-conformance-2026-08/verification.md`.

One row per case, in this format:

```
| case | capability | evidence level | result | evidence | timestamp |
```

For local-gate proof rows (baseline-gate task 3), record two rows — the
deliberate red run and the green run after revert — and name the local command
result in the evidence column so the gate is provably capable of detecting a
failure.

## Stop conditions

Halt and report rather than guessing if any of these occur:

- The pinned command's result set diverges from the baseline in a way not
  explained by the change being executed.
- A runtime change beyond the additive parameter in
  `conformance-l4-persistence` task 1.1 appears necessary.
- Any task requires editing `docs/SPECIFICATION.md`. The spec is the measuring
  stick; changing it to fit the measurement inverts the exercise.
- Any task appears to require a non-deployment GitHub Actions job. All
  development verification in this set is local.
- `cargo fmt --all -- --check` or `cargo check --all-targets` fails for a reason
  unrelated to the change in hand — that is a pre-existing break and it belongs
  in its own change.
