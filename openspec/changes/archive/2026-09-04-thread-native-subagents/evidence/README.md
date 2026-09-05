# Live child-cancellation evidence

Run locally at the phase boundary, with `LITER_LLM_BASE_URL` (including its
configured API prefix) and `LITER_LLM_MASTER_KEY` already in the environment:

```sh
node openspec/changes/thread-native-subagents/evidence/live-cancellation.mjs target/debug/uar-sidecar
```

`UAR_SMOKE_MODEL` defaults to `gpt-5.4`. The observer forwards requests to the
real configured provider; it supplies no model fixtures. Each attempt uses a
temporary database, a random JWT secret, and copies the three checked-in Cedar
policy files unchanged. Native file, terminal, and web tools are disabled. The
requested artifact allows only the three agent-management tools needed by this
scenario, and the script approves only an explicit `spawn_agent` request.

The smoke requires one logical router and one active child; network attempts
are recorded separately because provider settings can retain driver retries.
The child request is identified by the registered general-purpose artifact's
instruction, and its persisted thread/run identity must match the observed
running and cancelled lifecycle events. Every earlier attempt must be closed,
the selected child must be the sole active attempt, and cancellation must close
all attempts without starting another. Natural completion or a provider error
cannot satisfy the aborted-read assertion.

By default, the scenario requires child response text before cancellation.
The separately labelled `--before-first-response` scenario instead cancels
while the child's outbound request is pending, after the real router has
responded. It passed on 2026-09-04:

```sh
UAR_SMOKE_MODEL=k3 UAR_SMOKE_LOG=info node openspec/changes/thread-native-subagents/evidence/live-cancellation.mjs target/debug/uar-sidecar --before-first-response
```

The command exited 0. Its output is retained in `live-cancellation-report.json`.
The artifact-only acceptance review confirmed that this scenario meets task
6.3, whose criterion does not require emitted child text. The after-text
scenario remains unverified: gpt-5.4 and k3 child requests hit stream-start
timeouts; MiniMax-M3 also encountered provider 500/502 responses. No failed
attempt is represented as a passing after-text test.

The isolated configuration requests a 90-second stream-start allowance and no
retries; observed provider-level settings can still retain driver retries.
An explicit `uar.run_policy` selects no skills, MCP servers or knowledge bases,
and disables memory. Merely setting `max_active: 0` did not remove the 1,044
eligible skill IDs from the world-state permissions summary, which exceeded an
unrecognized model's 8,192-token budget. No production setting was changed.

The executable from the latest full integration build can instead launch the
same real server, without another Cargo invocation:

```sh
node openspec/changes/thread-native-subagents/evidence/live-cancellation.mjs /absolute/path/to/integration-test-executable --integration-helper
```

That helper has a fixed 30-second readiness limit. On 2026-09-04, two attempts
hit that limit while scanning the operator's 1,044 installed skills; no model
requests were made in either attempt. Another attempt reached the real
provider successfully, then correctly denied delegation because the isolated
directory had no policies. The runner now copies the repository policy set.
These failed attempts are not cancellation evidence.

Uncomfortable limit: this is one local graph-child scenario, not a live remote
peer cancellation test. A closed outbound request does not prove provider-side
computation or billing stopped. The runner's `shadows` field is diagnostic only; an
empty collection does not establish typed-turn parity. The typed-default gate
requires its own nonempty live comparison record. Retained scratch data includes
the isolated database and a redacted server log; its config contains the
throwaway JWT secret, never the upstream provider key.
