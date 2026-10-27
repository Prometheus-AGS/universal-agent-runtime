# CONVERSATION-RESTART-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/conversation-restart.cucumber.mjs
```

Observed: **1 scenario (1 passed), 6 steps (6 passed), 17.442 seconds**. Five behavior steps plus disposable-agent cleanup. Only this new scenario ran, twice; no previously verified scenarios ran.

A disposable agent acknowledged fresh facts with only READY. UAR process **47246** exited gracefully and process **47773** started with the same binary, configuration, environment, working directory and existing store. The external proxy stayed running. Endpoint/process binding was checked before and after restart. The saved disposable agent definition was unchanged.

A new prompt supplied only the original session ID and a question, without replaying facts or history. The response recovered `CASE-a04dddea-e965-4e2c-a1a2-70d6bd22e19e` and correctly computed **959** from the earlier quantities. Both distinct runs belong to session `0be2a4a8-c415-4bf7-9769-191ad66ea22a`. The disposable agent was deleted. Saved specialists and the store remain; UAR is running.

## Observed defect and repair

The first attempt really restarted UAR but returned null for both recalled facts: 1 failed scenario, 5 passed steps and 1 failed step, 26.339 seconds. The raw response and failure are retained. Normal chat resolution created a new empty local session after restart, and successful text-only turns were not saved by that path.

`src/uar/runtime/manager.rs` now loads owner-scoped durable history on a cold cache miss and saves successful turns before terminal completion. Store errors surface as run errors instead of silently claiming success. This is host persistence work, not a kernel write. No model switch, fabricated facts or weakened recall assertion.

The necessary local build ran once:

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
```

Observed: exit0, `Finished dev profile ... in 3m 32s`; warnings retained in build.log. The repaired binary was installed with an explicit digest before the BDD. The actual tested restart required unchanged binary/config digests. No broad suite, lint or previous scenario rerun.

## Files and limits

- `scripts/behaviors/conversation-restart.cucumber.mjs`: selects only this feature with retries disabled.
- `scripts/behaviors/features/conversation-restart.feature`: the one restart/recall behavior.
- `scripts/behaviors/steps/conversation-restart.steps.mjs`: random facts, genuine inference, no transcript replay, exact answer and cleanup.
- `scripts/behaviors/support/restart-owned-uar.mjs`: owned Mac process lifecycle; endpoint binding, lossless NUL-delimited environment capture using existing Python standard library, same-store restart and explicit binary replacement support for the repair installation.
- `src/uar/runtime/manager.rs`: the two persistence-path repairs above.
- Behavior README and OpenSpec delta: scenario and durable completion requirement. Ledger, receipt, runtime metadata and append-only memory: actual results.

No unrelated implementation was added. Guards address observed ownership/endpoint binding, environment preservation, aggregate step deadlines and actual persistence failures. No credentials supplied by the test, authentication work, provider configuration changes, publication or store reset.

**Uncomfortable limit:** source review identified that overlapping writes to the same conversation can commit whole-session snapshots out of order, and concurrent cold cache creation is not atomic. This sequential PASS does not certify concurrent same-session durability. That concern is recorded without expanding this manual scenario. Crash recovery, OS reboot, other storage backends and other platforms were not exercised. No general durability guarantee is claimed.

[PASS receipt](conversation-restart-01-macos-pass.json), [raw evidence](conversation-restart/0be2a4a8-c415-4bf7-9769-191ad66ea22a.json), [passing Cucumber report](conversation-restart-002-macos/cucumber.json), [initial failure](conversation-restart-001-macos/cucumber.json), [build receipt](conversation-restart-repair/build.json), [source reviews](conversation-restart-repair/source-review.json).

Complete for the manual sequence; stop for operator direction.
