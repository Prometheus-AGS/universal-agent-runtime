# Capability-dispatch JavaScript artifact review

Scope: the new Cucumber config, feature, steps, support helper, synthetic asset generator, manifest/runtime templates and fixture README; OpenSpec `carry-inline-images-and-constrain-dispatch`; supporting public route schemas and controlled-peer source. Cumulative source base: `629f5702`. Phase: `production-readiness-476-capability-dispatch`, Execute / acceptance pending. Fresh native context, same model family; this is weaker isolation than an independent-family review. No producer handoff or self-assessment used.

This is source inspection only, not runtime certification. No tests, lint, format, typecheck, compiler/import probes, builds, Docker/service or network commands ran. JavaScript-only scope has no TypeScript check; lint and execution remain NOT_EXERCISED under the delegated source-only boundary. PR merge readiness was not available and is unverified.

## Findings

### P1 — Cucumber can advance while the timed-out case still mutates shared peers

Location: `scripts/behaviors/steps/capability-dispatch.steps.mjs:8` and `:256`; supporting limits at `scripts/behaviors/support/capability-fixture.mjs:103`, `:117`, `:123`, `:129`, `:243`.

The runner's 180-second timeout is shorter than the sequential operations it wraps. With the supplied twelve-peer manifest, one `send()` permits 24 status requests at 15 seconds each plus a 120-second completion request. Cleanup permits two 120-second public DELETEs and 24 peer control requests. Slow but successful requests can therefore exceed 180 seconds without hitting their own deadlines. For example, twelve status responses taking ten seconds each, one fast completion, and twelve more ten-second status responses time out the step during its second snapshot.

The installed runner confirms that timeout is a `Promise.race`, not cancellation (`node_modules/@cucumber/cucumber/lib/time.js:24`, `user_code_runner.js:60`). It returns control while the original async function remains live. `After` then operates on the same world and resets peer controls, and a timed-out `After` can continue resetting `default` while the next scenario configures it. This defeats serial isolation and can contaminate counts, failure modes and receipts. It also makes cleanup uncertainty outlive the reported failure.

Minimal fix: establish a bounded lifetime for each complete step and cleanup operation, with a runner timeout strictly longer than all its owned awaited work. Use a shared operation deadline/cancellation signal across requests and join completion before cleanup or the next scenario; give cleanup its own bounded budget. Do not rely on raising the per-request timeout or on `parallel:0`. The exact scenario/API work does not otherwise need changing.

Falsifier for the repaired source: every compound operation has a finite total budget below its runner wrapper, and expiration aborts/settles its requests before that wrapper returns. Later authorized slow-peer execution must demonstrate no controls or observations occur after the hook/step settles. No such execution was performed here.

### P2 — Estimated-accounting oracle can pass without any final-wire accounting receipt

Location: `scripts/behaviors/steps/capability-dispatch.steps.mjs:239` through `:246`.

The visitor pools `uar.request-budget.v1` and `uar.attempt-manifest.v1` records, then requires only one pooled record and `fit_guarantee:false`. The orchestrator's ordinary attempt manifest already reports false independently of the destination driver's final-wire count (`src/llm/orchestrator.rs:1192`). If the actual driver's request-budget receipt disappears, this scenario still passes on the attempt manifest alone. It also never asserts an approximate/estimated count quality or the final-provider representation, despite the step claiming estimated image accounting. Correct typed delivery from the earlier step does not establish those accounting facts.

Minimal fix: require at least one `uar.request-budget.v1` receipt for the actual destination, `representation:final_provider_request`, `count_quality:approximate`, and `fit_guarantee:false`; keep separate checks on all relevant attempt manifests if desired. Tie the receipt to this run's observed destination. The explicit synthetic exact contract remains outside this case.

Falsifier: suppress only final request-budget events while preserving attempt manifests and the successful image request; the same accounting assertion must fail. This is an authored counterexample, not an executed result.

## Source observations and limits

- Both ordinary aliases actually mount `server.rs::api_chat_completion` (`src/server.rs:2421`, `:2794`), whose request accepts the fixture's typed messages, session/agent IDs, output cap, tools and response format. The separate OpenAI-agent endpoint's string-content schema is not the mounted schema used here.
- Registry inspection matches `ProviderView` and `ModelConfig`; health reads match `providers[].healthy/cooldown_remaining_secs`. Agent POST returns 201; owner-scoped session DELETE returns 204/404. Run listing exposes `conversation_id/run_id`; event snapshots expose camelCase `gapReason`.
- Peer arrivals are incremented before body parsing. The helper compares arrival deltas to observation deltas, so a fixture body rejection cannot earn zero-send credit. Observation-ring truncation is rejected. Actual ordered parts use safe hashes/counts and the swapped-hash control uses the same comparator.
- Digest recomputation recursively sorts object keys, preserves array order and removes only the declared contract stamp and root normalization object. Source/config/image identity formatting does not establish serving process identity; the README correctly assigns independent receipts to external provisioning.
- URL checks require credential-free literal loopback and forbid redirects. Fixture registry equality excludes undeclared live providers. Namespace ownership, secure external identity, local model assets and network namespace reachability still require external provisioning; a loopback URL alone proves none of those facts.
- Public response privacy assertions carry fixed messages and retain only hashes/counts. Part comparison failures expose safe summaries, not image bytes. No raw image request/response is intentionally attached. Real decoder acceptance, privacy behavior, cooldown, retry/fallback, mounted output fields and actual cleanup remain NOT_EXERCISED.
- Cold checkpoint and public strict-real-image selector coverage are explicitly deferred/excluded in the adopted scope. No genuine vision understanding, provider quality, platform or release acceptance follows from these deterministic cases.

Only this review file was written. No implementation guard or production code was added. No unrelated addition was made. The findings are source-demonstrable failure mechanisms; their proposed execution falsifiers remain unrun.
