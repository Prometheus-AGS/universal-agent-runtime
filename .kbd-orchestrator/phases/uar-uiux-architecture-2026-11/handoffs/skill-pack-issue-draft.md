# Draft issue for Prometheus-AGS/prometheus-skill-system

**Title:** KBD: a contract-bound phase can't take an operator-decided exit, and a read-only probe can lock a replica out of writes

---

## Summary

The operator decided how phase `uar-uiux-architecture-2026-11` (project UAR) should exit, and the release shipped. KBD still could not close the phase.

The runtime and pack give no supported way to record an exit that the operator decided instead of the adopted boundary packet. The only path is to rewrite and re-sign the contract's scenario set so the evaluator sees PASS. While doing that, five more defects each cost a signature or a round trip. The last one blocked every KBD write from any process on the checkout.

**Versions:**
- `prometheus` 1.14.0
- skill pack 1.15.0, generation `ff85887a…`
- macOS, with the checkout on an external APFS volume (`/Volumes/T7`)

**Related:** #184 (bounded phase exit) and #192 (divergent phase-close paths; unauthorized scope must not block termination). Both are closed, and this case is not covered by either.

## Scenario

- **Who:** the operator of a contract-bound phase, and an agent closing it out.
- **When:** the release shipped, and the operator explicitly replaced the adopted boundary packet with a smaller check, recorded in the decision log. That check passed.
- **Expected:** the operator signs one statement: "this phase exits on evidence E, scenarios X are withdrawn to issue/successor Y". KBD records a closure that says exactly that, and a successor can start.
- **Actual:** `kbd-phase-close-eval.sh` returns `BLOCKED reason=evidence-missing(boundary-results.json)`. The phase stays `OPEN` indefinitely, and the decision-log entry has no effect.

## Defects

### 1. No supported exit for an operator-superseded or withdrawn boundary (design gap)

`kbd-phase-close-eval.sh` closes only on PASS evidence for every scenario in the bound snapshot. `kbd-scope-authority-template.sh` accepts only `amend-contract` and `start-successor`. Free-text decisions are explicitly ignored. That is correct for safety, but the operator has no typed alternative.

The workaround we were pushed into was to amend the contract so that its scenario matrix equals the check that actually ran, then write `boundary-results.json` to match. It works, but it hides the real history inside a re-signed contract. The closure then reads as an ordinary PASS against scenarios nobody planned.

### 2. A read-only command can persist `readOnly: true` for a replica and block all later writes (likely cause, not confirmed)

**Observed:**
- `registry.json` has this entry for replica `1cf1630a…`:
  - `readOnly: true`
  - `readOnlyReason: "cross-process flock exclusion could not be proven: project volume: cannot create lock probe file: Operation not permitted (os error 1)"`
  - `updatedAt: 2026-11-23T10:10:15Z`
- That timestamp matches a `prometheus kbd status --json` run inside an agent's sandboxed shell.
- Since then, every write is refused, from the sandbox, from the operator's own terminal and from an unsandboxed agent shell: `replica … is read-only: registry classification forbids authoritative writes`. That includes contract adoption with a valid operator signature.
- This replica had accepted writes up to revision 3700 the day before.

**Defects, if the cause is confirmed:**
- A read command writes to the machine-wide registry.
- A probe failure in one restricted process becomes sticky for every process.
- Neither the refusal message nor `kbd replicas` says how to re-probe. Whether `kbd register <path>` re-probes is undocumented.

### 3. `kbd-contract-adopt.sh --lint-only` reports clean on a contract that adoption then rejects

Without `--status`, `kbd-contract-lint.mjs` skips the `owner-task` check (the `if (status)` block). `--lint-only` doesn't pass status, so it printed `15 scenario(s) clean`. Real adoption then failed with 15 `owner-task` errors. The operator had already signed the manifest, and that signature had to be discarded.

### 4. You can sign a manifest for a contract that won't adopt

`kbd-scope-authority-template.sh amend-contract` emits a manifest and prints "Sign with: …" without running the adoption lint. A signature is the most expensive human step in this flow, so it should come last.

### 5. `kbd-reflect` sources libraries the installed pack doesn't ship

`skills/kbd-reflect/SKILL.md` sources `shared/lib/waypoint.sh`, `hooks.sh`, `bottleneck-guard.sh` and `stage-gate.sh`. In the installed 1.15.0 generation, `shared/lib/` contains only `cadence-adapters/`, so `reflect:after` and `phase:after` cannot fire as documented.

### 6. Phase progress lives in three places that disagree

For the same phase:
- runtime task state: 54 S1 tasks complete;
- `phases/<id>/progress.json`: `changes: []`, `0/0`;
- OpenSpec `tasks.md`: 0/59 checked.

