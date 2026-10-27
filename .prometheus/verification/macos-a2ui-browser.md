# A2UI-BROWSER-01 — Mac PASS

One selected browser BDD passed: **1 test, 5 Gherkin steps**, 25.4 seconds (Playwright summary 28.4 seconds). No previously verified scenario ran.

```sh
node_modules/.bin/playwright test --config scripts/behaviors/a2ui-browser.config.mjs
```

Observed output: `1 passed (28.4s)`.

A disposable Liter/MiniMax-M3 agent genuinely called `a2ui_render`. Chrome displayed the calculator textbox and button. The test entered fresh value **606** and clicked that rendered button. The actual action POST returned **202**, carrying `parcel: "606"`; its distinct continuation run returned **625** and the marker stored only in the agent's instructions. Both were present in the assistant DOM. The disposable agent was deleted. No page errors or test authentication. UAR remains running with the same store and saved agents.

## Changes and observed defects

- `scripts/behaviors/a2ui-browser.config.mjs`: isolated Playwright-BDD configuration, one worker, zero automatic retries, retained reports/video/trace.
- `scripts/behaviors/features/a2ui-browser.feature`: five-step rendering/action scenario.
- `scripts/behaviors/browser/a2ui-browser.steps.mjs`: real browser interaction, exact action/continuation/arithmetic assertions, owned disposable cleanup and evidence. Response-wait rejection handling preserves an earlier observed UI failure rather than producing an unhandled rejection.
- `src/uar/runtime/native_skills/a2ui_render.rs`: native tool schema now permits the UAR catalog already accepted by the backend parser and required by the browser. Existing basic-catalog allowance remains.
- README, OpenSpec scenario delta, this receipt/report, behavior ledger and append-only memory record the selected behavior and its limits.

Earlier attempts remain in `a2ui-browser-001-macos` through `006`. They exposed missing Playwright FFmpeg, model-generated malformed tool arguments, an older served frontend, and the native-tool/browser catalog mismatch. The fixture now provides the exact valid template while still requiring genuine model/native-tool publication. FFmpeg was installed; the current frontend was built and served by only the owned local runtime. The two-line Rust fix built successfully with the existing server-full target in 153.25 seconds; existing compiler warnings remain. Reviews found no remaining blocker in the correction. The Rust-specialist role provided no verdict because it requested broader checks; no broader checks ran.

## Evidence and limits

[Receipt](a2ui-browser-01-macos-pass.json), [raw exchanges and action](a2ui-browser-007-macos/evidence.json), [Cucumber report](a2ui-browser-007-macos/cucumber.json), [Playwright report](a2ui-browser-007-macos/playwright.json), [rendered form](a2ui-browser-007-macos/rendered-surface.png). Video and trace are in the attempt's `artifacts/` directory. The continuation screenshot captures the form viewport, not the final answer; the exact answer was asserted in the assistant DOM and retained stream.

This is one successful workflow on Chrome/Mac, not repeat reliability, all components/catalogs, replay, other engines/platforms, or the dedicated AG-UI agent-route projection. The tool's existing example still uses the basic catalog; full catalog parity remains unverified. No unrelated additions or prior scenario reruns. Historical passes remain historical evidence. Stop for operator direction.
