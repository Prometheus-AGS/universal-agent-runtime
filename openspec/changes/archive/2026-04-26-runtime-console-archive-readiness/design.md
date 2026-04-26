## Context

The KBD phase now has archived evidence for frontend lint cleanup, runtime console visual tests, runtime event replay, Surreal Memory workflow mirroring, OpenSpec validation cleanup, and Moonshot provider diagnostic status. The final gate should summarize that evidence, rerun the most relevant commands, and then archive the canonical `runtime-console-entity-workflow` change.

## Goals

- Confirm all dependent KBD changes are archived with PASS refiner logs.
- Rerun the active OpenSpec, frontend, and focused backend gates.
- Keep generated static asset churn out of the worktree.
- Archive `runtime-console-entity-workflow` once its remaining validation task is complete.
- Leave KBD in a reflection-ready state.

## Non-Goals

- Do not broaden this into a full `cargo test` across unrelated surfaces if focused tests already cover the phase risks.
- Do not perform live provider calls or write provider credentials.
- Do not change runtime console UX during archive readiness.

## Decisions

- Use `openspec validate --changes` for active change health because repository-wide `--all` includes unrelated historical specs.
- Use targeted Playwright suites that were added for this phase rather than rerunning unrelated browser tests.
- Treat `static/index.html` hash churn from Rust build scripts as generated output and restore it after backend test builds.
- Archive `runtime-console-entity-workflow` through the OpenSpec CLI so spec sync happens consistently.

## Risks

- The final gate may expose unrelated active-change failures. If so, KBD should record the narrow blocker rather than hiding it.
- Build scripts can rewrite static asset hashes during Rust tests. The final check must confirm the file is clean.
