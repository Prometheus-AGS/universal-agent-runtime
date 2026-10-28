# REST-SETTING-RESTART-01 — Mac PASS

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/rest-setting-restart.cucumber.mjs
```

Observed on the first attempt: **1 scenario (1 passed), 5 steps (5 passed), 13.834 seconds**. Four behavior steps plus restoration hook. Only this new scenario ran.

GET `/api/uar/user/settings` returned `prompt_caching_enabled: null` (inherit). PUT changed only that field to `true`; an immediate GET matched the saved record. The owned UAR stopped gracefully (PID52394) and restarted (PID62351), keeping the same store, binary, configuration, environment and working directory. Endpoint/process ownership was checked before and after restart.

A fresh GET after restart returned the complete saved record unchanged, including `updated_at: 2026-10-27T00:05:51.736900Z`, the same user and `preferred_scope: session`. No PUT was replayed after restart before this assertion. Cleanup restored `prompt_caching_enabled: null` and confirmed the original preference fields. Its update timestamp appropriately changed; this restores preference semantics, not the original record timestamp or possible record absence.

## Changes and review

- `scripts/behaviors/rest-setting-restart.cucumber.mjs`: selects only this feature, with retries disabled.
- `scripts/behaviors/features/rest-setting-restart.feature`: the one update/restart/readback behavior.
- `scripts/behaviors/steps/rest-setting-restart.steps.mjs`: captures the original, changes one field, checks durable readback and restores the nullable value.
- Behavior README and OpenSpec delta document this scenario; receipt, ledger and runtime metadata record the observed result.
- Karpathy process log and append-only public conversation export preserve planning, execution and lessons for later skill extraction; project memory records the result.

Code and JavaScript source reviewers reported no concrete blockers. There were no live failures, product edits, rebuilds, model/provider calls, test credentials, store resets, publication or previously verified scenario reruns. The existing host restart adapter was reused unchanged.

No unrelated implementation was added. Guards cover actual REST/ownership boundaries, bounded operations, preserving unrelated preference fields, and refusing an observed outside settings change during restoration. Restoration failure would fail this scenario. UAR remains running with the original preference restored.

## Limits

This certifies REST persistence of this one per-user preference across this graceful Mac restart. **It does not prove that prompt caching is effective**, or cover all runtime settings, every nullable transition, crash recovery, other backends or other platforms. No inference was needed to prove the requested stored-setting behavior.

[PASS receipt](rest-setting-restart-01-macos-pass.json), [raw exchanges](rest-setting-restart/1d90d1c7-8dca-4092-9c43-59c648439b81.json), [Cucumber report](rest-setting-restart-001-macos/cucumber.json), [Karpathy process log](../knowledge/karpathy/manual-bdd-evolution-01a16d22/process.md).

Complete for this manual sequence. Stopped for operator direction.
