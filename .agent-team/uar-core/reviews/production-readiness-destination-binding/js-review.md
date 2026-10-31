# Independent JavaScript source review

Scope: one pass over the unstaged new global-fallback control artifact versus the index in `scripts/behaviors/support/capability-fixture.mjs`, `scripts/behaviors/steps/capability-dispatch.steps.mjs`, the fixture manifest/runtime templates, and relevant feature/API/serializer context. Phase: `production-readiness-476-capability-dispatch`, Execute. This is a local source review; PR merge readiness was not available or assessed. No producer handoffs, generation history, self-assessments, or previous review reports were consulted. Same-family reviewer isolation is weaker than cross-model isolation.

## Findings

No actionable defect found in the bounded JavaScript diff.

The source provides non-vacuous falsifiers:

- Initialization reads the existing settings API and requires one authoritative effective row for each global failover field, with the actual outside destination/base URL. It separately reads effective `llm.max_retries`, `resilience.retries_enabled`, and `resilience.retry_max_attempts`, matching actual registered setting keys and response shape. The templates select priority global failover, threshold 3, client retries 0, and host retries disabled/attempt bound 1.
- The new Given step preserves the restricted nonempty policy, creates a separate disposable unrestricted agent and session, requires exactly one observed primary call followed by outside delivery and a useful success, and checks exactly one primary health failure while still below cooldown. It restores the original agent, primary, fallback list, and session in `finally`. The following restricted positive case requires its captured capable fallback and zero outside calls; exact/scope negatives require failure and primary-only observations. Substituting the configured global outside chain for a nonempty artifact policy would fail these assertions.
- The qualified global routing model remains intact in configuration. The wire alias assertion matches the vendored generic provider's construction hint and matching-prefix stripping (`vendor/git/liter-llm/crates/liter-llm/src/client/mod.rs`, `provider/mod.rs`), rather than comparing the wire body to the qualified route blindly.
- Control session IDs are added before sends; agent IDs are added before creation POST. Both restricted and control IDs remain in the all-owned cleanup loops, immutable ownership record, normal receipt, and uncertainty lists. The existing ordered synthetic image comparator and captured fallback's own 32-token ceiling assertions remain in place.

## Evidence and limits

Commands were classified as source inspection: `pwd`, `cat`, `sed`, `rg`, `git diff --stat`, and targeted `git diff` reads. They exposed the source and API contracts described above; they did not execute the artifact. Some exploratory `rg` paths did not exist and were corrected to actual source locations; no validation result is implied.

No test, import, syntax check, lint, format check, typecheck, build, Cargo, Docker, service, network request, or other execution/verification command ran. Runtime behavior, actual settings responses, serializer output, operational topology, timeout behavior, cleanup outcomes, and selected artifact identity remain unverified. This review grants no phase acceptance or visual-understanding credit.

Changed only this report. No implementation changes, new guards, or unrequested production work were added.
