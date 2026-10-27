# Packet002 independent Rust source review

Disposition: PASS for the bounded Rust repair and activation logic, subject to the limitations below. No blocking finding. Verification gate remains CLOSED; this is source inspection, not acceptance evidence or authorization to execute the packet.

Scope: token-budget-repair.patch; affected allocation, request construction and context sizing in cost_budget.rs, manager.rs and team_execution/execution.rs; directly called Liter request construction, durable accounting and team settlement; verification-packet-002.md literal build/activation commands. The scenario feature and Cucumber configuration were read to establish the unchanged TEAM-01 selector and retry=0. No producer history, memory/gotchas or reviewer reports were read.

## Source conclusions

- cost_budget.rs:24–36 bounds output to min(64,000, total/2), keeps input separate, rejects allocations without positive input/output, and retains the total. A 128,000-token attempt allocates 64,000 to each side.
- team_execution/execution.rs:285–310 installs that output cap and preserves the attempt's whole token, cost and time grant. manager.rs:6464–6500 derives the input allowance from the host grant and narrows the outbound output cap against configured/model ceilings. The settings-only context path uses the host input allowance, limited further by the model window and output reserve.
- cost_budget.rs:1820–1878 reserves the explicit host input allowance plus the capped output for team requests without a final-wire contract. It does not relabel the allowance as measured input. Existing exact-contract admission continues through request_budget_bound.
- manager.rs:62–76 propagates or narrows the output ceiling into the graph driver. liter_driver.rs:493–550 rejects zero and conflicting extra-parameter ceilings and writes the typed cap to the outbound request field.
- Provider usage is not replaced with the reservation: cost_budget.rs:1510–1577 retains observed cumulative prompt+completion tokens, records durable usage, and requires priced terminal confirmation before marking usage complete. persistence/accounting.rs:218–261 preserves observed tokens and settles the observed amount rather than clipping it to the original allowance. team_execution/execution.rs:208–246 retains the whole team reservation when usage/effect certainty is missing; settlement.rs stores supplied actual usage without clipping.
- The build command targets the declared native binary with server-full, which includes server. Activation distinguishes the pre-build disk identity from the replacement binary, compares retained launch metadata, checks the named process command/config and authority expiry before SIGINT, refuses force-kill, requires a released store before copying it, launches from the retained fixture directory, and saves candidate identity/readiness before the scenario. These are source observations only; process ownership, private configuration, store state, executable identity and readiness were not probed.

## Nonblocking documentation finding

[LOW] Close the pre-build shell fence.
File: verification-packet-002.md:49–53.
Issue: PY_PREBUILD is followed by prose without a closing Markdown fence. The next opening fence therefore leaves subsequent rendered command blocks misaligned, and copying the rendered pre-build block can include prose as shell commands.
Fix: Add a closing triple-backtick line immediately after PY_PREBUILD. The Python heredoc and Cargo command themselves have no source-level blocker identified here.

## Evidence and limits

Only read-only source commands were executed: pwd, cat, sed, rg, nl, and git diff --staged --stat / scoped git diff --stat. The staged summary was empty; the scoped working-tree summary listed cost_budget.rs, manager.rs and team_execution/execution.rs. The narrow repair patch was read separately because the working-tree diff contains broader pre-existing work. The waypoint reports agent-team-concurrency-repair, task1.4, and gate CLOSED.

No test, Cargo command, build, lint, formatter, compiler/import probe, process/service/API check, manifest comparison, hash verification or acceptance script was run. Compilation, runtime concurrency, successful activation, real provider field behavior, final article correctness and actual cost/token usage remain unverified. The settings-only input allowance is estimated context sizing, not exact provider tokenization or a guaranteed model fit. Existing exact-contract and auxiliary/multiple-call behavior outside the single TEAM-01 repair is not certified by this review.

Changed only this requested review artifact. No production edits, extra guards, unrelated additions or source identity claims were made.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 1 | note |

Verdict: APPROVE — no blocking source finding in the bounded repair; runtime acceptance remains unverified.