Reflection's own N/T rule reads 0/0 and reports a misleading position.

## Proposed fix

### A. A typed, signed "superseded exit" disposition (the way out)

1. **New manifest action `supersede-exit`** in `kbd-scope-authority-template.sh` (namespace `prometheus-kbd-scope-amendment`). It binds:
   - `boundContractDigest`, the snapshot being superseded;
   - `operatorDecision`, the decision-log reference and verbatim text;
   - `exercised`: the scenarios that actually ran, with their verdicts, the evidence paths and the sha256 of each. They may be new IDs outside the bound contract.
   - `withdrawn`: every bound scenario and scope item not exercised, each with a destination (an issue URL, or a successor phase id);
   - `budgetAccounting`: `measured`, `estimated` or `unmetered`, with no pretend values;
   - `limits`: claims the exit explicitly does not make, for example "server-side rejection of a direct settings write not exercised".
2. **A new runtime disposition**, alongside `CLOSED`, `REFUSED_FAIL` and `REFUSED_BLOCKED`:
   - `prometheus kbd phase close --disposition superseded --manifest <m> --signature <sig>` records `CLOSED_SUPERSEDED`;
   - it is terminal and immutable, like `CLOSED`;
   - it ends execute dispatch and permits `start-successor`;
   - it never counts as certification of the withdrawn scenarios;
   - status, reflection and the closure packet show it differently from PASS.
3. **`kbd-phase-close-eval.sh --superseded <manifest> --signature <sig>`** verifies:
   - the signature against `allowed_signers`;
   - that the bound digest matches;
   - that every bound scenario appears in either `exercised` or `withdrawn`, with nothing silently dropped;
   - that each evidence digest is current;
   - that each withdrawn item has a destination.

   It then records `CLOSED_SUPERSEDED` and writes `closure-packet.md` with both lists.
4. **`CLOSED_UNMET`**, a sibling disposition for "end this phase without evidence". It uses the same manifest with `exercised: []`, so an abandoned or re-planned phase can end honestly instead of staying `OPEN`.
5. **Reflection support.** `kbd-reflect` reads the manifest and fills the goal table and Next Phase Seed from `withdrawn`. `kbd-next-phase` seeds the successor from it.

### B. Read-only classification

1. **Only write commands probe.** Read commands (`status`, `replicas`, `phase disposition`, `audit`) never write the registry.
2. **A failed probe applies to the current process only.** A later process re-probes, and a pass clears the flag.
3. **Add `prometheus kbd replicas --reprobe [--replica <id>]`**, and print that command in the read-only refusal message.
4. **Name the probe path** in `readOnlyReason`, so the operator can tell a sandbox denial from a real filesystem limit.

### C. Validate before signing

1. `kbd-scope-authority-template.sh` runs the full adoption lint, including `--status` from `prometheus kbd status --json`, and refuses to emit a manifest if it fails.
2. `kbd-contract-adopt.sh --lint-only` passes `--status` when the runtime is available, or prints that the owner-task checks were skipped.

### D. Pack integrity and projections

1. Ship the `shared/lib` files that the skills source. Alternatively, have `kbd-reflect` detect their absence and print the fallback, rather than documenting hooks that cannot run.
2. Write task completion to all three projections at the same boundary. Otherwise `kbd-apply reconcile` should flag the disagreement, and reflection should read the runtime count when `progress.json` is empty.

## Acceptance (BDD, one run each)

- **Superseded exit closes the phase:**
  - Given a contract-bound phase whose bound scenarios were never run, and a signed `supersede-exit` manifest listing 15 exercised PASS scenarios and the bound scenarios as withdrawn to an issue;
  - When the evaluator runs with `--superseded`;
  - Then it records `CLOSED_SUPERSEDED`, the closure packet lists both sets, and `start-successor` is accepted.
- **Nothing is silently dropped:** a manifest that omits one bound scenario from both lists is refused, and nothing is recorded.
- **No false certification:** after `CLOSED_SUPERSEDED`, status never reports the withdrawn scenarios as PASS.
- **Reads don't poison writes:** with a read command run under a process that denies the lock probe, a following write from an unrestricted process succeeds.
- **Re-probe clears the flag:** `replicas --reprobe` clears a stale `readOnly`, and the refusal message names that command.
- **Lint before signing:** for a contract with an unregistered `ownerTask`, the template emits no manifest and names the failing scenarios.

## Workaround in use (UAR)

- Contract amendment script: `amendments/superseding-exit.py` (lint clean with `--status`).
- Evidence writer: `amendments/superseding-exit-evidence.py`.
- Blocked at binding by defect 2, pending a registry re-register from an unrestricted terminal.
