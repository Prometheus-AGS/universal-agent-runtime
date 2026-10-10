# BAUAR testing handoff — 2026-10-10

Publication follows the operator's explicit instruction to stop testing and commit, push, open a PR, and merge the completed session source. This is a source handoff, not release certification. No new test, formatter, compiler, package, or review gate runs accompany publication.

## Source and integration boundary

The UAR candidate checkout is `/Users/gqadonis/.claude/worktrees/bauar-release-uar`, branch `codex/bauar-release-integration-2026-10-09`. The prior scoped source checkpoint is `84ca0ffff5da8fafc1e2e7f5585efc07a396b14e`, based on `a7cb972992d4f83db6585449ea81af0fe4a1c990`. The remaining session amendment in `tests/bauar_full_harness_cursor.rs` records model-call counts and checks that identical admission retries preserve the original run/history while changed input under the same admission ID conflicts without another model call.

Fetched `origin/main` is `f43b6b1d1498faf1969e52518b5f7e9c9a673d04`. At handoff preparation, the candidate is 145 commits behind and one ahead. A normal merge of that main revision was attempted and aborted after substantive contract conflicts; the original source branch is being published for handoff. Prior runtime evidence does not certify an eventual merged source. The scoped checkpoint includes execution identity/authority, configured MCP resource credentials and stdio boundaries, approvals, secret/receipt projection, full-harness transport, and their supporting source/tests. Credential acquisition and custody remain application responsibilities; MCP configuration determines the resource credential. Caller JWT forwarding is not a substitute for that configuration.

Cancelled F6 work remains excluded: `tests/bauar_session_owner.rs` and `src/uar/mcp_server.rs` were not inspected, hashed, diffed, staged, or tested for this handoff. Reopening that work requires a separate operator instruction.

## Actual evidence before publication

The selected UAR regression completed with five nonempty target summaries, **31 passing tests and 15 named negative scenarios**, using `server-full,test-probes`, locked Cargo dependencies, and one test thread:

| Selected target | Coverage boundary |
| --- | --- |
| `bauar_identity_boundary` | Local/remote admission identity and receiving authority |
| `bauar_resource_grants` | Configured resource grants and receiver enforcement |
| `bauar_stdio_boundary` | Process/environment boundary |
| `bauar_secret_projection` | Model/event/receipt secret projection |
| `bauar_full_harness_cursor` | Contiguous replay cursor and exact admission retry |

These are historical results at the selected pre-publication candidate and runtime bindings. They are not a fresh test of the PR merge commit.

The corrected Bossfang harness separately passed **18 cases (13 main, five native kernel)** with six permitted effects, 36 real controlled model calls, and confirmed owned-process cleanup. The tested path is Bossfang → a private application-equivalent supervisor → packaged UAR. Native direct Bossfang principal transport is not certified by that fixture. The model provider, execution harness, and job orchestrator remain distinct roles. Restart is unsupported/unknown and requires reconciliation; the evidence does not claim durable replay or external exactly-once effects.

The selected H01–H04 adjudication is PASS, while the historical full integration is FAIL. The desktop packaged acceptance attempt waited on native Keychain initialization for approximately 30 minutes before an operator-directed interruption. Supplemental cleanup confirmed owned processes absent; that is not desktop acceptance PASS. No full-phase, signing, notarization, installed-app, Intel, Windows, production remote IdP/receiver, or broad feature-matrix certification is claimed.

## Durable evidence references

The local phase evidence root (not included in this repository) is:

`/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/.kbd-orchestrator/phases/phase-bauar-release-acceptance`

| Artifact relative to that root | SHA-256 / outcome |
| --- | --- |
| `acceptance/candidate-inputs.json` | `758ae99202cfd6a916bc0bb934a6feabb9a4a0247b437070854edb3403fc8700` |
| `evidence/execute/attempts/runtime-seal-05.json` | `688ac5d9e6108457cd9f44b294e0cc2e5de6fbd99042a1f1b508a73c69b35af4` |
| `evidence/execute/harness-adjudication-02.json` | `b39f89606e792d23fd4bfa470e3554f17f865bad217d270e3e51d0dfd3c8a4ef`; selected PASS, overall FAIL |
| `evidence/execute/harness-adjudication-invocation-02.json` | `bd64ebbcd761a4348c9c9c1214b88cc2afe4a512387ef38a80e605b2b504feaa`; actual exit 0 |
| `evidence/execute/attempts/execution-0a50eef88a9cb91020cb5361/integration-e58a9cf8-f6db-40b2-aff2-e4e65af25b16.json` | `58bdcf0b828f24eeb32eb7b19f47f6659d6daa434f6eefb3a599127e3519ce74`; harness-only PASS |
| `evidence/execute/attempts/execution-0a50eef88a9cb91020cb5361/integration-bb652021-9b7f-4405-94d6-e868db1c7910.json` | `e8101761af8ef49790307b70effd8fda7cad81281813d8e733413e797030e8a8`; full integration FAIL, six of eight components PASS |
| `evidence/execute/owned-desktop-cleanup-reconciliation-01.json` | `1dcc6f24703adc2021053212dc88ca728447f610cc883a6b2ceb8bef9d32c4a4`; owned processes absent |

Private profiles, environment values, credentials, raw logs, compiler caches, and binaries stay outside the committed source handoff. Failure history is retained in phase evidence.

## Work for the next testing session

1. Checkout the published PR/merge commit and record its exact SHA and the final cross-repository candidates. Resolve dependencies from `versions.toml` and existing lockfiles without changing pins.
2. Reconcile the merged UAR source with the five-target evidence above. Upstream integration invalidates claims that the old run alone certifies the new head. When testing is authorized again, run the selected real-path targets with the approved features, locked dependencies, and one writer per build directory.
3. Resolve the actual desktop Keychain initialization blocker, then complete the remaining packaged desktop behavior. Preserve prior passing component results only when their source, command, runtime, and package bindings still match; rerun invalidated or failed scopes.
4. Complete the deferred review, produced-artifact QA, and release-specific checks at the appropriate boundary. Report unavailable platforms and unexercised production credentials as unverified. Do not reopen cancelled F6 implicitly.
5. Keep the original FAIL receipts and distinguish historical selected PASS from fresh merged-source/full release acceptance. Do not advance Reflect or claim certification merely because the source PR merges.

Publication omits local Lefthook validation using the hook's existing `LEFTHOOK=0` switch, honoring the operator's stop-tests instruction. Hook configuration is unchanged. The omitted pre-commit policy validation and other deferred checks must not be reported as passing.

## Observed merge block

The normal merge of origin/main stopped with eight content conflicts in eligible paths: `src/uar/api/full_harness/handlers.rs`, `src/uar/api/routes.rs`, `src/uar/runtime/manager.rs`, `src/uar/runtime/thread/approvals.rs`, `src/uar/runtime/tool_admission/lifecycle.rs`, `src/uar/runtime/tool_admission/mod.rs`, `src/uar/security/sidecar_guard.rs`, and `src/uar/security/verifier/mod.rs`. No cancelled F6 conflict was reported. No conflict resolution or new product change was attempted, and `git merge --abort` restored the committed candidate.

The newer main introduces durable approval records/resolution, delegated host-context run admission, representation/admission-owner cancellation semantics, and algorithm-bound JWKS keys. The candidate cache returns a `DecodingKey`; newer main verification expects a key carrying its allowed algorithm. Choosing either conflict side would lose material behavior. Combining these contracts requires a separately authorized forward-port and fresh local verification after the operator resumes testing. The publication preserves the completed source rather than treating the aborted integration as successful.
