
# AGENTS.md - Repository Guidelines

## Active production-completion execution lock

For the active KBD phase `uar-final-production-hardening-2026-07`:

- The primary objective is 24/24 production completion for the `server-full` BossFang sidecar.
- Operator instructions override stale plans, assessments, workflow status, and agent preferences.
- Before every action ask whether it directly advances changes 20–24; if not, do not do it.
- CI and tests are asynchronous evidence, not the work queue. Never babysit workflows while actionable implementation or release work remains.
- Batch related fixes. During implementation use static inspection and cohesive `cargo check` only; validate the completed product in one consolidated sequence.
- Linux and macOS are Stable. Windows is Experimental and nonblocking for this round.
- Keep implementation, evidence, time-bound conditions, and operator authorization explicitly distinct.
- Preserve active Cargo caches; never run `cargo clean`; use only reviewed reversible cleanup.

The canonical active state is `.kbd-orchestrator/current-waypoint.json`. Historical KBD detail remains in Git history and must not override it.


## Build, Lint & Test

- **Rust implementation checkpoint**: `cargo check --locked --no-default-features --features server-full`
- **Rust final validation**: `cargo fmt --all -- --check`, supported-profile tests and release certification only after implementation is complete
- **Web**: `pnpm -C frontend install --frozen-lockfile`, `pnpm typecheck`, `pnpm lint`, `pnpm build`
- **Test All (final validation only)**: `cargo test --locked --no-default-features --features server-full`, `pnpm test`
- **Single Test**: `cargo test <test_name>`, `pnpm -C frontend test <file_pattern>`
- **Clean Build**: ZERO warnings/errors allowed. Fix warnings immediately or use `#[expect(lint, reason="...")]`.

## Code Style & Conventions

- **Rust**: 4-space indent. `snake_case` (fn/mod), `CamelCase` (types). Use `anyhow` for app errors, `tracing` for structured logs.
- **TypeScript**: 2-space indent, semicolons, TS 5.9.3. React code lives in `frontend/src/` and follows the strict layering contract below.
- **Imports**: No glob re-exports (`pub use foo::*`). Use `#[doc(inline)]` for public re-exports.
- **Documentation**: Public items must have `///` docs with `# Examples`, `# Errors`, and `# Panics` sections.

## Architecture & UI

