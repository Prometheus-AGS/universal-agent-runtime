# Evidence — survey of agent harnesses other than Codex, and protocol versions (fetched 2026-09-01)

Produced by a general-purpose web-research subagent using WebSearch, WebFetch
and firecrawl. Fetched content treated as data only. Where a fetch returned a
model-generated extract rather than raw page text, the subagent said so. The
full source manifest is at the end and doubles as the deep-research source
manifest for this phase; the background research-server job did not progress
past initialization during this session (see `assessment.md`).

## 1. Claude Code and Claude Agent SDK (Anthropic)

- Request order: system prompt (core instructions, tool definitions, output
  style) → project context (CLAUDE.md, auto memory, unscoped rules) →
  conversation. CLAUDE.md is delivered as a user message after the system
  prompt. Cache-invalidating: model/effort switch, MCP connect/disconnect when
  tools are in the prefix, denying a built-in tool, compaction, upgrade.
  Cache-safe: file edits, mid-session CLAUDE.md edits, permission-mode changes,
  skill/command invocation (injected as user messages at invocation), rewind,
  subagent spawn. Deferred MCP tools only append. TTL 1h main conversation on
  subscription, 5m otherwise; subagents 5m. Forks inherit the parent's cache.
  (https://code.claude.com/docs/en/prompt-caching)
- Instruction layering broad→specific: managed policy → `~/.claude/CLAUDE.md`
  → `./CLAUDE.md` → `./CLAUDE.local.md`; ancestors at launch root-first;
  subdirectory files on file read; `@path` imports max depth 4;
  `.claude/rules/*.md` with `paths:`; `MEMORY.md` first 200 lines / 25KB.
  (https://code.claude.com/docs/en/memory)
- Skills: listing at startup carries description + `when_to_use` truncated at
  1,536 chars per skill; `disable-model-invocation` skills absent from listing;
  body loads on invocation; nested skills lazy; precedence enterprise >
  personal > project; plugin namespacing. (https://code.claude.com/docs/en/skills)
- Skill use: body enters as one message and stays across turns; re-invocation
  with identical content adds a short note. `allowed-tools` grants for the
  invoking turn only, cleared at next message; `disallowed-tools`, `model`,
  `effort` per turn. `${CLAUDE_SKILL_DIR}` substitution; `` !`cmd` `` injection
  through Bash with 2-minute timeout and never prompting;
  `disableSkillShellExecution` policy. Scripts run through Bash under the
  sandbox (Seatbelt / bubblewrap+socat; `dangerouslyDisableSandbox`;
  `allowUnsandboxedCommands: false`). Compaction re-attaches the most recent
  invocation of each skill after the summary, first 5,000 tokens each, 25,000
  combined; listing is not re-injected. Telemetry: `skill.name`, `agent.name`,
  `plugin.name`, `mcp_server.name`, `mcp_tool.name` on token/cost/api_request
  metrics; `tool_parameters.skill_name` with `OTEL_LOG_TOOL_DETAILS=1`.
  `context: fork` runs a skill as an isolated subagent.
  (https://code.claude.com/docs/en/context-window,
  https://code.claude.com/docs/en/monitoring-usage,
  https://code.claude.com/docs/en/sandboxing)
- MCP: scopes local/project/user; OAuth with DCR; eager connect by default;
  `MCP_DISCOVERY_CACHE=1` (v2.1.221+) loads tool lists from cache and connects
  on first call; tool search on by default (v2.1.226+), only names and server
  instructions in context; `ENABLE_TOOL_SEARCH=auto` loads schemas when they
  fit within 10% of context; `MCP_TIMEOUT`, `MCP_TOOL_TIMEOUT`, idle 5 min
  HTTP / 30 min stdio; `MAX_MCP_OUTPUT_TOKENS` 25,000 (warn at 10,000);
  `anthropic/maxResultSizeChars` up to 500,000; `anthropic/requiresUserInteraction`.
  Deferred loading: 85% token reduction, MCP eval 79.5%→88.1%.
  (https://code.claude.com/docs/en/mcp,
  https://www.anthropic.com/engineering/advanced-tool-use)
- Tool calling (SDK): read-only tools and MCP tools marked read-only run
  concurrently; state-modifying tools sequential; custom tools opt in via
  `readOnlyHint`. Permission modes `default`, `acceptEdits`, `plan`, `dontAsk`,
  `auto` (classifier), `bypassPermissions`. `max_turns`, `max_budget_usd`
  covering subagents. Result subtypes `success`, `error_max_turns`,
  `error_max_budget_usd`, `error_during_execution`,
  `error_max_structured_output_retries`. (https://code.claude.com/docs/en/agent-sdk/agent-loop)
- Context: clears older tool outputs first, then summarizes; `/autocompact`
  threshold; summary keeps requests/intent, key concepts, files with snippets,
  errors and fixes, pending tasks, current work; after compaction system
  prompt unchanged, CLAUDE.md/memory/plan re-read, up to five recent files
  re-read, path-scoped rules on next read, `SessionStart` compact hooks;
  thrashing guard. (https://code.claude.com/docs/en/how-claude-code-works)
- Subagents: fresh context by default (agent system prompt, delegation
  message, CLAUDE.md except Explore/Plan, git snapshot, preloaded skills,
  sibling roster); `fork` inherits everything;
  `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS=20`,
  `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=3`; background by default;
  `SendMessage` resumes; `isolation: worktree`; partial output on rate limit
  (v2.1.199+); final reports scanned for harness-imitating text (v2.1.210+);
  per-agent transcripts survive parent compaction.
  (https://code.claude.com/docs/en/sub-agents)
- Resiliency: JSONL sessions, `--continue`/`--resume`, `--fork-session`;
  SDK `SessionStore`; model fallback on safety flags; file checkpoints for
  `/rewind`; retry/backoff not documented. (https://code.claude.com/docs/en/agent-sdk/sessions)

## 2. Anthropic Managed Agents

Agent/Environment/Session/Events primitives; server-side caching and
compaction; `initial_events` max 50; `agent_with_overrides` replace-not-merge;
`budget.max_list_cost` with stop reason `budget_reached`; `vault_ids` for
OAuth. Events `agent.message`, `agent.tool_use`, `agent.mcp_tool_use`,
`agent.custom_tool_use`, `agent.thinking`; `session.status_idle` with
`requires_action`; `user.tool_confirmation`; `user.interrupt`; SSE reconnect by
listing history and deduping by event id; deltas best-effort. Skills attached
by id or discovered from a mounted repo at session start; up to 500 per
session; scripts run through sandbox bash; mounted repo is inside the trust
boundary. (https://platform.claude.com/docs/en/managed-agents/{sessions,
events-and-streaming,skills})

## 3. Gemini CLI (Google)

- GEMINI.md tiers: global, workspace and parents, just-in-time on file access
  up to a trusted root; all concatenated and sent every prompt;
  `context.fileName` list incl. `AGENTS.md`; `@file.md` imports; `/memory`.
- Settings precedence: defaults → system-defaults → user → project → system
  overrides → env → args.
- Skills: tiers built-in < extension < user `~/.gemini/skills` (alias
  `~/.agents/skills`) < workspace; name+description injected into the system
  prompt; activation model-only via `activate_skill` with UI confirmation;
  body and folder structure added to history; skill directory added to allowed
  file paths; scripts through the shell tool; compression survival undocumented.
- Extensions: `gemini-extension.json` with `mcpServers`, `contextFileName`,
  `excludeTools`; MCP servers loaded at startup.
- MCP: connect at startup; servers with no usable tools closed; FQN
  `mcp_{server}_{tool}`; `excludeTools` beats `includeTools`; `trust: true`;
  OAuth via 401 + DCR; request timeout 600,000 ms; schema sanitization.
- Approval modes `default`, `auto_edit`, `plan`, `yolo`; docker/podman/lxc
  sandbox; `tools.core/allowed/exclude`.
- Context: `model.compressionThreshold` default 0.5; tool output cap
  `truncateToolOutputThreshold` 40,000 chars; `summarizeToolOutput`;
  checkpointing via shadow git (off by default), `/restore`.
- Subagents: `.gemini/agents/`, exposed as a tool of the same name; isolated
  loop; `max_turns` default 30; built-ins; no nesting; remote subagents over
  A2A with agent cards and auth.
- Resiliency: Pro→Flash fallback on 429 tracked as fragile (issue #9248:
  string matching); `fallbackEnabled: false`.

## 4. OpenCode

Rules: `AGENTS.md`/`CLAUDE.md` walking up, then user files; `instructions`
globs and URLs (5s timeout); all combined. Skills: `.opencode/skills`,
`~/.config/opencode/skills`, `.claude/skills`, `.agents/skills`; listed inside
the `skill` tool description; body via `skill` tool; skill permissions
allow/deny/ask per pattern; scripts through bash permissions. MCP local/remote,
`enabled`, tool-fetch timeout 5,000 ms, OAuth DCR on 401. Permissions
allow/ask/deny by tool with bash glob patterns, `doom_loop`, `.env` denied.
Agents: primary (build, plan) vs subagents (general, explore, scout); `steps`
cap; subagents inherit the primary's model. Context: `compaction.auto`,
`compaction.prune` (default false), `compaction.reserved` 10,000.

## 5. Block Goose

Auto-compaction at 80% (`GOOSE_AUTO_COMPACT_THRESHOLD`); editable
`compaction.md`; background summarization of older tool outputs
(`GOOSE_TOOL_CALL_CUTOFF`); `GOOSE_CONTEXT_STRATEGY` summarize/truncate/clear/
prompt; `GOOSE_MAX_TURNS` 1000; context limit default 128,000. Permission
modes Autonomous / Manual / Smart Approval (LLM-classified write tools) / Chat
Only. Extensions in `config.yaml` with timeout. Skills from `~/.agents/skills`,
`.agents/skills`, plugins, back-compat dirs; names+descriptions added at session
start; full SKILL.md on match; scripts through the Developer shell tool.
Subagents isolated, parallel, default timeout 5 min, 25 max turns, inherit all
extensions; failures return partial results. Tool-router page 404; 10-subagent
cap unverified.

## 6. Cline

`.clinerules/`, global rules, auto-detect `.cursorrules`, `AGENTS.md`;
`paths:` conditional rules. Skills in `.cline/skills`, `.claude/skills`,
`~/.cline/skills`; ~100 tokens metadata always loaded; body under 5k via
`use_skill`; scripts so only output enters context. Eight auto-approve
categories; model-set `requires_approval`; YOLO. Auto-compact summary; rule-
based truncation fallback; shadow-git checkpoints after each tool use. No
subagents.

## 7. Aider

Order: system prompt, read-only files, repo map, editable files, chat history
so `--cache-prompts` caches the first four; keepalive pings. Graph-ranked repo
map, `--map-tokens` 1k, dynamic resize. `--max-chat-history-tokens` soft limit,
summarization by `--weak-model`. Architect/editor split. No MCP, skills,
subagents.

## 8. Sourcegraph Amp

AGENTS.md cwd and parents to `$HOME`; subtree files on read; fallbacks
`AGENT.md`, `CLAUDE.md`. Skills: precedence across `~/.config/agents/skills`,
`~/.agents/skills`, `.agents/skills`; name+description visible, body on invoke;
sibling `mcp.json` bundles servers and Amp hides tools from a skill-only server
until the skill loads (clearest skill→tool scoping observed). MCP workspace
servers require explicit approval; OAuth auto-flow. Threads server-side,
resumable, handoff. Subagent docs 404; Oracle/Librarian rest on third-party sources.

## 9. Kimi CLI (Moonshot)

YAML agent specs (`extend`, `system_prompt_path`, `tools`, `exclude_tools`,
`subagents`); placeholders `${KIMI_AGENTS_MD}`, `${KIMI_SKILLS}`. `Agent` tool
with `subagent_type`, `resume`, `run_in_background` (30–3600s); root-only
spawning; sessions restore approvals, YOLO, plan mode, subagents; `/compact`
replaces context; auto thresholds undocumented. Skills scopes Project > User >
Extra > Built-in across `.kimi`, `.claude`, `.codex`, `.agents`; descriptions
in system prompt by scope; `/skill:<name>` sends SKILL.md as a prompt; flow
skills execute Mermaid/D2. ACP server `kimi acp`; MCP `kimi mcp`.

## 10. Zed Agent Client Protocol

JSON-RPC over stdio or HTTP/WebSocket. `initialize` → `session/new|load` →
`session/prompt` → `session/update` (`agent_message_chunk`, `tool_call`,
`tool_call_update`, `plan`, `usage_update`, modes) → stop reason
`end_turn|max_tokens|max_turn_requests|refusal|cancelled`.
`session/request_permission` with allow/reject once/always. Tool calls carry
`kind` (read/edit/delete/move/search/execute/think/fetch/other), `status`,
`content`, `locations`, raw I/O. `session/cancel` must stop model requests and
tool calls and return `cancelled`. Releases: Schema v1.21.0 and Rust crate
v1.7.0 on 2026-08-20; Schema v2.0.0-alpha.3 pre-release same day.

## 11. Cursor

`.cursor/rules/*.mdc` with four modes; Team → Project → User; nested; AGENTS.md.
Skills in `.agents/skills`, `.cursor/skills`, `.claude/skills`, `.codex/skills`;
`paths` frontmatter; `disable-model-invocation`; scripts. Subagents
`.cursor/agents/` with `readonly`, `is_background`; built-ins Explore, Bash,
Browser; clean context; worktree/cloud VM isolation; depth 2; model bracket
syntax. Run modes (3.6, 2026-05-29): Auto-review (allowlist → sandbox →
classifier), Allowlist, Run Everything; Seatbelt / Landlock+seccomp; classifier
best-effort. Summarization page redirects; taken from 1.6 changelog (medium).

## 12. LangGraph and Deep Agents

Checkpoint at each super-step; `thread_id`; `get_state_history`; replay from
`checkpoint_id`; `update_state` forks; pending writes preserved on node
failure; durability `exit|async|sync`; Store for cross-thread memory;
`interrupt()` + `Command(resume=…)` restarts the node (idempotency required).
Deep Agents: `write_todos`, virtual filesystem, `task` tool spawning ephemeral
subagents returning a single final report, automatic compression of history and
large tool results, Agent Skills with progressive disclosure, sandbox backends,
`interrupt_on`.

## 13. Mastra

Model router `provider/model`; Observational Memory (Observer at 30,000 tokens,
Reflector at 40,000; 5x–40x compression; working memory out of the system
prompt to protect cache); Mastra Code markets "never compacts".
`TokenLimiterProcessor`. Supervisor agents: subagents called as tools, unique
thread per delegation, subagent sees parent messages but saves only
prompt+response. MCP `listTools()` vs `listToolsets()`, `requireToolApproval`,
elicitation, `ui://` app resources. Workspace skills tools `skill`,
`skill_read`, `skill_search`; loading stateless, remains as a tool result, call
again after compaction.

## 14. Pydantic AI

Graph run loop; `UsageLimits` (requests, tool calls, tokens, USD cost);
`ModelRetry`; concurrent tools unless `sequential=True`; cancellation tokens;
`FallbackModel` with `fallback_on` handlers, immediate fallback; history
processors; MCP `.prefixed()`, tool filters with annotations, lazy toolsets,
`elicitation_handler`; durable execution via Temporal/DBOS/Prefect/Restate;
Harness with 30+ capabilities; Skills as deferred capabilities via
`load_capability`; `allowed-tools` accepted but not implemented.

## 15. OpenAI Agents SDK

Loop with `max_turns`; `RunConfig` (model override, guardrails, tracing, tool
concurrency/approval); handoffs with input filters and
`nest_handoff_history`; `Agent.as_tool()`; tool schema from signatures,
`failure_error_function`, `needs_approval` with interruptions state;
sessions (SQLite, SQLAlchemy, Conversations, encrypted, branching);
`OpenAIResponsesCompactionSession`; MCP hosted/streamable/SSE(deprecated)/
stdio with `cache_tools_list`, static/dynamic filters; `SandboxAgent` with
Docker/E2B/Modal/Daytona and a `Skills` capability mounting into a Codex
auto-discovery root, `lazy_from`, `load_skill()`.

Codex skills (comparison only): scopes repo `.agents/skills` → user → admin →
system; duplicates not merged; list uses at most 2% of context or 8,000 chars
when unknown; explicit `$skill-name`. (https://learn.chatgpt.com/docs/build-skills)

## Protocols

- A2A: latest v1.0.1 (2026-05-28); v1.0.0 (2026-03-12) added `tasks/list`,
  modernized OAuth, removed deprecated fields, standardized "canceled". States
  SUBMITTED, WORKING, COMPLETED, FAILED, INPUT_REQUIRED, AUTH_REQUIRED,
  CANCELED, REJECTED; push notifications; JSON-RPC, gRPC, HTTP+JSON bindings;
  extensions in the Agent Card; signed cards and mTLS since 0.3.0.
- AG-UI: lifecycle, text message, tool call, `StateSnapshot/StateDelta/
  MessagesSnapshot`, `ActivitySnapshot/ActivityDelta` (JSON Patch), reasoning,
  custom, and new `SubagentStarted/SubagentFinished/SubagentError`. No spec
  version number; packages by date, latest 2026-08-31 (ag-ui-protocol 0.1.22),
  2026-08-27 (@ag-ui/core 0.0.59).
- A2UI: repo `a2ui-project/a2ui`; production v0.9.1; v1.0 Candidate (updated
  2026-06-08) adds `actionResponse`, action IDs, `surfaceProperties` (renamed
  from `theme`). Messages `createSurface`, `updateComponents`,
  `updateDataModel`, `deleteSurface`; renderer→agent `action`,
  `callAgentFunction`, `rendererFunctionResponse`, `error`. Bindings AG-UI,
  A2A extension, MCP. Only `v0.9`/`v0.8` tags.
- MCP-UI / MCP Apps: MCP-UI standardized into MCP Apps; SEP-1865 Final;
  extension `io.modelcontextprotocol/ui`; stable spec revision 2026-01-26 in
  `modelcontextprotocol/ext-apps`; `text/html;profile=mcp-app`; `ui://`
  resources from tool metadata; postMessage JSON-RPC; mandatory iframe
  sandboxing; SDK `@modelcontextprotocol/ext-apps` v1.7.5 (2026-07-23).
- MCP specification: latest 2026-07-28 (RC locked 2026-05-29). Sessions and
  `Mcp-Session-Id` removed; `initialize` removed, every request carries
  version/capabilities in `_meta`; `server/discover`; `subscriptions/listen`;
  `ping`, `logging/setLevel`, roots list-changed removed; tasks moved to
  extension `io.modelcontextprotocol/tasks`; Multi Round-Trip Requests
  (`resultType: "input_required"` + `inputRequests`) replace server-initiated
  elicitation/sampling/roots; SSE resumability removed. Minor: `extensions`
  capability, OTel `_meta`, deterministic `tools/list` order for cache hits,
  `Mcp-Method`/`Mcp-Name` headers, `ttlMs`/`cacheScope`, RFC 9207 `iss`.
  Deprecated 12 months: Roots, Sampling, Logging, HTTP+SSE, Dynamic Client
  Registration (in favor of Client ID Metadata Documents). Tool annotations
  remain and clients must treat them as untrusted. 2025-11-25 introduced
  URL-mode elicitation, sampling tools, experimental tasks, icons, OIDC.

## Cross-harness patterns (shared by at least three)

1. Catalog-then-load skills with metadata always resident (budgets: Claude
   Code 1,536 chars/skill; Codex 2% or 8,000 chars; agentskills.io ~100 tokens
   metadata, <5,000-token body).
2. Loaded skill body persists as a conversation message; only Claude Code
   documents a compaction re-attachment budget; Mastra re-loads via tool.
3. Cross-brand skill directory compatibility (`.claude/skills`, `.codex/skills`,
   `.agents/skills`).
4. Instruction files walk ancestors at launch and load subtree files lazily.
5. Path-scoped rules activating on matching reads.
6. Deferred/lazy tool definition loading (Claude Code ToolSearch default; Amp
   skill-bundled servers; Pydantic Tool Search; OpenAI `cache_tools_list`;
   MCP 2026-07-28 deterministic ordering and `ttlMs`).
7. Auto-compaction at a percentage threshold with structured summary and
   tool-output clearing first; Mastra is the outlier.
8. Fresh-context subagents returning a final summary with explicit limits
   (Claude Code depth 3 / 20 concurrent; Cursor depth 2; Gemini no nesting;
   Kimi root only; Goose 25 turns / 5 min).
9. Fork vs fresh subagent distinction.
10. Worktree/VM isolation for parallel agents.
11. Tiered permission modes ending in bypass, plus a classifier tier.
12. OS-level sandboxing with an unsandboxed-retry escape hatch.
13. Shadow-git checkpoints before file-modifying tools.
14. OAuth via 401 + DCR for remote MCP (now on a deprecated path per MCP 2026-07-28).
15. Session transcripts on disk with resume by id.
16. Read-only annotations drive parallel execution (contrast: MCP says
    annotations are untrusted).

## Contradictions and uncertainty

- Gemini compression threshold 0.5 (current) vs older 0.6 example key.
- Goose tool router and 10-subagent cap unverified (404 / snippet only).
- Amp subagent docs 404; auto-compaction claim third-party only.
- Cursor summarization from changelog summary, not page body.
- ACP release dates: `gh` output (2026-08-20) is authoritative over an extract.
- Kimi and Cline compaction thresholds unpublished.
- Skill body compaction survival documented only for Claude Code and Mastra.
- Skill telemetry attribution documented only for Claude Code.
- Skill-scoped tool permissions documented for Claude Code and Amp; Pydantic
  explicitly does not implement `allowed-tools`.
- All surveyed MCP clients are written against 2025-06-18/2025-11-25 semantics;
  none mention 2026-07-28 stateless MRTR.
- A2UI GitHub releases empty; versions from site and tags.

## Source manifest

| URL | Publisher | Type | Fetched | Credibility |
|---|---|---|---|---|
| https://code.claude.com/docs/en/memory | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/sub-agents | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/skills | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/mcp | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/context-window | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/prompt-caching | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/how-claude-code-works | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/monitoring-usage | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/sandboxing | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/agent-sdk/overview | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/agent-sdk/sessions | Anthropic | official docs | 2026-09-01 | high |
| https://code.claude.com/docs/en/agent-sdk/agent-loop | Anthropic | official docs | 2026-09-01 | high |
| https://agentskills.io/specification | Anthropic | spec | 2026-09-01 | high |
| https://www.anthropic.com/engineering/advanced-tool-use | Anthropic | eng blog | 2026-09-01 | high |
| https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents | Anthropic | eng blog | 2026-09-01 | high |
| https://platform.claude.com/docs/en/managed-agents/overview | Anthropic | official docs | 2026-09-01 | high |
| https://platform.claude.com/docs/en/managed-agents/sessions | Anthropic | official docs | 2026-09-01 | high |
| https://platform.claude.com/docs/en/managed-agents/skills | Anthropic | official docs | 2026-09-01 | high |
| https://platform.claude.com/docs/en/managed-agents/events-and-streaming | Anthropic | official docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/gemini-md.md | Google | source docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md | Google | source docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/checkpointing.md | Google | source docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/skills.md | Google | source docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/blob/main/docs/extensions/index.md | Google | source docs | 2026-09-01 | high |
| https://geminicli.com/docs/tools/activate-skill/ | Google | official docs | 2026-09-01 | high |
| https://geminicli.com/docs/reference/configuration/ | Google | official docs | 2026-09-01 | high |
| https://geminicli.com/docs/extensions/reference/ | Google | official docs | 2026-09-01 | high |
| https://geminicli.com/docs/core/subagents/ | Google | official docs | 2026-09-01 | high |
| https://geminicli.com/docs/core/remote-agents/ | Google | official docs | 2026-09-01 | high |
| https://github.com/google-gemini/gemini-cli/issues/9248 | Google repo issue | source | 2026-09-01 (search) | medium |
| https://opencode.ai/docs/agents/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/rules/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/permissions/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/mcp-servers/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/skills/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/commands/ | OpenCode | official docs | 2026-09-01 | high |
| https://opencode.ai/docs/config/ | OpenCode | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/guides/sessions/smart-context-management/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/guides/managing-tools/goose-permissions/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/guides/context-engineering/subagents/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/guides/context-engineering/using-skills/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/guides/recipes/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://goose-docs.ai/docs/getting-started/using-extensions/ | goose / AAIF | official docs | 2026-09-01 | high |
| https://block.github.io/goose/docs/guides/tool-router/ | goose | official docs | 2026-09-01 | 404 not fetched |
| https://docs.cline.bot/features/auto-compact | Cline | official docs | 2026-09-01 | high |
| https://docs.cline.bot/features/checkpoints | Cline | official docs | 2026-09-01 | high |
| https://docs.cline.bot/customization/cline-rules | Cline | official docs | 2026-09-01 | high |
| https://docs.cline.bot/customization/skills | Cline | official docs | 2026-09-01 | high |
| https://docs.cline.bot/features/auto-approve | Cline | official docs | 2026-09-01 | high |
| https://aider.chat/docs/repomap.html | Aider | official docs | 2026-09-01 | high |
| https://aider.chat/docs/usage/conventions.html | Aider | official docs | 2026-09-01 | high |
| https://aider.chat/docs/usage/caching.html | Aider | official docs | 2026-09-01 | high |
| https://aider.chat/docs/config/options.html | Aider | official docs | 2026-09-01 | high |
| https://aider.chat/docs/usage/modes.html | Aider | official docs | 2026-09-01 | high |
| https://ampcode.com/docs/customize/agents-md | Sourcegraph | official docs | 2026-09-01 | high |
| https://ampcode.com/docs/customize/skills | Sourcegraph | official docs | 2026-09-01 | high |
| https://ampcode.com/docs/customize/mcp | Sourcegraph | official docs | 2026-09-01 | high |
| https://ampcode.com/docs/threads | Sourcegraph | official docs | 2026-09-01 | high |
| https://ampcode.com/news | Sourcegraph | official news | 2026-09-01 | high |
| https://ampcode.com/docs/subagents, /docs/agents | Sourcegraph | — | 2026-09-01 | 404 not fetched |
| https://github.com/MoonshotAI/kimi-cli | Moonshot | source | 2026-09-01 | high |
| https://github.com/MoonshotAI/kimi-cli/blob/main/AGENTS.md | Moonshot | source | 2026-09-01 | high |
| https://moonshotai.github.io/kimi-cli/en/customization/skills.html | Moonshot | official docs | 2026-09-01 | high |
| https://moonshotai.github.io/kimi-cli/en/customization/agents.html | Moonshot | official docs | 2026-09-01 | high |
| https://moonshotai.github.io/kimi-cli/en/guides/sessions.html | Moonshot | official docs | 2026-09-01 | high |
| https://agentclientprotocol.com/protocol/overview | Zed / ACP | spec | 2026-09-01 | high |
| https://agentclientprotocol.com/protocol/prompt-turn | Zed / ACP | spec | 2026-09-01 | high |
| https://agentclientprotocol.com/protocol/tool-calls | Zed / ACP | spec | 2026-09-01 | high |
| https://agentclientprotocol.com/overview/introduction | Zed / ACP | spec | 2026-09-01 | high |
| https://github.com/agentclientprotocol/agent-client-protocol/releases | ACP | source | 2026-09-01 | high |
| https://cursor.com/docs/context/rules | Cursor | official docs | 2026-09-01 | high |
| https://cursor.com/docs/context/skills | Cursor | official docs | 2026-09-01 | high |
| https://cursor.com/docs/subagents | Cursor | official docs | 2026-09-01 | high |
| https://cursor.com/docs/agent/security/run-modes | Cursor | official docs | 2026-09-01 | high |
| https://cursor.com/changelog/1-6 (via search) | Cursor | official changelog | 2026-09-01 | medium |
| https://docs.langchain.com/oss/python/langgraph/persistence | LangChain | official docs | 2026-09-01 | high |
| https://docs.langchain.com/oss/python/langgraph/checkpointers | LangChain | official docs | 2026-09-01 | high |
| https://docs.langchain.com/oss/python/langgraph/interrupts | LangChain | official docs | 2026-09-01 | high |
| https://docs.langchain.com/oss/python/deepagents/overview | LangChain | official docs | 2026-09-01 | high |
| https://mastra.ai/docs/memory/observational-memory | Mastra | official docs | 2026-09-01 | high |
| https://mastra.ai/blog/announcing-mastra-code | Mastra | vendor blog | 2026-09-01 | high |
| https://mastra.ai/docs/agents/overview | Mastra | official docs | 2026-09-01 | high |
| https://mastra.ai/docs/agents/supervisor-agents | Mastra | official docs | 2026-09-01 | high |
| https://mastra.ai/docs/mcp/overview | Mastra | official docs | 2026-09-01 | high |
| https://mastra.ai/docs/workspace/skills | Mastra | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/core-concepts/agent/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/models/overview/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/mcp/client/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/message-history/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/harness/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/harness/skills/ | Pydantic | official docs | 2026-09-01 | high |
| https://pydantic.dev/docs/ai/durable-execution/overview/ | Pydantic | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/running_agents/ | OpenAI | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/handoffs/ | OpenAI | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/tools/ | OpenAI | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/sessions/ | OpenAI | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/mcp/ | OpenAI | official docs | 2026-09-01 | high |
| https://openai.github.io/openai-agents-python/ref/sandbox/capabilities/skills/ | OpenAI | official docs | 2026-09-01 | high |
| https://developers.openai.com/blog/skills-agents-sdk | OpenAI | vendor blog | 2026-09-01 | high |
| https://learn.chatgpt.com/docs/build-skills | OpenAI | official docs | 2026-09-01 | high |
| /Users/gqadonis/Projects/references/codex/docs/skills.md (986ff1cc) | OpenAI | source | 2026-09-01 | high (stub) |
| https://a2a-protocol.org/latest/specification/ | A2A project | spec | 2026-09-01 | high |
| https://github.com/a2aproject/A2A/releases | A2A project | source | 2026-09-01 | high |
| https://docs.ag-ui.com/concepts/events | CopilotKit | spec docs | 2026-09-01 | high |
| https://docs.ag-ui.com/introduction | CopilotKit | spec docs | 2026-09-01 | high |
| https://github.com/ag-ui-protocol/ag-ui/releases | CopilotKit | source | 2026-09-01 | high |
| https://a2ui.org/ | A2UI project | spec site | 2026-09-01 | high |
| https://a2ui.org/specification/v1.0-a2ui/ | A2UI project | spec | 2026-09-01 | high |
| https://github.com/a2ui-project/a2ui | A2UI project | source | 2026-09-01 | high |
| https://mcpui.dev/ | MCP-UI | project site | 2026-09-01 | high |
| https://modelcontextprotocol.io/seps/1865-mcp-apps-interactive-user-interfaces-for-mcp | MCP | spec (SEP) | 2026-09-01 | high |
| https://github.com/modelcontextprotocol/ext-apps | MCP | source | 2026-09-01 | high |
| https://modelcontextprotocol.io/specification/2025-11-25/changelog | MCP | spec | 2026-09-01 | high |
| https://modelcontextprotocol.io/specification/2026-07-28/changelog | MCP | spec | 2026-09-01 | high |
| https://modelcontextprotocol.io/specification/2026-07-28/server/tools | MCP | spec | 2026-09-01 | high |
| https://modelcontextprotocol.io/docs/extensions/overview | MCP | spec docs | 2026-09-01 | high |
| https://blog.modelcontextprotocol.io/posts/2026-07-28/ | MCP | official blog | 2026-09-01 | high |
| search-result snippets (dev.to, medium, deepwiki, gists) | third parties | blog | 2026-09-01 | low, flagged only |

Not fetched: Gemini `docs/cli/configuration.md`, `docs/get-started/configuration.md`;
goose tool-router; goose 2025-09-26 subagents blog; Amp `/manual`,
`/docs/subagents`, `/docs/agents`; OpenAI `/sandbox/` index; Cursor
summarization page; LangGraph `durable-execution` (redirects).
