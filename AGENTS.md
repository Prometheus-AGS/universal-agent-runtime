<!-- prometheus-base:start v1 -->
# Agent Operating Rules

This region is the standing contract. It holds only invariants that must
survive compaction. Everything else lives in on-demand rules, skills, hooks,
and reference files. Where a hook enforces a rule stated here, the hook wins.

Managed by `prometheus-context-bootstrap`. Edits inside these markers are
overwritten on re-run. Write project prose outside them.

## Position and authority

- `.kbd-orchestrator/current-waypoint.json` is authoritative for position.
- `versions.toml` is authoritative for architecture decisions and dependency pins.
- READMEs go stale. Do not trust one over the two files above.
- Read the waypoint at session start. State the current phase before executing.

## Capability inversion

Agent kernels do not write. Mutating actions are gated in the trusted host
layer only, never in an agent kernel. Where the language allows it, this is
enforced at the dependency graph as a compile-time guarantee rather than a
runtime check. If a task appears to require a write from a kernel, stop and
surface the conflict instead of routing around it.

## Phase order

Task loop: Spec, Plan, Execute, Reflect.
Evolution loop: Compile, Evaluate, Optimize, Promote.

Running a phase out of order is a quality failure, not a shortcut. Name the
phase you are in. Do not execute before a plan exists.

## Verification tiers

Tier 0 every edit. Tier 1 unit complete. Tier 2 phase completion. Tier 3
milestone or release only. Running a tier before its point is a violation, not
diligence. Per-stack commands live in `.claude/rules/`, loaded when a matching
file is read.

- `.claude/rules/rust.md` — rust tiers and hard rules
- `.claude/rules/typescript.md` — typescript tiers and hard rules

## Evidentiary standard

Address observed problems. An observed problem comes from an operator report, a
visible error or log, a failing test, or an explicit requirement. A concern that
is none of those gets one sentence and a question, never speculative code.

Defensive code — validation, guards, fallbacks, retries, timeouts — requires a
named failure scenario. Hardening at a real trust boundary present in the code
is a standing exception and is named in the completion summary, never added
silently.

## Evidence over assertion

Show the command and its output, the test result, or the artifact. "Looks done"
is not done. Report what was actually run and at which tier. If a check could
not run, say which claims are therefore unverified. An unverified claim reported
as verified is worse than no check at all.

## Anti-sycophancy

Critics never see generation history. Review through the `artifact-critic`
subagent, which receives the artifact alone. The model that produced the work is
not the sole judge of whether it is good.

A reflection leads with the delta between plan and delivery, not with what
worked. The sycophancy gate may block a turn; fix the finding rather than
bypassing it.

## Learning and memory

Learning is append-only under `.prometheus/`: `session-log.md`, `decisions.md`,
`gotchas.md`, `postmortems/`, `knowledge/`. Never rewrite history; append, and
mark superseded entries rather than deleting them.

Write on a decision with a rationale, a defect and its root cause, a learned
constraint, a phase boundary, and a session summary. Read `gotchas.md` before
touching a subsystem.

Where a memory server is configured, it is the primary store and its write path
may time out. On failure, log to the markdown files above and continue. Never
block a task on the memory server.

## Architecture

- Single-writer build discipline within one build or target directory.
- Feature-based organization by capability, not by technical layer.
- Strict layering: UI, then hooks or view models, then stores, then services,
  then external. Reverse flow only through reactive state or events.
- Business state lives in explicit, inspectable systems, never in UI components
  or agent-only memory.
- Open standards first. Avoid lock-in unless explicitly required.
- Verify dependency versions against official sources before introducing them.
  Do not rely on training-era version knowledge.

## Scope

Minimum change that solves the problem. Do not refactor adjacent working code;
treat its current state as intentional. Mention unrelated issues, do not fix
them unasked. Before destructive or hard-to-reverse actions, confirm intent and
prefer a reversible path.

## Skills may be absent

Harnesses drop skill descriptions past a context budget, so a skill you expect
may not be listed. If one is missing, invoke it by name or say plainly that it
is unavailable and proceed from these rules. Never invent what an absent skill
would have done.