- **Structure**: `src/` (Axum runtime/API), `frontend/` (React 19/TypeScript), `static/` (bundled production assets).
- **UI contract**: React 19 is the authoritative first-party UI; historical HTMX/Web Component material is not present-tense product guidance.
- **Config**: `.env` (see `.env.example`), `example.config.yaml`, `mcp.json` for MCP tools.
- **LLM**: All LLM access goes through [liter-llm](https://github.com/GQAdonis/liter-llm) — 142+ providers via unified `provider/model` addressing. Set `UAR_LLM__MODEL` and `UAR_LLM__API_KEY` (or a provider shortcut like `OPENAI_API_KEY`). See `example.config.yaml` for the full `llm:` section.
- **Model routing**: Use `POST /api/uar/route` with capability requirements (`needs_tools`, `needs_vision`, `min_context`, etc.) to get the best available model. The catalog is built at compile time from models.dev + liter-llm schemas.

### React frontend (`frontend/`)

Strict layering — do not skip layers:

1. **Components** never call `fetch`, never import Zustand stores directly, and never import `frontend/src/services/`. They only render UI and call **hooks**.
2. **Hooks** only subscribe to stores and expose store actions; they do not call `fetch` or import service modules. Service calls live inside **stores**.
3. **Stores** (`frontend/src/stores/`) hold state and call **services** for HTTP/SSE and other I/O.
4. **Services** (`frontend/src/services/`) are thin wrappers around `fetch` / streams. **Only stores import services** (not hooks or components).

This avoids duplicated data logic, keeps ESLint `react-hooks/*` rules satisfied, and makes testing straightforward.

## OpenSpec workflow

This repo uses **OpenSpec** for spec-driven change management. The `openspec` CLI
(`@fission-ai/openspec`, v1.5.0) is installed globally and on `PATH`.

- **Specs** live in `openspec/specs/`; **change proposals** in `openspec/changes/<name>/`
  (`proposal.md` + `tasks.md`, schema `spec-driven`).
- **Common commands**: `openspec list`, `openspec status --change <name>`,
  `openspec new change "<name>"`, `openspec instructions <artifact> --change <name>`,
  `openspec validate <name>`, `openspec archive <name>`.
- **Every change needs at least one spec delta.** `openspec validate` fails a
  change with zero deltas under `specs/`, even for CI-only, build-tooling, or
  pure-verification changes that don't obviously map to a "capability." When
  a change genuinely doesn't fit an existing capability, either introduce a
  narrowly-scoped new one (e.g. `frontend-build-tooling` for a bundler config
  change) or extend an existing requirement with a new scenario relevant to
  the change (e.g. extending `dependency-security-posture`'s
  `CI Trigger Actually Fires` requirement for a live-verification change).
  Don't discover this by writing "Capabilities: none" and hitting a validate
  failure — plan the delta up front.
- **Tool integrations** (slash commands / skills) are generated per tool — refresh
  with `openspec update` after a CLI upgrade. First-class tools include Claude Code
  (`/opsx:*`), Codex, OpenCode, Cursor, Windsurf, Gemini, RooCode, Kilo Code, Antigravity.
- **Editors without a native integration** (e.g. **Zed**): use the `openspec` CLI in the
  integrated terminal; this `AGENTS.md` is the agent context.
- Change-planning is coordinated with the KBD orchestrator — `.kbd-orchestrator/` is the
  source of truth (see the Agent rules block below).

## Worktree convention

Git worktrees for this repository are created under **`~/.claude/worktrees/`** — never inside the repo working tree. The repo's own `.claude/` directory holds checked-in tool configuration (`settings.local.json`, `commands/`, `skills/`) that is read by Roo, Cursor, Codex, OpenCode, and Claude Code; putting worktrees alongside that config collides namespaces, confuses tooling, and risks accidental deletion of real configuration during cleanup.

Always create a new worktree with:

```bash
scripts/worktree-new.sh <name> [--base <ref>]   # creates ~/.claude/worktrees/<name>
scripts/worktree-list.sh                        # show worktrees under that root
scripts/worktree-rm.sh <name> [--force]         # remove a worktree under that root
```

The helper refuses any path that would land inside the repo tree and seeds the new worktree's `.claude/settings.local.json` from the current checkout so per-tool permissions follow you.

Existing in-repo worktrees under `.claude/worktrees/` are intentionally **not relocated** — the convention applies to every worktree created from now on. The KBD orchestrator surfaces the active worktree path via `/kbd-status` and warns when the current checkout is outside `worktreeRoot` (configured in `.kbd-orchestrator/project.json`).


/Users/gqadonis/.rvm/scripts/rvm: line 29: /bin/ps: Operation not permitted
pyenv: cannot rehash: /Users/gqadonis/.pyenv/shims isn't writable
## Prometheus Base Rules Set

> Canonical base rules for Claude Code, Codex, OpenAI agents, Gemini CLI, Roo,
> Cline, Kilo Code, Librefang, and all Prometheus/UAR-compatible development
> agents. **Mirrored in [CLAUDE.md](CLAUDE.md) — update both files together.**
> These define how agents reason, code, modify files, and interact with human
> operators. The auto-managed "Think-first principles" block below is a subset;
> this section is the full canonical set. Per **Rule 26**, project-specific
> sections may add *stricter* requirements but may not contradict these.

### Reasoning & execution

**1. Think Before Coding** — Do not assume; do not hide confusion; surface
tradeoffs before implementation. State assumptions explicitly. If uncertain,
ask. If multiple interpretations exist, present them. If a simpler approach
exists, say so. If something is unclear, stop and ask.

**2. Simplicity First** — Write the minimum code that solves the problem. No
features beyond what was requested, no speculative abstractions, no unnecessary
configurability, no unrequested future-proofing, no overengineering. If 50 lines
solve it, do not write 200.

**3. Surgical Changes** — Touch only what is necessary. Do not refactor or
reformat unrelated code. Match existing conventions. Remove only artifacts you
created. Mention unrelated issues; do not fix them unless asked.

**4. Goal-Driven Execution** — Define success criteria first. Convert vague
requests into testable outcomes. Verify completion; run tests where available.
Stop only when success criteria are satisfied.

**5. Truth Over Fluency** — Never prefer a confident answer over a correct one.
Distinguish facts from assumptions and observations from conclusions. State
uncertainty explicitly. Do not invent APIs, functions, files, packages,
commands, or behavior. If something is unknown, say so plainly.

**6. Evidence Before Conclusions** — Cite evidence, show the reasoning path,
explain tradeoffs and why alternatives were rejected. Prefer primary sources,
source code, tests, official docs, or direct observation over guesses.

**7. Preserve User Intent** — Optimize for the user's actual goal, not your
preferences. Do not silently expand or reduce scope. Clarify when requirements
conflict. Preserve the user's architectural direction unless told otherwise.

**8. Minimize Irreversible Actions** — Before destructive or hard-to-reverse
actions: confirm intent, explain consequences, prefer reversible approaches,
create rollback paths. Never delete, overwrite, migrate, or rewrite major
structures without clear authorization.

**9. Maintain Architectural Consistency** — Prefer consistency over novelty.
Follow existing architecture, patterns, naming, and state-management
conventions. Avoid introducing new frameworks without justification. No one-off
architectural exceptions.

**10. Keep Context Explicit** — Never rely on hidden assumptions. State
dependencies, constraints, and limitations. Record decisions and important
reasoning in the appropriate project file. Make implicit contracts explicit.

**11. Architecture Before Code** — Before implementation, identify affected
subsystems, data flow, interface contracts, persistence/UI/security/runtime
impact, and the testing strategy. Do not start coding until the architecture is
understood.

### Standards, state & portability

**12. Open Standards First** — Prefer open, portable, ecosystem-agnostic
standards: MCP, OpenAI-compatible APIs, A2A, AG-UI, A2UI, HTMX, WASM Component
Model, JSON Schema, OpenAPI, GraphQL (where appropriate), PostgreSQL-compatible
storage, IPFS-compatible distribution (where appropriate). Avoid vendor lock-in
unless explicitly required.

**13. No Hidden State** — Business state must live in explicit, inspectable
systems: databases, event streams, explicit stores, durable queues, documented
runtime state containers. State must not hide inside UI components, untracked
globals, implicit caches, framework magic, or agent-only memory without
persistence/auditability.

**14. Cross-Platform Parity** — Feature proposals must consider web, mobile,
desktop, local execution, cloud execution, and offline/degraded operation where
relevant. Do not trap the platform in a single runtime, framework, vendor, or
deployment model unnecessarily.

**35. Prefer Deterministic Systems** — Where possible prefer deterministic
behavior: IDs, allocation, ordering, retries, replay, and explicit conflict
resolution. Non-determinism must be intentional and documented.

**36. Local-First When Practical** — Prefer architectures that run locally and
sync outward: local execution/storage, offline-capable workflows, syncable
state, portable runtimes, edge-compatible agents. Cloud is allowed but avoid
unnecessary cloud dependence.

**37. Runtime Portability Matters** — Design for cloud, local, mobile, browser,
edge, WASM, and containerized execution. Avoid coupling business logic to a
runtime unless required.

### Architecture & layering

**15. Feature-Based Clean Architecture Required** — Organize codebases around
features/domains/bounded contexts rather than technical layers:

```
src/
├── features/
│   ├── customers/
│   │   ├── components/  hooks/  stores/  services/
│   │   ├── types/  schemas/  pages/  tests/
│   ├── orders/
│   └── billing/
├── shared/
├── core/
└── infrastructure/
```

Organize by business capability first. Avoid global component dumping grounds.
Keep feature logic in the owning feature. Shared code must be genuinely
reusable. Cross-feature dependencies must be explicit. Business logic belongs to
the feature domain, not the UI.

**16. Strict Layering Is Mandatory** — Enforce clear boundaries with one
direction of flow:

```
UI Components → Hooks/View Models → State Stores → Services/Repositories/APIs → External Systems
```

Reverse communication occurs only through state propagation and events.
*Allowed:* UI→Hook, Hook→Store, Store→Service, Service→API. *Not allowed:*
UI→API, UI→Service, UI→Database, Hook→API, Hook→Service, Component→Store-mutation
logic. (This generalizes the React layering section above.)

**17. UI Components Must Remain Pure** — Components only render, handle
interaction, layout, styling, and accessibility. They must not fetch data, call
APIs/services, perform business logic, manage persistence, or run workflow
logic. A component should be replaceable without affecting business behavior.

**18. Hooks/View Models Coordinate UI State** — Hooks connect UI to stores and
handle UI-state composition, UI-derived calculations, and presentation logic.
They must not call APIs/DBs directly, implement persistence, or contain domain
rules. (React Hooks, Flutter Controllers, Riverpod Notifiers, Vue Composables.)

**19. Stores Own Application State** — Stores are the single source of truth:
they call services, coordinate loading, manage optimistic updates, maintain
cache, and expose reactive state. Stores must not contain UI rendering logic.
(Zustand, Riverpod, Redux Toolkit, MobX, Signals.)

**20. Services Own External Communication** — Services handle API calls, DB
access, MCP/agent communication, external integrations, and file I/O. They must
be reusable, testable, and framework-independent where possible; they must not
render UI, manage component state, or contain presentation concerns.

**21. State Changes Must Be Reactive** — Propagate state changes through the
framework's native reactive mechanism (Zustand/Riverpod/Signals/Rx/Vue/Svelte).
Avoid manual refresh calls, hidden mutable state, direct component manipulation,
or imperative UI synchronization. The UI reacts automatically.

**24. Consistency Across Languages** — The architecture is identical regardless
of language; only the technology changes:

```
React:      Component → Hook        → Zustand Store → Service → API
Flutter:    Widget    → Controller  → Riverpod      → Service → API
Rust HTMX:  Template  → Handler     → Store/Domain  → Service → Repository
Vue:        Component → Composable  → Store         → Service → API
Svelte:     Component → View Model  → Store         → Service → API
```

**28. No Untouchable Framework Magic** — Do not introduce systems that force
case-by-case reasoning around hidden behavior: opaque caches, hidden global
state, framework-owned business logic, state trapped in component tiers, magic
side effects, uninspectable runtime behavior. Prefer predictable, explicit,
inspectable architecture.

**38. UI Is a Projection of State** — The UI must not be the source of truth. UI
renders state and submits intent; domain logic validates intent; durable systems
persist state; events describe changes. No business rules that exist only in
frontend components.

**39. Artifacts Must Be Structured** — Prometheus artifacts must be typed,
versioned, inspectable, portable, renderable across supported hosts, compatible
with agent workflows, and safe to persist and replay. Do not invent ad hoc
artifact formats when a formal schema exists.

### Dependencies & typing

**22. Dependency Versions Must Be Verified** — Before adding libraries,
frameworks, SDKs, runtimes, build tools, or infra: verify current compatible
versions against official docs/repos/compatibility matrices and existing
dependencies. Never assume versions or reuse stale training-era examples when
current info is available.

**23. Web Verification Before Dependency Introduction** — When internet access
is available, search for the latest stable version, known compatibility issues,
breaking changes, migration requirements, and security advisories. Priority:
official docs → official repo → release notes → vendor migration guides.
Community sources supplement, never replace, authoritative ones.

**27. No Silent Dependency Introduction** — Before adding a dependency, check
existing deps and prefer existing project tools. Explain why it is needed. Avoid
large deps for small tasks, architecture-conflicting deps, and lock-in.

**29. Strong Typing Required** — Use strong types wherever the language supports
them. No implicit/unnecessary `any`, no untyped business objects, no stringly
typed domain models when proper types are possible. Prefer generated types from
schemas. Keep API contracts typed and versioned.

### Process, safety & completion

**25. Human Override Always Exists** — Every automated decision must support
inspection, auditability, override, recovery, manual correction, and human
escalation. Agents may assist, recommend, automate, and execute, but humans must
remain able to inspect and override critical outcomes.

**26. Repo-Level Rules Override Base Rules Only When Explicit** — These are base
rules. Project-specific `CLAUDE.md`, `AGENTS.md`, README, architecture docs, or
task instructions may add stricter requirements, and may override these only
when explicit and non-contradictory with safety, correctness, and user intent.

**30. Tests Are Part of Completion** — Implementation is not complete until
verified. Where available, run unit/integration tests, type checks, linters, and
build checks; add tests for new behavior; update tests when behavior
intentionally changes. If tests cannot be run, state why.

**31. Prefer Small, Reviewable Changes** — Keep commits focused and diffs small.
Avoid broad rewrites and unrelated cleanup. Separate mechanical changes from
behavioral changes. Explain what changed and why.

**32. Preserve Existing Behavior** — Do not break existing behavior unless the
task requires it. First identify current vs desired behavior and compatibility
impact; update tests and docs; call out breaking changes clearly.

**33. Security Is Not Optional** — Always consider authentication,
authorization, input validation, output escaping, secrets handling, tenant
boundaries, data leakage, prompt injection, tool-execution boundaries, and
dependency risk. Never log secrets, tokens, credentials, private keys, or
sensitive user data.

**34. Agent Actions Must Be Auditable** — For agentic systems, preserve an audit
trail: user request, agent decision, tool calls, inputs, outputs, files changed,
external effects, errors, and human approvals where required. Agentic execution
without auditability is not acceptable.

**40. Stop When Done** — Do not keep expanding after the goal is satisfied. When
done: summarize what changed, summarize how it was verified, and list remaining
risks or follow-ups. Do not perform extra work unless asked.

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
