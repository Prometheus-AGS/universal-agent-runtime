# Local gate source review

2026-10-22. Phase `uar-post-lifecycle-verification-readiness`, Execute. Verification CLOSED. This is the single explicitly authorized source-only review of `scripts/release/client_tools_local_gate.py` and `openspec/changes/openai-client-tools/handoffs/local-gate.md`. No executable product verification was performed.

Verdict: **BLOCK execution readiness pending cleanup corrections.** This is not runtime certification. Producer identity was not supplied; reviewer is an isolated native GPT-6-family agent with artifact/intent access and no producer conversation. Same-family isolation is weaker than a different-family judge; different-family certification remains pending. No additional judge was dispatched.

## Findings

### [CRITICAL / HIGH] Cleanup failure abandons remaining owned children

File: `scripts/release/client_tools_local_gate.py:159–178`, `:262–270`.

Exact source: `for name, child in reversed(self.children):`, followed by `child.send_signal(signal.SIGTERM)`, `os.killpg(child.pid, signal.SIGKILL)`, and `child.wait(timeout=15)`. The only error handler is outside the entire `supervisor.stop_join()` call: `except (OSError, subprocess.TimeoutExpired) as error:`. Log closure is after the loop.

Issue: an OSError from one child's signal/kill, or a final bounded-join timeout, exits the entire cleanup loop. Candidate and provider entries remaining in reverse order receive no stop/join attempt, and logs are not closed by the supervisor. The outer handler records failure but does not resume cleanup. This contradicts handoff lines 63–72, which promise stop/join of each child and log closure on failure. This is a demonstrated control-flow defect, not evidence that any current child actually resists shutdown.

Unexecuted falsifying scenario: matrix teardown raises OSError or its post-kill wait times out; observe whether candidate and provider still receive stop/join attempts. The current source has no such continuation path.

Fix: handle and record cleanup exceptions per child, continue to every remaining owned child, and close all logs in a final cleanup block. Preserve the first failure and individual attempted/joined/forced-kill receipts rather than asserting joined on failure.

### [CRITICAL / HIGH] Repeated interruption bypasses cleanup and final receipt

File: `scripts/release/client_tools_local_gate.py:197–199`, `:258–272`.

Exact source: `def interrupted(signum, _frame): raise KeyboardInterrupt("operator signal " + str(signum))`; this handler remains active when `finally` calls `supervisor.stop_join()`. That cleanup call catches only `(OSError, subprocess.TimeoutExpired)`.

Issue: a second SIGTERM, or Ctrl-C, during the cleanup wait raises KeyboardInterrupt out of the finally block. Remaining children are abandoned and `private_json(evidence / "local-gate.json", report)` is skipped. The initial KeyboardInterrupt catch at line 258 cannot catch an exception raised in its own finally block. This directly conflicts with the stated interruption ownership contract.

Unexecuted falsifying scenario: interrupt the supervisor once during matrix execution and again during candidate shutdown; current source has no handler that ensures remaining cleanup/reporting completes.

Fix: enter an explicit cleanup state that defers repeated SIGTERM/SIGINT interruption until bounded cleanup and report persistence complete. Restore handlers afterward as appropriate. Keep failure status for interrupted execution.

### [WARNING / MEDIUM] Preserved HOME permits operator skill discovery

File: `scripts/release/client_tools_local_gate.py:60–65`, `:121–142`; handoff lines 36–47.

Exact source: the child allowlist includes `"HOME"`; generated configuration has no `skill_source_roots` override. `src/server.rs:1688–1694` defaults roots to `skills` plus `standard_agent_skills_dir()`. `src/uar/runtime/skills/storage/filesystem.rs:558–564` resolves the latter to the user's `.agents/skills` directory. Startup then initializes and reconciles those skills (`src/server.rs:1712` onward).

Issue: a fresh working directory and credential environment allowlist do not isolate all operator filesystem configuration. The candidate can discover the real user's skills, so fixture state can depend on host-installed content. No credential theft, external execution, or leakage was demonstrated; this finding does not claim those effects.

Fix: use a private HOME directory in the evidence tree and explicitly configure the intended skill roots, or disclose the retained operator skill dependency and narrow the isolation claim. An explicit root list alone should be assessed against other HOME-based discovery before claiming complete filesystem isolation.

## Binding and reporting observations

- Read the entire supervisor, handoff, the two change spec deltas, team policy/map, and relevant referenced contracts. `/api/agents` source accepts AgentArtifact and returns 201 with the saved artifact (`src/uar/api/discovery.rs:316–329`). Tool discovery exposes `source`, server/raw name, and `namespaced_name` (`:205` onward), matching the supervisor's lookup. `MCP_CONFIG_PATH` resolution is present (`src/mcp/registry.rs:3027` onward). Explicit config bypasses implicit home config (`src/config.rs:1706` onward), and main does not implicitly load `.env` (`src/main.rs:38` onward).
- Fresh secret/JWT construction, private exclusive credential/config creation, subprocess argument lists, and child environment filtering are visible in source. Tokens are passed via matrix environment names rather than command arguments. This establishes authored mechanisms only; actual JWT admission and absence of secrets from child-generated logs remain unverified.
- Matrix invocation explicitly names OpenAI, Anthropic, AG-UI, A2A and ACP (`local_gate.py:240–245`). The acceptance program iterates selected protocols and records the authored matrix cases (`client_tools_acceptance.py:587–604`). The supervisor requires nonempty all-passed results and a normal matrix exit code before setting `matrix_passed_remaining_gates` (`local_gate.py:249–257`).
- Exit 2 and explicit cold restart, graph and cumulative-budget gaps are truthful by source. They do not fulfill the durable restart/graph/budget spec requirements. No speculative quota finding is raised. No startup/config failure was demonstrated in this source-only pass; this bounded inspection is not exhaustive proof of runtime binding compatibility.

## Method, commands and limits

All shell commands were classified as source inspection: `git diff -- '*.py'` returned empty output; `cat`, `sed`, `nl`, `head`, and `rg` read the named artifacts and source contracts. Empty tracked diff did not establish absence of the supplied file; it was read directly. Two bounded `rg` requests named nonexistent provider/embedding paths and returned path-not-found diagnostics; no behavior conclusion relies on those paths. No imports, pycompile, script invocation (including help), static analysis, startup, network probe, test, build, packaging, or other product gate ran. Static analysis was intentionally not attempted under the operator lock.

Skills applied: `uar-project-review`, `review-verification-protocol`, and installed `python-patterns`. Python resource cleanup/error propagation guidance applies to the demonstrated teardown paths. Style-only annotation/line-length changes are not requested by this bounded review.

The available `mcp__sycophancy_correction.detect_sycophancy` screened the finding/verdict text with strict sensitivity: score **0.0**, classifications **[]**, correction mandatory **false**, audit timestamp `2026-10-22T16:38:53.189413Z`. This screened editorial bias, not product behavior. No automatic correction claim is made.

Only this review artifact was added. No production files, credentials, configuration, or guards were changed. Proposed guards trace to the stated failure/interrupt cleanup contract and actual HOME discovery boundary. Current source state is the freshly read local file content and exact excerpts above; no frozen digest was generated. All runtime results, syntax/tooling compatibility, binary/source provenance, persistence effects, all-five protocol acceptance, clean shutdown, restart, graph and budget behavior remain unverified. The reported build003 receipt was supplied context, not independently examined or rerun here.
