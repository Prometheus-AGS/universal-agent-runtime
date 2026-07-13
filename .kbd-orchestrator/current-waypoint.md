# Current Waypoint — Universal Agent Runtime

> EXECUTION LOCK: Reach 24/24 production completion. CI is asynchronous evidence, not the work queue. Do not monitor workflows while actionable implementation or release work remains. Operator instructions override stale context.

- Phase: `uar-final-production-hardening-2026-07`
- Stage: final certification and release
- Product: `server-full` BossFang sidecar
- Progress: 19/24 changes formally complete; changes 20–24 are implementation-complete and locally validated, and await immutable candidate evidence, time-bound conditions, or operator-authorized publication
- Stable platforms: Linux and macOS
- Experimental/nonblocking: Windows
- Active surface: changes 20–24 only
- Local validation: authoritative `cargo check`, consolidated Rust formatting/tests, frontend typecheck/lint/tests/build, workflow/schema validators, OpenSpec validation, YAML parsing, and diff hygiene are green
- Next action: commit and review the validated patch, then hold the source stable, obtain authorization plus a configured signing identity at external-effect gates, and execute one immutable RC → evidence → GA sequence

Before every action ask: **Does this directly advance changes 20–24 toward completion?** If not, do not do it.
