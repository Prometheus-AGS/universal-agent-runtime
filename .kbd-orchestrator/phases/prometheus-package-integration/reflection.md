# Phase Reflection: prometheus-package-integration

**Project:** universal-agent-runtime
**Date:** 2026-07-06
**Phase completion:** 100% of the redesigned scope (3/3 changes); 12/14 of the original 2026-05-26 plan's items were already done before this session resumed, #13 confirmed done separately, #4 fully replaced
**Changes completed:** 3 / 3 (redesigned plan); originally 14 items, later corrected to a 3-change replacement for item #4 only

## What diverged, and why (leading with this per this project's own reflect convention)

This phase's defining characteristic was **repeated correction of its own premises**, not smooth execution against a stable plan. Three separate times, a claim carried forward from the original 2026-05-26 plan or made earlier in this same session turned out to be wrong on direct inspection, and each time the scope had to be narrowed or redirected as a result:

1. **#13 (`dockerfile-multistage-with-submodules`) was wrongly assessed as an unfinished gap.** A first-pass check (`grep -c "^FROM"` on the Dockerfile) was misread as "not the polyglot 5-toolchain design." Direct content inspection then showed the opposite: all 5 toolchains (Rust/Node/Python/Go/wasmtime) are installed, all 5 declared volumes are present, and the image is wired into 5 `docker-compose*.yaml` files and 5 GitHub Actions workflows including `deploy.yml` and `release.yml`. The user had already directed a redesign of "both #4 and #13" based on the wrong initial read; once corrected, the user confirmed leaving #13 alone. **Root cause**: a cheap structural check (`grep -c`) was treated as sufficient evidence for a "is this fully implemented" judgment, when it only proved "how many stages exist," not "what's in them."

2. **`surreal-memory-server` and `liter-llm`, two of the original plan's three "managed binaries," are not separate spawnable processes at all.** Both are linked Rust libraries (git dependencies in `Cargo.toml`) used in-process — `surreal_memory::Memory`/`MemoryStorage` throughout `src/uar/memory/`, `liter_llm::DefaultClient` in `src/llm/liter_driver.rs`. The original plan's own text had already hedged this ("surreal-memory-server, *if we ever spawn it standalone*"), which should have been a stronger signal to verify before scoping the redesign around all 3 "binaries." **Root cause**: carrying forward an old plan's noun ("binary") without re-verifying it still describes current architecture.

3. **`wire-toolchain-provisioning`, planned as Round 2's second change, had no real integration point.** The plan assumed UAR's code compiles skills from source and needs toolchain resolution to do so. It doesn't — `wasm_runtime.rs` only loads already-built `.wasm` files, and the Dockerfile's own comment says its toolchains are kept resident "for user builds" (a human manually building their own skill inside the container). Rather than force a fake wiring into a nonexistent call site, the toolchain `ToolSpec`s were kept as ready-to-use, tested recipes with no forced caller, and the plan's change count was reduced from 4 to 3 — confirmed with the user, not decided unilaterally.

Each of these was caught by direct code verification (grep, read, checking actual call sites) rather than trusting a document's prose, and each was surfaced to the user for confirmation before the scope changed. None were silently absorbed or glossed over.

## Goals

| Goal | Status | Notes |
|---|---|---|
| Verify all 14 originally-planned `prometheus-package-integration` items | MET | 12/14 confirmed done via direct code inspection (several delivered via an evolved "direct per-entity hooks" design — `useAgents`/`useKnowledgePage` — rather than the originally-planned generic `useEntityList` calls, a positive divergence). #13 also confirmed done (see correction #1 above). Only #4 was a genuine gap. |
| Redesign #4 (`binary-instance-discovery`) per the user's "use git, install necessary tools" direction | MET | Delivered `src/uar/orchestrator/provisioning.rs`: a pluggable `ToolProvisioner` with Adopt → NativePackageManager → GitInstall → PrebuiltBinary fallthrough, gated behind `ProvisionOptions::allow_install` so it never silently modifies the host. Scope corrected twice during assessment (see above) before landing on: MCP stdio server command resolution (the one real, currently-spawned dependency) + 5 toolchain recipes (no forced caller). |
| Cross-platform support equally robust on Linux/macOS/Windows | PARTIAL | The provisioning module's logic (`cfg!(target_os = ...)` branches for Adopt/NativePackageManager/PrebuiltBinary) is written for all 3 OSes and unit-tested for the OS-selection logic itself, but only actually *exercised end-to-end* on Linux (this session's environment). The macOS/Windows branches (`brew`, `winget`/`choco`, `.exe` `where` lookup) are correct by inspection and code review, not by a real macOS/Windows CI run — no such run was available this session. Disclosed here rather than claimed as fully verified. |
| Do not touch database engines (SurrealDB/Postgres) provisioning | MET | Explicitly out of scope per the user's own `AskUserQuestion` answer; nothing in `provisioning.rs` references either. |
| Leave `process_supervisor.rs` and the Dockerfile alone unless a real need was found | MET | Neither was modified. `process_supervisor.rs` was read in full and confirmed to have zero current call sites (a different problem shape — TCP-service adoption — with no current target); the Dockerfile was confirmed already-correct and the user directed leaving it as-is. |

