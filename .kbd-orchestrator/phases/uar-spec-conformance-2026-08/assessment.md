# Assessment

Phase: **uar-spec-conformance-2026-08**. Written 2026-08-09, before planning.
Every figure below was measured this session; none is carried forward.

## What exists

| Artifact | State |
|---|---|
| `docs/SPECIFICATION.md` | 718 lines, 27 capabilities C-01..C-27, 39 GAP references |
| `tests/integration/live/capability_cases.rs` | 20 test functions, 19 capabilities |
| `tests/integration/live/harness.rs` | boots a real server; `#[serial]`; ~10s cold boot |
| `tests/integration/live/stub_llm.rs` | in-process OpenAI-compatible stub; no keys, no spend |
| Live tier compile | `cargo check --features server-full --test integration` → exit 0, 1m54s |
| Baseline run | 18 passed / 2 failed / 194.70s — see `baseline-2026-08-09.md` |

The instrument is intact and runnable. This is not a greenfield problem.

## Gap 1 — 8 of 27 capabilities have no test

**C-16, C-18, C-19, C-21, C-24, C-25, C-26, C-27.**

Not a random tail:

- **C-21** is tenant isolation — a security property. Absent.
- **C-25, C-26, C-27** are the DID / VC / wallet capabilities. The `frf-did` and
  `frf-wallet` crates were built 2026-08-07 with 37 unit tests between them and
  have **never been exercised through the runtime**.
- **C-24** is the peer mesh.

The newest and most security-sensitive capabilities are precisely the ones with
no instrument.

## Gap 2 — the matrix does not run in CI

```
$ grep -rln 'capability_cases\|live::capability' .github/workflows/
(no matches)
```

`live-integration.yml` was hardened: line 75 reads *"No continue-on-error. This
is the gate."* But that gate is a **compile** gate, two later steps remain
`continue-on-error: true`, and nothing executes the matrix.

This is the 25-day harness failure one level up. That failure was "the tier did
not compile and CI showed green." The current state is "the tier compiles and CI
never runs it." Both produce a green checkmark that carries no information about
whether the runtime works.

## Gap 3 — the instrument's error rate exceeds the runtime's

Two consecutive runs:

| Run | Reported failures | Actual runtime defects |
|---|---|---|
| earlier (18 cases) | 5 | 3 |
| 2026-08-09 (20 cases) | 2 | **0** |

Today both failures were tests asserting contracts the runtime deliberately does
not honour: C-04's credentials endpoint guards unconditionally (correct), and
C-13's session route was retired on purpose and says so in its error body.

**Consequence for planning:** adding 8 cases on top of a miscalibrated instrument
compounds the problem. The two known-wrong assertions must be corrected before
coverage is extended.

## Gap 4 — evidence labels overstate what was exercised

Current distribution: 13 `l3_`, 2 `l2_`, 3 `shape_only_`, 2 `absent_`.

Against a stub whose fixtures the test author wrote, "did my code parse my own
canned output" is L2-wired, not L3-exercised. That applies to every capability
whose correctness depends on model output — C-03, C-05, C-08 at minimum.

`shape_only_` and `absent_` are also undefined relative to the L-scale. The
taxonomy is not closed, so "honestly labelled" cannot currently be verified.

## Gap 5 — four structural limits, unchanged since adversarial review

Ruled by MiniMax-M3 (critic) and Kimi k3 (judge), artifact-only isolation. Their
verdict on the instrument: **a smoke matrix, not a doneness measurement.**

1. **No L4.** Fresh temp SurrealKV path per boot, no shutdown hook on
   `start_server`. Write→reboot→read is inexpressible. C-12 and C-13 are exactly
   where L3 without L4 is nearly worthless.
2. **No semantics.** Shape assertions only. C-03 returning the wrong model
   passes.
3. **One profile.** `server-full`. Silent on `embedded-mobile`, where GAP-04
   says the Rust library API *is* the contract.
4. **No tenant isolation.** C-21 needs two tenants and a cross-read attempt;
   `#[serial]` single-tenant cases cannot express it.

## What the current evidence supports

*On `server-full`, against a stub LLM, in a throwaway database: 18 of 19
exercised capabilities return correctly-shaped responses; 8 capabilities are
unmeasured; the matrix does not run in CI; and the two reported failures are
instrument defects rather than runtime defects.*

That is the whole claim. It is not a doneness measurement.

## Risk if this phase is skipped

The runtime is at `v1.0.0` — tagged and published 2026-07-11, `main` now +367
commits — with **no current per-capability evidence of what it does**. The
`decisions.md` entry of 2026-08-09 records the related finding that four
certification changes remain PENDING and no supply-chain artifacts exist on disk.

An estate whose commercial position is evidence discipline is currently unable to
answer "does the runtime do what its specification says" for any single
capability. That is the exposure.
