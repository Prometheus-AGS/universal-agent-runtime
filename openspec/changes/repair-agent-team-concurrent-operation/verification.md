# Verification — existing team scenario only

TEAM-01 is the sole acceptance row: `scripts/behaviors/features/math-writing-team.feature`, scenario `Concurrent calculation and editorial work merge into one article`.

Given the existing saved math and writing agents, durable UAR store and configured real Liter/MiniMax-M3 model, run the existing project. Require distinct actual member runs to overlap, correct successful individual results, and a final writer article containing the math result and editorial work derived from both actual output artifacts. Existing accounting/assertions remain honest. Queued tasks, simultaneous client requests or a fabricated answer are not success.

After all child implementation, source inventory and explicit final gate authority, serially build the repaired candidate and run:

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs
```

The actual final packet names independent Rust/code review and exact owned restart/config/environment details; no implicit probes. Reuse the existing persistent fixture and retain all saved agents/KBs and failures. Keep repaired UAR running. No separate ADMIT/SETTLE, replay/unknown-usage fixtures, new recovery API or unrelated accepted test reruns.

Record source/binary identity, exact commands and actual BDD report plus member/run/artifact evidence under stable .prometheus/verification/teams paths. First failure: preserve it and discuss the specific fix versus backlog; no automatic retry. PASS supports only this Mac team behavior. Reflect and hand off to the parent after actual acceptance. No execution occurred during Plan.

## Current operator disposition — 2026-10-26
The earlier packet/signature/gate workflow above is retained as historical planning. The operator replaced it with one-at-a-time manual BDD verification and explicitly authorized fixing the existing team scenario until it passed. TEAM-01 is now PASS on Mac:1scenario,8steps,1m14.458s; see `.prometheus/verification/team-01-macos-pass.json`. Do not repeat that scenario in this manual sequence or infer completion of other platform/case obligations. Await the next operator-selected behavior.