## Delivered Changes

- `provisioning-strategy-core` (commit `555252d`) — new `src/uar/orchestrator/provisioning.rs`: `ToolSpec`, `ToolProvisioner::resolve()` with the 4-strategy fallthrough, `ProvisionOptions::allow_install` gate. 7 unit tests. (by: claude-code)
- `wire-mcp-server-provisioning` (commit `b99cab7`) — wired into `mcp/registry.rs`'s stdio spawn path (fast-path preserved via new `is_on_path()`), curated `kreuzberg` `ToolSpec`, plus the 5 toolchain recipes (`skill_toolchain_specs()`) folded in from the reduced-scope `wire-toolchain-provisioning`. 6 more unit tests (13 total). (by: claude-code)
- `provisioning-tests-and-docs` (commit `9489cb9`) — `docs/PROVISIONING.md`, a D-E entry in `docs/ARCHITECTURE.md`, and an `#[ignore]`d end-to-end test closing the `GitInstall` strategy's disclosed test gap (local offline git fixture, no network). (by: claude-code)

Total: 3 commits, 3 OpenSpec change directories (not archived — this project's established, disclosed pattern is to verify directly via `cargo check`/`test`/`clippy` rather than run `/opsx:verify` + `/opsx:archive`, per the decision record from `uar-security-deps-and-hygiene`).

## Technical Debt

- **Native-package-manager installation and prebuilt-binary download/extract are not exercised end-to-end by an automated test.** Doing so would install real packages or download real archives against whatever host runs the test suite — not acceptable for CI. Disclosed explicitly in `docs/PROVISIONING.md`'s "Non-goals" section and in both `provisioning-strategy-core`'s and `provisioning-tests-and-docs`'s OpenSpec proposals, not silently left as an assumed-covered gap.
- **macOS/Windows branches are code-reviewed, not live-tested** (see Goals table above) — this session's environment is Linux-only.
- **`known_tool_spec()`'s fallback case leaks memory** (`Box::leak` for uncurated MCP command names) — a small, one-time, bounded leak per unique MCP server name (`mcp.json` entries are loaded once at startup, not a per-call or user-scale input), documented inline as an accepted trade-off, not an oversight.
- **The original `integration-tests-and-docs` (#14)'s remaining scope from before this session** (system tests for live-bus latency and the builtin-skill-409 contract, noted as still-missing during assessment) is **not** addressed by this phase — it was out of scope for the redesigned #4 work and was not re-scoped in. Worth flagging explicitly for a future phase rather than letting it quietly disappear now that #4 is closed.

## Architecture Integrity

- AGENTS.md / Prometheus Base Rules violations: NONE identified. Rule 8 (Minimize Irreversible Actions) was applied directly in the design — `ProvisionOptions::allow_install` defaults to `false` specifically so this module cannot silently install packages, clone-and-build third-party source, or download-and-extract archives without explicit opt-in. Rule 27 (No Silent Dependency Introduction) was applied by choosing to shell out to the OS's native `tar`/`git` rather than adding new archive-handling or `which`-equivalent crates.
- Constraint violations: NONE. `cargo clippy --lib` held at 502 warnings (this session's established pre-existing baseline) across all 3 changes — verified via explicit A/B comparison after each round, not assumed.

## Cross-Tool Coordination Notes

- Progress tracking: RELIABLE. `progress.json`'s `changes_total`/`changes_completed` were kept in sync with the actual delivered scope at each correction point (14 → resumed at 12/14 confirmed done → redesigned-#4 plan of 4 changes → reduced to 3 changes mid-execution), rather than left stale against an outdated count.
- Handoff quality: CLEAR. `handoffs/assess.handoff.json`, `plan.handoff.json`, and `execute.handoff.json` were all written this session (the phase had none before, being a legacy resume) and each explicitly named the corrections made so far, so a different tool resuming mid-phase would not have re-trusted the same wrong assumptions.
- Recommendation: when resuming a long-stale phase (this one sat untouched for over a month), budget real time for re-verifying the original plan's factual claims against current code before scoping new work from it — this phase needed three separate corrections, and a cheaper initial check (e.g., `grep -rn "Command::new\|spawn(" ...` for each named "binary" before writing the redesign scope) would have caught corrections #2 and #3 before the user was asked to make decisions based on the wrong premise.

## Lessons Learned

- A structural/count-based check (`grep -c`) is not sufficient evidence for a completeness judgment — it can prove "how many things exist" without proving "what's actually in them." Use direct content inspection before asserting a file/feature is or isn't done.
- When a plan document names an architectural entity (e.g., "the liter-llm proxy," "surreal-memory-server"), verify with a live grep for its actual spawn/call sites before scoping work around it — plan prose can describe intent or an earlier architecture, not current reality, and can go stale silently.
- Hedged language in an old plan ("if we ever spawn it standalone") is a signal, not noise — when planning work that treats a hedge as settled fact, that's exactly the claim to re-verify first.
- Folding a reduced-scope change into an adjacent one mid-execution (this phase's Round 2) is preferable to either forcing a fake integration point just to preserve the original plan's shape, or silently dropping the item — both the user and the audit trail should see the reduction explicitly, which they did here (via `AskUserQuestion` + updated `progress.json`/`plan.md` notes).

## Next Phase Focus

Recommend closing this phase and starting fresh (no obvious single follow-on name yet — the user should pick focus based on current priorities). Top 3 candidate areas surfaced by this phase's own findings, in case a next phase picks one up:

1. **Original #14's carried-over gap**: system tests for live-bus latency and the builtin-skill-409 contract, still not found in the codebase as of this session.
2. **Live cross-platform verification of `provisioning.rs`**: a macOS and/or Windows CI run (or manual test) exercising the `brew`/`winget`/`choco` branches for real, closing the "code-reviewed, not live-tested" gap noted above.
3. **Dependabot/security posture**: the broader KBD history (`uar-security-deps-and-hygiene`) found and resolved a 96-alert backlog as of 2026-07 — worth a fresh check given time has passed since that phase closed, per this project's own established pattern of re-verifying rather than assuming security posture stays static.

## Context for Next Phase

Use this file, plus `assessment.md` and `plan.md` (both in this same directory), as prior context for the next `/kbd-assess` invocation. `plan-2026-05-26-original.md` and `assessment-2026-05-26-original.md` preserve the phase's original, pre-correction scope for historical reference.

## Sycophancy Self-Check

The `sycophancy-correction` MCP skill's `analyze_reflect_phase` tool was invoked but returned an unrecoverable error (`HTTP 401 Unauthorized: token_invalidated` from its local backend at `localhost:8181`) — not the documented "stubbed rewrite" caveat, but a full outage of its LLM invocation path. Applied the same three checks manually instead of skipping them:

- **S-08 (Reflect Phase Inversion)**: This reflection opens with "What diverged, and why" — three named corrections and their root causes — before any goals/success table. It does not open with "successfully completed" framing.
- **S-03 (Caveat Collapse)**: Surfaces 4 concrete technical-debt items, one explicit `PARTIAL` goal (cross-platform live-testing), and 2 disclosed test-coverage gaps (native-package-manager/prebuilt-binary strategies not exercised end-to-end; original #14's live-bus/409 tests still missing).
- **S-02 (Agreement Without Grounding)**: Each Goals-table entry cites specific evidence (commit hashes, file paths, verification commands run) rather than echoing the plan's expected outcome as if it were automatically true.

No re-invocation was possible given the tool outage; this manual pass is the final check for this reflection.
