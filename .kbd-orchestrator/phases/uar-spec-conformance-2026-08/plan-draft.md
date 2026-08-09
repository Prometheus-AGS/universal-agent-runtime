# Plan — DRAFT, pending analyze

**Status: not the phase plan.** The KBD process stops after assessment by
operator instruction; analyze → plan is the next step and is the operator's to
run. This file records the change set that survived adversarial review so that
step starts from reviewed material rather than a blank page.

## Review record

Sent artifact-only (no generation history) to two models through the liter-llm
gateway on 2026-08-09.

- **Critic — MiniMax-M3.** 3 CRITICAL, 1 SERIOUS, ~5 MINOR.
- **Judge — Kimi k3.** Verdict: **INSUFFICIENT as written — but close. Four
  small amendments fix it.**

Upheld and applied:

| Finding | Amendment |
|---|---|
| "27 of 27 have a case" measures coverage, not conformance — 27 useless tests satisfy it | C-03 carries a per-capability target evidence level; shortfalls become published exclusions |
| Baseline did not pin the backend mode while verification did — breaks the plan's own reproducibility goal | C-01 pins `UAR_LIVE_INTEGRATION_BACKEND=recorded` and records it in the artifact |
| C-05 misclassified as "bounded, test-shaped" — a shutdown hook is boot-path work | C-05 splits into (a) runtime seam, (b) tests; only (b) hands off |
| `shape_only_` / `absent_` undefined in the relabel scheme | C-04 defines the full taxonomy before relabelling |
| "Turns the job red" is not proof — a compile error also turns it red | C-02 proof must name the failing case |
| Codex deliverables have no acceptance bar | Handoff carries a re-review gate |

Rejected by the judge, and I accept the rejection:

- *"`L4 unverifiable` is a deliverable failure."* — No. The goal is an honest
  measurement, not a pass count; an honest "unverifiable" is a success.
- *"Someone could re-add `continue-on-error` later."* — True of every CI gate;
  an argument for code review, not a plan defect.

Added after review, from the baseline the reviewers did not have:

- **C-01b.** Correct the two miscalibrated assertions. Both failures today were
  tests asserting contracts the runtime deliberately does not honour. Extending
  coverage on a miscalibrated instrument compounds the error.

## Change set

**C-01 · Baseline.** DONE 2026-08-09 — `baseline-2026-08-09.md`.

**C-01b · Correct the two miscalibrated assertions.** `l3_c04_credentials_listing`
must send a token or assert 401 as the contract; `shape_only_c13_sessions` must
assert the documented retirement (`legacy_route_disabled`) rather than 200. Exit:
20/20 pass, and each corrected case states which contract it now asserts.

**C-02 · Blocking CI gate.** A dedicated job running `capability_cases` with
`continue-on-error: false`. Not inside `live-integration.yml`, which still
carries two `continue-on-error: true` steps. Exit: a deliberately broken case
turns the job red **and the log names that specific case**.

**C-03 · Close the 8-capability hole**, with a target evidence level per
capability:

| Capability | Target | Rationale |
|---|---|---|
| C-21 tenant isolation | L3 + negative | two tenants, cross-read returns denied — not 404 |
| C-25 node DID | L3 | `did:key` derivation is offline and deterministic |
| C-26 DID resolution / VC verify | L3 | offline for `did:key` |
| C-27 wallet | L3 | forged-issuer and expiry cases must fail closed |
| C-16, C-18, C-19 | L2 minimum | raise if the surface allows |
| C-24 peer mesh | published exclusion | needs two devices; state it, do not fake it |

Exit: every capability has a result at or above its target, or a published
exclusion naming the reason. A route-not-found discriminator alone is L1 and does
not satisfy any row.

**C-04 · Define and apply the label taxonomy.** `l1_` present, `l2_` wired,
`l3_` exercised, `l4_` round-tripped, `shape_only_` shape without semantics,
`absent_` asserts absence, `excluded_` published exclusion. Then relabel: stub-
backed cases whose correctness depends on model output become `l2_`. Exit: every
case name carries a defined prefix.

**C-05 · L4 for persistence.**

> **CORRECTED 2026-08-09 after reading `src/server.rs`.** This change was
> classified as "boot-path refactor, scope separately, do not hand to Codex."
> That classification was wrong, and wrong in the direction that mattered: it
> made L4 look expensive and pushed it out of the handoff.
>
> **Graceful shutdown already exists.** `serve_on_listener` creates a
> `tokio_util::sync::CancellationToken` at `server.rs:1386`; a signal-handler
> task (1388-1420) waits on SIGINT/SIGTERM, drains the ingestion pool, then
> calls `http_shutdown.cancel()`; `shutdown_future` (1425-1438) awaits that
> token and drains in-flight connections with a timeout; both listeners are
> wired through `.with_graceful_shutdown(...)` at 1441 and 1453.
>
> Nothing needs to be built. The token is created internally and only signals
> can fire it, so what a test lacks is a way to *own* it.

- (a) *Runtime seam* — accept a caller-supplied `CancellationToken` so a test can
  trigger the shutdown path that already exists. `start_server_sidecar`
  (`server.rs:1357`) already takes a caller-supplied
  `oneshot::Sender<SocketAddr>` for readiness; a `CancellationToken` parameter
  is the same shape of change against the same function. Additive, and the
  existing signal handler keeps working unchanged.
- (b) *Harness* — `boot_test_server` must accept a fixed DB path rather than
  minting a fresh temp path per boot, so the same store can be reopened.
- (c) *Tests* — write→reboot→read for C-12 and C-13, once (a) and (b) exist.

**All three are now in scope for the Codex handoff.** The earlier split existed
because (a) was believed to be a boot-path redesign; it is a parameter addition
on a function that already takes caller-supplied channels.

Exit: C-12 and C-13 produce a real L4 result, or are published as
`L4 unverifiable` with the blocking reason named. The reason may no longer be
"no shutdown hook exists" — that is refuted.

## Sequencing

C-01 (done), C-01b, C-02 are executable without runtime changes.
C-03, C-04 and **all of C-05** hand to Codex at kbd-execute.

*Superseded:* an earlier revision held C-05(a) back for its own scoping decision,
on the belief that a shutdown hook had to be designed. Reading `server.rs`
refuted that — see the C-05 entry above. No separate scoping step is needed.

## Verification

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases -- --test-threads=1
```

Same command as the baseline — same mode, same profile, same thread count. That
identity is the reproducibility guarantee; changing any of the three invalidates
comparison against `baseline-2026-08-09.md`.

Tier 3 is out of scope and `tier-guard.sh` blocks it while status is `running`.

## The uncomfortable thing

**This phase can reach 27/27 and still not license the claim it appears to
license.** Three of the four structural limits — semantics, second profile, real
provider — are outside its scope entirely, and C-05 closes only the first.

The output must be a matrix with evidence levels, never an aggregate. If it ever
reports "27/27 conformance," it will mislead in exactly the way that got an
earlier method killed.

Second discomfort: this phase opens by running an instrument that already
existed. That is an accurate record of where the previous session's hours went.
