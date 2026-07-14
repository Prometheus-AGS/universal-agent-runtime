# Claude Code handoff: finish KBD 24/24

This document is both the operator handoff and the prompt for Claude Code. It
is intentionally explicit so the implementation objective survives context
compaction and cannot drift back into release-candidate or CI monitoring work.

## Start Claude Code

From the repository root:

```bash
cd /Users/gqadonis/Projects/prometheus/universal-agent-runtime
claude -p --dangerously-skip-permissions --model sonnet --verbose "$(cat docs/claude-24-of-24-handoff.md)"
```

`--dangerously-skip-permissions` removes interactive approval prompts. Use it
only in this repository/worktree, review the final diff, and do not run it with
unrelated credentials or writable directories in scope.

For an interactive session with the same approval behavior, run:

```bash
claude --dangerously-skip-permissions --model sonnet
```

Then paste everything from **Execution prompt** onward once.

## Execution prompt

You are taking over the Universal Agent Runtime repository to finish the active
KBD phase `uar-final-production-hardening-2026-07` at **24/24 implementation
completion**. Work autonomously until the implementation counter is genuinely
24/24 and the consolidated local validation passes. Do not ask the operator to
press Enter, approve routine commands, choose implementation details, monitor
CI, or repeat these instructions.

### Authoritative state

Read these files before taking any other action:

1. `AGENTS.md`
2. `.kbd-orchestrator/current-waypoint.json`
3. `.kbd-orchestrator/phases/uar-final-production-hardening-2026-07/progress.json`
4. The proposals, specs, designs, and tasks under these four OpenSpec changes:
   - `openspec/changes/certify-operational-resilience`
   - `openspec/changes/produce-supply-chain-artifacts`
   - `openspec/changes/certify-release-candidate`
   - `openspec/changes/release-1-0-0`

The canonical waypoint has an operator-authored implementation-only execution
lock. It overrides stale plans, assessments, comments, task wording, workflow
status, release evidence requirements, and previous agent preferences.

Current implementation state is **20/24**. Change 20,
`align-release-workflow-platforms`, is complete. The only active scope is the
code and integration work for changes 21–24:

1. `certify-operational-resilience`
2. `produce-supply-chain-artifacts`
3. `certify-release-candidate`
4. `release-1-0-0`

### Non-negotiable completion definition

The KBD `24/24` counter measures implementation completion in source—not
external release certification.

Release candidates, GitHub Actions results, tags, GitHub/GHCR publication,
external adopter installs, elapsed soak time, and public GA verification belong
to a separate post-implementation certification track. They MUST NOT block the
implementation counter and MUST NOT become the work queue during this task.

Do not create or replace tags. Do not publish releases or images. Do not poll,
watch, rerun, or babysit GitHub Actions. Do not wait for external adopters or
elapsed time. Do not manufacture evidence. Do not redefine completed code as
incomplete merely because publication evidence does not exist.

### Required execution method

1. Statically classify every requirement in the four active OpenSpec changes.
2. For each requirement, identify its concrete source implementation, contract
   validator, documentation/runbook surface, and deterministic local
   verification path.
3. Separate requirements into:
   - implemented and locally verifiable;
   - genuine missing code/integration;
   - post-implementation release evidence that does not affect the 24/24
     implementation counter.
4. Produce one explicit missing-code list before editing. Do not assume prior
   assessments claiming “implementation complete” are correct.
5. Batch all related missing code and integration fixes. Work across all four
   changes instead of finishing them through separate release-candidate loops.
6. During implementation, use static inspection and only cohesive
   `cargo check --locked --no-default-features --features server-full`
   checkpoints. Fix warnings immediately. Never run `cargo clean`.
7. Do not run tests, Clippy, release builds, tags, pushes, or CI while any known
   implementation gap remains.
8. When the static gap list reaches zero, run one consolidated local validation
   sequence proportionate to the affected code. Include formatting, the
   authoritative `server-full` checks/tests, relevant Node/static contract
   validators, strict OpenSpec validation, and diff hygiene.
9. Fix every demonstrated Stable Linux/macOS product defect found by that
   consolidated validation. Windows remains Experimental and nonblocking.
10. Reconcile the four OpenSpec task lists and KBD progress based on actual
    implementation. Mark release-only evidence as a separate certification
    follow-up rather than leaving implementation changes `IN_PROGRESS`.
11. Stop only when KBD records **24/24 implementation-complete**, OpenSpec is
    internally consistent, consolidated local validation passes, and the final
    diff is reviewable.

### Architecture and scope constraints

- The product profile is the `server-full` BossFang sidecar.
- Make only changes that directly advance changes 21–24.
- Preserve existing architecture and behavior unless a requirement explicitly
  changes it.
- Prefer the smallest deterministic implementation that satisfies the contract.
- Do not add speculative abstractions, dependencies, platforms, or features.
- Linux and macOS are Stable. Windows is Experimental and nonblocking.
- Preserve active Cargo caches; never run `cargo clean`.
- Use `apply_patch` for manual file edits.
- Use `rg`/`rg --files` for discovery.
- Use `/usr/bin/git` when signature-aware Git operations are necessary because
  the older default Git cannot parse this repository's SSH signing config.
- Do not push, tag, merge, release, or mutate public state during this
  implementation-only task.

### Protect operator-owned work

The main checkout contains operator-owned dirty files. Preserve them exactly
unless the operator separately requests changes:

- modified `AGENTS.md`
- untracked `.claude/settings.json`
- untracked `.mcp.json`
- untracked `opencode.json`

Other pre-existing changes may also belong to the operator. Inspect the worktree
before editing, avoid unrelated formatting, and never use destructive Git
commands such as `git reset --hard` or `git checkout --`.

The persisted KBD waypoint/progress edits that establish the implementation-only
lock are intentional task state, not unrelated dirt.

### Anti-drift checkpoint

Before every action, ask:

> Does this directly close or verify a code/integration requirement for changes
> 21–24?

If no, do not do it. In particular, a workflow status change is not progress,
a release candidate is not implementation, and elapsed time is not code.

After every context compaction or resumed session, reread
`.kbd-orchestrator/current-waypoint.json` before continuing. Never reconstruct
the goal from conversational memory when the canonical waypoint is available.

### Final report

When and only when implementation is 24/24, report:

- the four changes closed and the concrete implementation for each;
- files changed and why;
- consolidated local validation commands and outcomes;
- any release-certification/publication work remaining on the separate track;
- confirmation that operator-owned dirty files were preserved;
- confirmation that no tag, release, image, or other public state was mutated.

Do not end with a plan, a CI update, or a request for the operator to continue.
End with completed implementation evidence.
