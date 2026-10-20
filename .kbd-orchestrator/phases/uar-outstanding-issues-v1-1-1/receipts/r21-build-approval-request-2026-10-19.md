# R21 fix — verification-before-phase-completion approval request (2026-10-19 ~22:55 UTC)

Status: **PENDING OPERATOR DECISION.** Nothing below has been run. The verification gate
stays CLOSED (AGENTS.md operator lock 2026-10-17) until the operator explicitly names these
commands.

## Why this is an exception request

AGENTS.md lets the gate open only after every planned implementation task in the whole active
phase is complete. The authoritative waypoint reports `implementationCompleted 137 /
implementationTotal 152` (planRevision 63). This session did not reconcile the item-level
ledger: the child `progress.json` shows only the umbrella change as PENDING. **15
implementation items remain open, unnamed at item level.** So running the R21 oracle on a
fixed binary is verification before full phase implementation, which needs the operator's
explicit permission.

Source inspection cannot resolve it. The R21 oracle (T1) is a runtime measurement: does the
request that actually reaches the model fit the declared window plus the reserved output? Only a
built binary driving the loopback mock can show the dispatched request or the pre-dispatch refusal.

## Exact commands proposed (run serially, lead-owned, single writer)

1. Build. This is the command recorded for v5 attempt 2 (PROOF.md, embedding child, "Finished
   release [optimized] in 19m 08s, exit 0"), with a dedicated target directory so that
   `./target/release/universal-agent-runtime` — the binary the running 1907/1908 instances were
   launched from — is not replaced:

   ```
   CARGO_TARGET_DIR=/private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/opencode/uar-r14/target-r21fix cargo build --release --features server-full --bin universal-agent-runtime
   ```

   The dedicated target dir means a full cold build. Expect it to take longer than v5's
   19 minutes. If the operator prefers the shared `target/`, say so explicitly; the
   on-disk binary used by 1907/1908 would then be replaced, while their running processes
   keep the old executable.

2. Identity record (read-only): `git rev-parse HEAD`, sha256 of `git diff HEAD`, and sha256 of
   the produced binary. Recorded in this receipt.

3. Focused R21 oracle rerun: a copy of `uar-r14/r21/r21-probe.sh` with `BIN` set to the new
   binary and its expected sha256 set to the hash from step 2. Outputs go to `uar-r14/r21-fix/`.
   Same anonymous loopback mock fixture (window 5919 calibrated as before, 6 long turns, 2
   fixture skills, KB). Command: `bash .../uar-r14/r21-fix/r21-probe.sh`.

## What the result would and would not prove

- Would prove, for the NEW candidate identity only, whether T1 holds:
  - Either the dispatched request (messages + tool definitions) is at or below the window minus
    the 1000-token reserve, with trimming now leaving room for tools;
  - or the run is refused before dispatch with `context_overflow` and a liability message.
  T2–T6 would be re-observed on the same run.
- Would NOT prove:
  - the 15 open phase implementation items;
  - any other R-row on the new candidate (the R-row PASS grades are bound to v4
    `96bf4868…5154`);
  - the deferred residuals (graph-path request counting, the iteration-1 destination-preparation
    tokens, a wire-exact tokenizer);
  - the authored-but-unrun unit tests. Compilation itself is first proven by step 1.
- **New candidate identity.** The binary would include the uncommitted llm.api_key masking patch,
  the R21 fix (`reduce.rs`, `runtime/manager.rs`, `llm/orchestrator.rs`, `turn/builtin.rs`,
  `instance/pump.rs`), and any other pre-existing dirty-tree changes. It is not v4 and would be
  reported as a new candidate.

## Separately pending (not part of this request)

- R23: building `uar-sidecar` (command in `uar-r23/identity-design.md` §1) plus a ruling on
  whether a harness-asserted host subject satisfies the frozen no-login identity.
- R4: no existing mechanism serves no-login memory. This needs a product change or an accepted gap.