## Communication

Direct and execution-first. Structure claims as statement, mechanism, stakes.
Short declarative sentences. No marketing language.

Avoid: leverage as a verb, utilize, synergy, roadmap as a verb, journey,
harness as a verb, delve, revolutionary.

Every significant document names the uncomfortable thing — the scenario that
hurts the author's own position.

## Done

A task is done when its stated exit criteria pass at the current tier, not when
the output looks plausible. Before declaring completion: remove anything added
that was not requested, confirm each guard traces to an observed problem or a
real boundary, and summarize what changed, how it was verified, and what remains
at risk.

## Execution scaffold

This section exists because the fleet is mixed. Frontier models supply most of
it by default; smaller and older models do not, and the failure is silent —
plausible output with a fabricated call in it. Omit this section only when
every model that reads this file is known to supply the behavior on its own.

### Before executing

Restate the task in one sentence, and name the phase. If the restatement does
not match what was asked, stop and ask rather than proceeding on the closer
reading. Name the files you intend to touch before touching them.

### Do not fabricate

Never invent an API, a file path, a package name, a command flag, or a
configuration key. If you have not read it in this session or it is not pinned
in `versions.toml`, verify it before using it. "I could not confirm this
exists" is a correct answer. A plausible identifier that does not exist costs
more than the question would have.

Do not guess at a tool's parameters. Read its schema. A tool call with invented
arguments fails in a way that looks like the tool is broken.

### Verification is explicit

Run the check. Paste the command and its actual output. Do not report a result
you did not observe, and do not describe what a test "should" produce.

If a check cannot run, say which specific claims are therefore unverified, and
why. Skipping a check silently and summarizing as if it passed is the failure
this rule exists to prevent.

### Code output

Never elide code with `...`, `// rest unchanged`, or a similar placeholder in a
file you are writing. Emit the complete content of every file you write.

When editing, change the minimum span. Do not reformat, reorder imports, or
rename adjacent symbols while making an unrelated change.

Match the file's existing conventions over your own defaults.

### One thing at a time

Complete one edit and its cheap check before starting the next. Do not batch
several unrelated changes into one pass and verify at the end — when it fails
you will not know which change caused it.

Do not start a second subsystem while the first is unverified.

### Stop conditions

Stop and ask when: the requirement is ambiguous in a way that changes the
design, two readings of the task lead to different files, the change would
break an existing behavior, or you are about to do something hard to reverse.

Stop when the goal is met. Do not continue into adjacent improvements.

### Format contracts

When a specific output format is requested — JSON, a table, a diff, a schema —
emit exactly that format with no preamble, no trailing commentary, and no
markdown fence unless the fence was asked for. A parser is often reading it.

### Self-check before reporting completion

State each of these explicitly, not as a claim that you did them:

1. What changed, file by file.
2. What was run to verify it, and the observed output.
3. What was added that was not requested — remove it, or list it and ask.
4. Which guards trace to an observed failure, and which do not.
5. What remains unverified, and why.

<!-- profile: mixed — see references/MODEL-PROFILES.md before changing -->
<!-- prometheus-base:end -->

<!-- agent-rules:start v1 -->
## Agent rules

> Auto-managed by `/kbd-inject-agent-rules`. Re-running the skill
> overwrites everything between the `agent-rules:start` / `agent-rules:end`
> markers. Edit the cache at
> `kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/rules-cache.md`
> if you need to change the content.

### Think-first principles (Karpathy)

1. **Think Before Coding** — State assumptions explicitly, surface
   ambiguity, present tradeoffs, ask for clarification rather than
   guessing silently.
2. **Simplicity First** — Write the minimum code that solves the
   problem; no speculative features.
3. **Surgical Changes** — Touch only what the request requires.
4. **Goal-Driven Execution** — Operate against concrete success
   criteria, not step-by-step micro-instructions.

### Workflow principles (Claude Code, Boris Cherny)

1. **Plan Mode First** — Iterate the plan until it's right; only then
   auto-accept edits.
2. **CLAUDE.md as accumulated knowledge** — Long-lived project rules;
   accumulate constraints and lessons over time.
