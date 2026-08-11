# Reflection — uar-spec-conformance-2026-08

Written 2026-08-11 at phase close. Leads with the delta, not the result.

## The delta: the executor corrected the spec twice, and both corrections mattered

The plan I authored specified tests that could not be written as described. Codex
found this during execution and refused to fake them.

**Correction 1 — C-25/C-26/C-27.** I wrote task 3.2 as *"C-25 node DID — target
L3, `did:key` derivation is deterministic and offline, assert against the
published W3C vector used by `frf-did`."* That is true of the `frf-did` crate.
It is not reachable from UAR, because **UAR has no `frf-did` or `frf-wallet`
dependency.** I specified tests against crates the runtime does not consume.

**Correction 2 — C-21.** I specified *"boot with two tenants; tenant A writes;
tenant B reads the same resource; assert B is denied."* There is no cross-tenant
surface in the runtime to target. The test as written cannot exist.

Both became `excluded_` cases with the blocking reason named — and, importantly,
written so they **fail the moment the gap closes**. An exclusion that
self-invalidates cannot silently outlive the condition that justified it. That
is better than what I specified.

**Root cause.** I wrote the spec from the capability list rather than from the
call graph. `frf-did` exists and I built it four days earlier, so "C-25 is
testable" felt true. It was true of the crate and false of the runtime. The
distinction only appears if you check what UAR actually links.

**Corrective action.** The same error appeared twice more in this phase and was
caught by review rather than by me:

| Occurrence | How it surfaced |
|---|---|
| `stub_llm.rs` vs `wiremock`/`httpmock` | critic W-1 forced a registry check |
| "no shutdown hook exists" (OQ-1) | reading `server.rs` refuted it outright |
| C-25/26/27 and C-21 targets | executor hit them during implementation |

All three ran the same direction: **assuming absent infrastructure that already
existed, or assuming present infrastructure that did not.** The generalisable
rule, now recorded: *ground every change in a file and line before writing its
exit criteria.* Analysis in this phase already carries that note; it belongs in
the next phase's spec discipline too.

## Second delta: my reporting was wrong twice during verification

I read a partial log and reported "1 passed, 90 filtered out" as the matrix
result. It was `harness::tests::process_server_helper` running individually; the
matrix was still executing. Earlier the same day I reported a killed background
job (exit 144) as a compile failure when it was a SIGTERM with zero errors.

Both are the same defect as the spec errors above — reading a proxy and treating
it as the thing. The correction that works is mechanical: **wait on the process,
not on a byte count, and read the log rather than the wrapper's exit code.**

## What the phase produced

Independently re-run on `main` at `38d41a42`, not read from the executor's
artifacts:

```
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured;
62 filtered out; finished in 264.16s
```

| Measure | Value |
|---|---|
| Capabilities covered | **27 / 27**, none missing |
| Evidence | 1 L4 · 14 L3 · 4 L2 · 1 shape-only · 3 absent · 6 excluded |
| Tasks | 44 / 44 across three changes |
| Runtime change | one additive `Option<CancellationToken>` on `start_server_sidecar` |

The L4 round-trip is genuinely cold: it writes through the public API, shuts down
via the real graceful-shutdown path, boots a **separate process** on the same
SurrealKV path, and reads back. Its negative control is an env switch that
repoints the second boot, so the test can be *demonstrated capable of failing*
rather than assumed to be.

## What the phase did not produce, stated plainly

The strongest claim this supports is: *on `server-full`, against a recorded
in-process stub, in a harness-created database, each capability returns a
correctly-shaped response, and one survives a cold restart.*

Three of the four structural limits from the original adversarial review are
untouched and were never in scope: **no semantics** (shape, not content), **no
real-provider behaviour**, **one profile**. The fourth (no L4) is closed for
C-12 only.

`goals.md` forbade publishing an aggregate percentage or a runtime-level verdict.
That constraint held. `verification.md` carries per-capability limits — *"retrieval
relevance is not claimed"*, *"no transfer to embedded-mobile"* — rather than a
headline number.

## Scope changes that did not go through review

Two, both defensible, neither adversarially vetted:

1. **Six exclusions where the reviewed spec sanctioned one** (C-24). The added
   five are C-21, C-25, C-26, C-27 and a durability exclusion for C-13.
2. **The CI requirement in the merged spec is broader than the operator
   decision.** The decision was temporal — defer until the code base works. The
   merged text is categorical: *"GitHub Actions MUST NOT run the matrix or other
   unit, integration, conformance, lint, format, or routine development
   checks."* That is a standing repo-wide rule arriving through a
   measurement-phase spec delta, and it implies the existing `CI`, `Coverage`,
   `BDD Chat Scenario Suite`, and `Cookbook examples` workflows should be
   retired rather than repaired. **Open for the operator.**

## Carried forward

- **GAP-05 wording is overstated in `SPECIFICATION.md`.** Judge ruling
  2026-08-09: the registry is empty of *built-ins* always, empty *overall* only
  on a fresh device, because `SkillService::initialize` loads persisted skills
  through `DatabaseStorageProvider`. The spec's "empty skill registry" and
  "capability at 0%" both overstate the code. Amend deliberately, not
  mid-measurement.
- **The six exclusions are the next phase's work list.** Each collapses into a
  real result when its blocking condition clears — which makes the matrix a
  progress instrument, not just a snapshot.
