PLAN: handle-kbd-uninitialized-runtime
Project: universal-agent-runtime
Date: 2026-08-25
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-kbd-uninitialized-runtime: Make a registered empty KBD runtime accept its first typed mutation safely.
   - Scope: upstream kbd-runtime integration | prometheus CLI | OpenSpec | UAR submodule pin | local installation
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: In an external prometheus-skill-system worktree based on the pinned rollover commit, add an OpenSpec capability for registered-runtime initialization. Reuse the existing legacy-aware initializer for mutating commands, correct the status hint, and add process-level tests for successful first mutation plus non-zero command rejection. Build and install the CLI, prove the behavior against an isolated registered runtime, then pin the exact upstream commit in UAR with a UAR OpenSpec delta and verification evidence.

EXECUTION ROUND ORDER
Round 1: fix-kbd-uninitialized-runtime

TASK ORDER
1. Create `~/.claude/worktrees/fix-kbd-uninitialized-runtime` on upstream branch `codex/fix-kbd-uninitialized-runtime`, based on `f1e58b25b0a9926c24d1bb0ddb6c0678d16c6f49`; do not edit the UAR submodule checkout.
2. Create upstream OpenSpec change `initialize-kbd-runtime-on-first-mutation` with a new `kbd-runtime-initialization` capability covering automatic initialization, legacy-state preservation, actionable status, and non-zero rejected commands.
3. Update the CLI so only commands that need to mutate canonical state initialize an empty registered runtime. Keep read-only status non-mutating, remove the speculative `migrate --apply` hint, and include the runtime path in initialization failures.
4. Add focused unit/process tests proving status guidance, first-mutation initialization, history-compatible projection, idempotent initialization, and non-zero exit on a rejected typed command.
5. Run strict OpenSpec validation, focused CLI/runtime tests, formatting, Clippy, and the affected workspace build.
6. Commit and push the upstream review branch, build and install the updated `prometheus` CLI, and run an isolated installed-binary proof. The daemon is unchanged and does not require restart.
7. Create UAR OpenSpec change `fix-kbd-uninitialized-runtime`, extend `kbd-phase-inventory-governance`, pin the exact upstream commit, record evidence under `.prometheus/`, validate, commit, push, and open a UAR PR that links issue #265.
8. Close issue #265 only after the repository fix is merged; otherwise leave it open with the review links and exact remaining action. Preserve the issue as audit history rather than deleting it.

EXPLICIT SCOPE CUTS
- Do not add Unix-socket transport; the canonical local fallback remains the correctness path and issue #265 marks socket transport optional.
- Do not initialize during registration or read-only status; registration remains identity/replica bookkeeping and status remains non-mutating.
- Do not alter migration semantics beyond removing the irrelevant status recommendation.

COMMANDS TO RUN
/opsx:new fix-kbd-uninitialized-runtime

PLAN COMPLETE