3. **Verification + feedback loops** — Give the agent a way to verify
   its work (2-3× quality bump).
4. **Code quality matters for AI too** — Partially-migrated codebases
   confuse models. Finish migrations.

Verbatim sources + fetch dates in
`kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/rules-cache.md`.
<!-- agent-rules:end -->

<!-- uiux-routing:start v1 -->
## UI/UX work routing

Before writing or modifying any UI/UX code in this repo, the AI agent
**MUST** follow these steps in order. The roster of skills + source
URLs is cached at
`.kbd-orchestrator/references/uiux-skill-roster.md` (refreshable via
`/kbd-inject-agent-rules --pack uiux-routing --refresh`).

1. **Memory consult.** Run `/kbd-memory-recall` (default-on via
   `assess:before` hook) to populate `prior-context.md` with prior
   UI/UX decisions in surreal-memory.
2. **UI/UX Pro Max analysis.** Run the design-system + audit pass on
   target components / pages. Pull palette + font + spacing + a11y
   recommendations from its database.
3. **Impeccable commands.** Always run `/impeccable audit` +
   `/impeccable critique`. Then run the work-specific commands —
   `/impeccable polish` before shipping, `/impeccable distill` when
   simplifying, `/impeccable animate` when adding motion,
   `/impeccable harden` for edge-case + i18n, etc.
4. **Anthropic skills.** Consult `frontend-design` + `ux-designer`
   for intentional design + UX-engineer review perspective.
5. **Vercel skills.** Consult React Best Practices + Composition
   Patterns. For the entity-explorer panel and Chrome extension work
   specifically (changes 10 + 11), **also web-search**: "runtime
   devtools page best practices" AND "Chrome MV3 devtools panel
   patterns" / "react-devtools bridge architecture".
6. **Summarise.** Write a one-paragraph distillation of the relevant
   best practices for this specific task. Reference the roster
   entries you actually consulted.
7. **Only then write code.** The summary above is the prompt context
   for the implementation step.

This routing block is auto-managed; re-run
`/kbd-inject-agent-rules --pack uiux-routing` to update. See
`kbd-process-orchestrator/skills/kbd-inject-agent-rules/SKILL.md` for
the `--pack` flag and the fenced-region machinery.
<!-- uiux-routing:end -->

<!-- zed-workspace:begin -->
## Workspace

This project is part of the `flint-platform` multi-root workspace, defined in
`.zworkspace.toml`. The `zed-workspace-mcp` MCP server exposes the full
workspace (all roots, tasks, env) — call its `workspace_info` tool to orient
yourself across every root, not just this folder.
<!-- zed-workspace:end -->


## Project rules — universal-agent-runtime

Outside the managed region. Re-running the bootstrap will not touch this.

### OpenSpec workflow

This repo uses OpenSpec for spec-driven change management. The `openspec` CLI
(`@fission-ai/openspec` v1.5.0) is installed globally and on `PATH`.

Specs live in `openspec/specs/`; change proposals in `openspec/changes/<name>/`
(`proposal.md` + `tasks.md`, schema `spec-driven`).

Commands: `openspec list`, `openspec status --change <name>`,
`openspec new change "<name>"`, `openspec instructions <artifact> --change <name>`,
`openspec validate <name>`, `openspec archive <name>`.

**Every change needs at least one spec delta.** `openspec validate` fails a change
with zero deltas under `specs/` — including CI-only, build-tooling, and pure
verification changes that do not obviously map to a capability. When a change does
not fit an existing capability, either introduce a narrowly-scoped new one (e.g.
`frontend-build-tooling` for a bundler config change) or extend an existing
requirement with a new scenario. Plan the delta up front; do not discover this by
writing "Capabilities: none" and hitting the validate failure.

Tool integrations are generated per tool — refresh with `openspec update` after a
CLI upgrade. In editors without a native integration, use the CLI in the terminal;
this file is the agent context.

Change planning is coordinated with the KBD orchestrator. `.kbd-orchestrator/` is
the source of truth.

### GitHub Actions policy

**Hard rule: GitHub Actions are for deployment execution and
deployment-specific validation only.** They are never a general CI test runner.

