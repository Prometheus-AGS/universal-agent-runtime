# TEAM-01 Mac completion — 2026-10-26

The existing TEAM-01 scenario passed: 1 scenario, 8 steps, 1m 14.458s. UAR remains running. Authentication belongs to external infrastructure and is no longer a manual-test responsibility.

## Files changed

- `scripts/behaviors/prompt.mjs`: removed test authentication dependency.
- `scripts/behaviors/steps/streaming.steps.mjs`: removed test authentication dependency.
- `scripts/behaviors/steps/persisted-agent.steps.mjs`: removed test credentials; native listener and external client endpoint are separate across restart.
- `scripts/behaviors/steps/math-knowledge.steps.mjs`: removed test authentication dependency.
- `scripts/behaviors/steps/writing-orchestrator.steps.mjs`: removed test authentication dependency.
- `scripts/behaviors/steps/math-writing-team.steps.mjs`: removed test authentication dependency; verified gateway route separately from admitted accounting pricing identity; retained raw accounting before assertions.
- `scripts/behaviors/README.md`: documented external provisioning and credential-free test invocation.
- `tools/uar-jwt-proxy/src/main.rs`: external identity host preserves tenant and issued/not-before timestamps; tests do not handle these values.
- `src/uar/runtime/cost_budget.rs`: narrows actual team call output cap against durable remaining usage before reservation/dispatch; retains atomic admission.
- `AGENTS.md`, `.prometheus/decisions.md`, `.prometheus/gotchas.md`, `.prometheus/session-log.md`: standing external-auth rule, corrections and observed results; memory service also updated.
- `openspec/changes/simplify-user-workflow-verification/specs/product-validation-evidence/spec.md`: functional scenarios do not perform authentication.
- `.prometheus/verification/`: retained earlier failures, fresh Cucumber/raw-team receipts, live-setting receipt, article, runtime metadata and updated behavior ledger.
- Local provider setting: only MiniMax-M3.max_output_tokens updated to64000 through existing settings API, with readback and preserved unrelated configuration.

## Verification actually run

`CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked -p uar-jwt-proxy`: PASS,41.35s.

`CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`: PASS,2m40s; existing warnings retained in log.

`node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/team.cucumber.mjs`: final PASS,1 scenario,8 steps,1m14.458s. Credential-free child environment is recorded in manual-team-noauth-003-result.json. This is the operator-selected manual production-path integration check, not a full-suite or release validation.

Source-only independent reviews reported no remaining actionable findings. Earlier failed runs001 and002 are preserved, not relabeled.

## Scope and guards

No unrelated scenarios, upgrades, publication or new verification machinery were added. The external proxy is the existing identity host. Identity boundaries remain enforced outside tests. The remaining-budget guard addresses observed second-call exhaustion; model-route/pricing assertions address the observed alias mismatch. No speculative guard was added.

## Unverified

Other scenarios were not rerun, and Linux/Windows/iOS were not exercised. This pass does not establish full legacy97-case coverage or every team feature. It establishes the selected concurrent math/writing and correctly merged article behavior on the current Mac runtime.

See [pass receipt](team-01-macos-pass.json), [article](team-01-macos-article.md) and [Cucumber output](manual-team-noauth-003.log). Stop for operator direction.
