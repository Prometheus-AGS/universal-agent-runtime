# Issue #386 local acceptance

On 2026-11-03, the media authority suite passed against the real macOS arm64 UAR HTTP process and a disposable embedded SurrealKV catalog. A separate local service supplied caller identities. The BDD runner received only nonsecret URLs and fixture selectors. The simulated media peer retained its state across UAR restarts.

| Scenario | Observed |
| --- | --- |
| Claims and cancellation serialize and survive UAR restarts | 9 steps passed |
| Exact authority governs meetings and simulated call takeover | 6 steps passed |
| Scope, approval and worker boundaries reject unauthorized effects | 4 steps passed |

Cucumber exited 0 with **3 scenarios and 19 steps passed** in 53.969 seconds. The suite observed one dispatch winner, an unknown outcome surviving restart, reconciliation of the original attempt without another invocation, and absence of the exercised content canary from the durable catalog and runtime diagnostics. It inspected the durable catalog after authorization refusals. The disposable fixture shut down after the run.

The tested parent was `c33607fbb82d093d43ccc8959914b9c04766dfe3` with the issue changes and the parent-pinned Liter revision `0d74c56659943e3fa44a3c8e6d51a27514f3b35c`. The UAR binary SHA-256 was `ae7e5e6b95d96251427117b68711bca70278fa3c45a1e463915b878b215c5166`. The Cucumber JSON report SHA-256 was `41dc2a626b3c08a84f00b53055f998ef8df0dd3906b34a3e632fbfc9515fc79c`.

`cargo build --locked --no-default-features --features minimal,local-models --bin universal-agent-runtime --example mock_media_peer --example surrealql_store_runner` finished with zero warnings. The command aligned the native C/C++ minimum at macOS 11.0. A temporary compiler wrapper selected LLVM LLD and macOS 13.4 for the UAR executable, matching the bundled ONNX Runtime minimum. No build settings changed in the repository.

`node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/media-effects.cucumber.mjs` ran the live suite. `openspec validate dot-media-action-authority --strict` and `git diff --cached --check` exited 0. The independent artifact review found no blocking defect and identified a weak peer-count assertion in the denial scenario. That assertion was replaced by inspection of the durable catalog.

Original logs, source and binary identities, and the Cucumber report are retained locally under `.prometheus/verification/issue-386/`. Apple’s default linker reported the compact unwind size warning in both debug profiles attempted. Acceptance covers the tested local server profile and simulated peer.