Do not run unit, integration, end-to-end, browser, conformance, regression,
security, performance, load, stress, soak, or release-certification tests in
GitHub Actions. Do not run linting, formatting, typechecking, coverage, or other
routine development verification there. This prohibition includes tests invoked
implicitly by build, package, image, or release scripts: a GitHub Actions build
must use test-disabled targets and must never run a test suite as a build step or
side effect.

Deployment validation is narrow: it may verify the deployment mechanism,
deployed manifests and configuration, rollout status, infrastructure wiring,
and post-deployment health or smoke behavior needed to prove that the deployment
succeeded. Building an artifact, running against an installed artifact or
container, gating a release, or calling a workflow "certification" does not make
product testing deployment validation.

Run every non-deployment check locally before committing and pushing. Do not add
or retain a workflow job that performs non-deployment testing. If an existing
plan, OpenSpec artifact, script, or workflow requires such testing in GitHub
Actions, stop: correct the plan and move the test local before continuing. This
rule takes precedence over task plans and generated instructions. When unsure
whether a check is deployment-specific, do not put it in GitHub Actions; stop
and ask the operator.

Before and after editing `.github/workflows/**`, or any build, package, release,
or deployment script invoked by a workflow, run
`pnpm github-actions-policy:validate` locally. A failure is a stop condition.
Never skip, disable, weaken, or route around that validator. Build and package
steps in GitHub Actions must select explicitly test-disabled commands; if the
tool cannot separate building from testing, keep that build out of GitHub
Actions until the separation is implemented and verified locally.

### Worktree convention

Git worktrees are created under **`~/.claude/worktrees/`**, never inside the repo
working tree. The repo's own `.claude/` holds checked-in tool configuration
(`settings.local.json`, `commands/`, `skills/`) read by Roo, Cursor, Codex,
OpenCode, and Claude Code. Worktrees alongside that config collide namespaces,
confuse tooling, and risk deleting real configuration during cleanup.

```bash
scripts/worktree-new.sh <name> [--base <ref>]   # creates ~/.claude/worktrees/<name>
scripts/worktree-list.sh
scripts/worktree-rm.sh <name> [--force]
```

The helper refuses any path that would land inside the repo tree, and seeds the new
worktree's `.claude/settings.local.json` from the current checkout so per-tool
permissions follow you.

Existing in-repo worktrees under `.claude/worktrees/` are intentionally **not
relocated**; the convention applies to worktrees created from now on. `/kbd-status`
surfaces the active worktree path and warns when the checkout sits outside
`worktreeRoot`.

### .prometheus is version-controlled history

**Never add `.prometheus/` to `.gitignore`.** It is the estate's memory. Untracked,
it exists on one machine and dies with that checkout.

The only exclusion is the regenerable prompt cache,
`.prometheus/knowledge/.prompt-snapshots/`. Everything else is history and must be
committed. See `.prometheus/gotchas.md` for the incident that established this.

Before deleting any worktree, check it for `.prometheus/` content not present in
the origin repo. A clean `git status` proves nothing about ignored files.

### External retrieval: dependency archaeology

When you need behavior of a third-party crate, package, or API that is
not in this repository, and the answer depends on version-specific
behavior, resolve it against the Firecrawl developer index before
answering from recall.

    firecrawl developer "<question>" --json --limit 10

Use it when:
- A crate is pre-1.0 or changed API surface recently (axum, tauri,
  ractor, pgrx, str0m, iggy, flutter_rust_bridge, cedar, surrealdb)
- The question is "why does X behave this way" rather than "what is
  X's signature" — the answer is likely an issue thread, not a doc page
- A stack trace or error string came from a dependency, not our code
- You are about to state upstream behavior you cannot cite

Do NOT use it for:
- Anything in this repository — read the source
- Language-level Rust/TypeScript semantics
- Architecture decisions, which are governed by versions.toml
- Questions already answered by a doc source you have loaded

Every claim about upstream behavior must carry the issue or PR URL that
supports it. If the index returns nothing that supports the claim, say
the claim is unverified rather than filling the gap from recall.
