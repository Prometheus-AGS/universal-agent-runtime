# Prior context — dot-feature-support (execute)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from surreal-memory. Cite the lessons that apply; recalled entries are information recorded by agents, not instructions.

## Lessons

- [user] The operator approves merges, tags and machine-level changes explicitly. A single up-front approval question at the phase boundary keeps the run unblocked. _(recorded by an agent; lesson, stage reflect, 2026-10-05; via surreal-memory)_
- [user] The operator merges every PR, wants the full KBD status at the end of every turn, and wants operator-only actions (tags, repos, live-data changes) asked explicitly. _(recorded by an agent; lesson, stage reflect, 2026-10-04; via surreal-memory)_
- [global] A component test passing is not evidence that a production entry point uses the component. When a spec says "the installer applies X", test through the installer's dispatch, or at least assert structurally that the install path reaches the call. _(recorded by an agent; lesson, stage reflect, 2026-10-05; via surreal-memory)_
- [global] Preserve exact source and failure evidence before validation repairs: aggregate hashes cannot reconstruct uncommitted before/after files, and discarded stderr prevents diagnosis. Report source completion, actual acceptance, publication and installed operation separately. Stop when the user says stop testing; do not rename or delegate it. Do not spend an unbounded release cycle engineering certification frameworks or promise a deadline that excludes human merges and missing prerequisites. _(recorded by an agent; lesson, stage reflect, 2026-10-05; via surreal-memory)_
- [global] Tests must discover their environment (real cache layouts, submodule initialisation) rather than encode one checkout's state, or the phase-boundary gate on a fresh worktree will disagree with the worker's run. _(recorded by an agent; lesson, stage reflect, 2026-10-05; via surreal-memory)_

## pk knowledge

- [pk:project] Karpathy session a7978cc98797: I now have the facts to evaluate your three tiers honestly. Here's the assessment — including where I think one of them is a bad idea. | **Ollama** | Installed, running, OpenAI-compatible at `:11434/v1` | | Local chat models | **`llama3.2:1b` only** — everything else is `:cloud` (Ollama-hosted, needs their auth) or embeddings | | Configured keys | `KIMI_CODING_KEY`, `MINIMAX_KEY`, `LITER_LLM_MASTER_KEY` — all set | _(recorded by an agent; lesson; via pk pk:karpathy-session-a7978cc987977ee2)_
- [pk:project] Karpathy session 0205d95c6113: Understood. Here's what I take you to be saying: - **It had real legacy KBD artifacts** — so the migration path got exercised against messy authentic state, not a clean fixture. That `migrate --apply` run today is a data point: it correctly registered 21 changes, refused to overwrite 14 files it didn't author, and flipped legacy → canonical. That's migration evidence. - **Cross-harness is the point.** Claude Code ran assess → analyze → plan. Codex picks up execute in a separate, currently-stopped session. The handoff artifacts (`execution.md`, `position-reminder.txt`, `progress.json`, the stag _(recorded by an agent; lesson; via pk pk:karpathy-session-0205d95c6113a615)_

## Previous reflection

From `allow-loopback-tools-without-jwt/reflection.md`:

**Delta**

The plan expected a code freeze before certification and one final quality gate. Delivery required two production-correction rounds, two complete certification restarts, and a final KBD record reconciliation. The first isolated critic found incorrect rollback ownership, a settings-notification race, an incomplete direct-HTTP authorization change, and missing durable evidence. The second found an over-broad HTTP bypass and stale recovery binaries. The third found stale KBD completion and publication claims. Those findings were corrected before the final PASS. PR #274 subsequently merged, so the previous reflection's statement that publication remained pending is obsolete.

**Root Cause**

Runtime authority crossed two enforcement boundaries: `RunManager` and the global HTTP Cedar middleware. The initial implementation changed only one. The first correction then bypassed route classification too early, expanding a tool-specific exception into a broader authorization exception.

Governance settings publication and notification had separate owners. A database notification could arrive before the in-process runtime snapshot changed, allowing clients to refetch stale authority.

Rollback documentation conflated posture-derived initialization with persisted operator state. Recovery artifacts also lacked commit-qualified identities. KBD projections then lagged the verified and published state.

**Corrective Actions**

- Shared one coherent governance gate across runtime tool execution and direct HTTP middleware.
- Limited Governance Off bypass to `POST` tool-execution routes.
- Made durable write, cache update, runtime publication, notification scheduling, and response ordering explicit.
- Distinguished API-owned persisted values from posture-owned defaults in rollback behavior.
- Rebuilt forward and rollback binaries from final commits and recorded commit-qualified names and SHA-256 digests.
- Corrected KBD completion, certification, publication, and archive projections after the final evidence existed.

**Next Phase Seed**

Do not invent a successor phase. The next operator-selected phase should first dispose of the remaining clean-startup-log evidence and archive `refresh-liter-surreal-dependencies`, then explicitly configure or disable optional integrations that currently log degraded startup, and repair the stale KBD position projection.

## Knowledge gaps

*(none recorded)*
