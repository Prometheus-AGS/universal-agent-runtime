# UAR operator shell — research findings and design brief

Date: 2026-10-05. Scope: the built-in React 19 web app in `universal-agent-runtime/frontend`,
redesigned to match the shell conventions of ChatGPT / Codex desktop, Claude Desktop / Claude Code,
Kimi desktop, and DeepSeek Harness, with liter-llm provider and model configuration as the
primary pain point. Three research tracks fed this: a web survey of the tools above plus the
open-source multi-provider apps (Cherry Studio, Open WebUI, Jan, Cursor, LibreChat, Continue,
Msty, LM Studio); a code audit of the current UAR frontend; and a read of UAR's liter-llm domain
(`src/llm/`, `src/uar/api/providers.rs`, `catalog/provider_catalog.json`).

## 1. What the current UAR frontend does today

- **Two stacked left columns.** A 240px nav rail (`app/shell/nav-rail.tsx`) sits beside a second
  240px thread list inside the chat route (`components/layout/left-sidebar.tsx`). Users see 480px of
  chrome before content. No other tool in the survey does this.
- **No right panel.** `--right-w: 280px` is declared and unused. `RunInspector` exists but mounts only
  in the admin Runs cockpit. Run traces render inline inside messages.
- **Session configuration hides in a sheet.** Model, tools, skills, knowledge bases, MCP and approvals
  per session are behind a gear icon that is disabled until a thread exists.
- **Provider setup is spread over five places**: the Providers master-detail page, a Configure dialog
  (key + base URL, no validation, no test), `/admin/models` (orphaned — nothing links to it),
  `/admin/credentials` (orphaned; free-text provider id), Settings → Provider Overrides, plus the
  session sheet's model selector. The onboarding checklist's "Set a default model" step links to a
  page that cannot pick a model.
- **Default is a provider, not a model.** `POST /providers/{id}/default` sets a provider; the banner
  guesses `provider/models[0]` instead of reading `default_model`.
- **No test connection in the UI** although the backend has `POST /api/uar/providers/{id}/test`
  returning `{ok, latency_ms, received_text}` with 400 / 502 / 504 outcomes. Provider health
  (`/providers/health`, cooldown and consecutive errors) is fetched but never rendered.
- **Whole-object PUT** for every model toggle, silent dialog close, errors far from the form, raw ids as
  primary labels, 27 flat settings namespaces, and a full-screen "No Model Configured" gate (the
  Open WebUI anti-pattern already called out in `docs/uar-uiux-assessment-and-implementation-plan`).
- **Tauri** window is 800×600 undecorated with fake traffic lights, title still "KnowMe", default
  identifier, and no window permissions, so the app launches in its mobile layout on desktop.

## 2. Convergent anatomy across the leading AI tools

Patterns shared by three or more of ChatGPT, Codex, Claude, Claude Code, Kimi, Gemini, DeepSeek:

| Region | Convention users now expect |
|---|---|
| Sidebar top | New chat + search. ⌘⇧O new, ⌘K palette, ⌘B or ⌘⇧S toggles the sidebar. |
| Sidebar middle | Pinned → Projects → recency buckets (Today / Yesterday / Previous 7 days / Previous 30 days), infinite scroll. ChatGPT's 2026 move of history into a popover was its most-complained-about desktop change. |
| Sidebar bottom | Avatar → menu with Settings, shortcuts, account. ⌘, opens settings. |
| Agent sessions | Grouped by project/workspace, status badge, archive on hover, several in parallel. |
| Right panel | Artifact / canvas / diff opens *beside* the chat, never over it. Code–preview toggle, copy, download, version stepper, expand to full. Toggled by shortcut, not only by an X. |
| Agent inspector | Tabbed right panel: Changes, Files, Terminal, Browser, Plan, Reasoning (Kimi Code, Codex, Claude Code). Resizable; Claude Code adds drag-dock and pop-out. ⌘\ closes the focused pane. |
| Composer | `+` menu (attach, screenshot, tools, connectors), `/` commands, `@` context, dictation, mode switcher. Send appears when there is content. |
| Model picker | In or beside the composer. Three or four curated models with one-line "good for" labels, a **More models** overflow, effort/thinking as a sibling dial. Shortcut ⌃⇧M / ⌘⇧I. |
| Settings | Left tab list + detail. ChatGPT uses a modal, Claude and Cherry a full page. **No model tab** — the model is a composer concern; providers are a settings concern. |
| Desktop | Custom titlebar with the sidebar running under the traffic lights, global quick-entry hotkey (⌥Space), menu-bar/tray presence. |
| Permissions | Mode selector beside Send (Manual / Accept edits / Plan / Auto) plus inline Allow once / Always / Deny cards. |

## 3. Decisions for the UAR shell

1. **One sidebar, not two.** 272px, collapsing to a 60px icon rail (⌘B). Order: brand, New chat,
   Search, the three workspace surfaces (Knowledge, Agents, Runs), Projects, Recents by bucket,
   account row with runtime status. The old "Configure" group (Providers, MCP & tools, Skills,
   Presentations, Runtime settings) folds into Settings.
2. **Right inspector as a system, not a feature.** One `panel` region with tabs Run · Files ·
   Browser · Plan. Opens beside the thread at 440px (resizable 360–720px, width remembered), becomes a
   slide-over below 1180px, full-screen on phones. ⌘⇧D opens Run, ⌘⇧E Files, ⌘⇧B Browser, ⌘\ closes.
   Any chip in a message ("3 tool calls", "+12 −1", "report.md") opens the matching tab.
3. **Composer owns execution choices.** Left: `+` (attach, connectors, skills). Right rail: Agent chip,
   Model chip (curated list + Auto + More models), permission mode, Send. Model shows as display name
   with the `provider/model` address underneath in mono; the picker lists only enabled models.
4. **Settings is a full page with a left nav** (⌘,): General, Models & providers, Tools & MCP, Skills,
   Knowledge & memory, Governance, Appearance, Desktop, About. The 27 namespaces become sections inside
   those nine pages. URL-addressable, so "Configure provider" links land on the right panel.
5. **Chat is never gated.** With no model configured the composer still renders; the model chip shows
   "Choose a model" and opens the provider setup in the right panel, inline.

## 4. Models & providers: the design

Modeled directly on the UAR domain: catalog provider (read-only, 343 entries, 313 with a `chat`
endpoint) → configured provider (`ProviderView`: enabled, protocol auto|chat|responses, default_model,
models[] with per-model enabled, credential_configured) → credential (last-4 hint only) → routing
(default + fallbacks, failover strategy priority|round_robin|cost_optimized).

**IA: one page, master–detail.**

- **Left column**: configured providers, each row = mark, name, status pill (Connected · Key missing ·
  Unreachable · Cooldown · Disabled), model count, enable switch. Below: "+ Add provider" that opens
  the right panel with catalog search over all 343 entries, filtered to `chat`.
- **Top strip: Roles.** Default chat · Fast / background · Embeddings, each a real `provider/model`
  address with a change button. This replaces the provider-level default with a model-level one.
- **Detail**: header (name, protocol, base URL, tier badge from the support matrix family), then three
  tabs:
  - **Connection**: credential as *paste a key* or *reference an env var* (write-only; shows last 4 and
    Replace / Clear), base URL with `/v1` hint, protocol, **Test connection** with specific outcomes
    (OK + latency, 400 no credential or no enabled model, 401/403 rejected, 429 rate limited, 502 provider
    error, 504 timeout), last health check.
  - **Models**: searchable table — display name, mono address, capability chips (tools, reasoning,
    vision, structured, streaming), context, $/1M in and out, enable switch, default radio. Bulk
    enable by filter, "Sync models" (marked as a backend gap today), manual "+ Add model id".
  - **Routing**: fallback chain, failover strategy and thresholds, cost ceiling by model class
    (small 1.0 / medium 5.0 / large 15.0 / frontier uncapped, labelled provisional as in `router.rs`).
- **Add provider flow (3 steps in the right panel)**: pick from catalog → connect & test → enable
  models and set as default. Saves `POST /providers` then `POST /providers/{id}/test`; the prototype
  notes that testing an unsaved key is not possible with today's API.

**What the best tools do that we adopt**: Cherry Studio's master–detail and per-provider enable
switch; Jan's HTTP-specific test outcomes and explicit API-format field; Open WebUI's env-var and
allowlist thinking; Cursor's per-model toggles; Continue's role assignments; DeepSeek Harness's
write-only secrets with env-var references. **What we avoid**: Cherry's easy-to-miss enable switch
(status pill names the problem), Cursor's global base-URL override, Perplexity's per-thread model
reset (persist per workspace), Open WebUI's two configuration places.

## 5. Backend deltas the prototype depends on

Small, each one a single endpoint or field:

| Need | Today | Delta |
|---|---|---|
| Model-level default and roles | provider-level `/default` | `PUT /api/uar/routing/roles {chat, fast, embedding}` as `provider/model` |
| Per-model enable | whole-provider PUT | `PATCH /providers/{id}/models/{model} {enabled}` |
| Test unsaved connection | test only after save | `POST /providers/test` accepting a transient config |
| Remote model discovery | none | `POST /providers/{id}/sync-models` calling the provider's `/models` |
| Tier badge | matrix keyed by family | expose `tier` on `GET /api/catalog` rows |
| Real MCP health | `/api/uar/mcp/health` reports every server as `connected` with a tool count | add `status` (connected · reconnecting · failed · disabled), `latency_ms`, `last_checked`, `attempt` |
| Per-tool approval tier | `ToolApprovalPolicy` at run scope only | `PUT /api/uar/settings/tool-approvals` keyed by `server__tool`, with a server default; Cedar deny stays final |
| Cross-run tool audit | admission evidence per run only | `GET /api/uar/tools/audit?server=…&since=…` |
| Orchestrator routes to user agents | `orchestrator-agent` runs a fixed `RouterNode` that only picks `general-purpose` or `rust-reviewer`; the intent classifier matches skills, not agents | match each message against agent `metadata.description`/`tags` with the classifier's `accept_threshold`; emit the decision (agent, score, matched terms) as a run event so the chat can show it |
| Routing on/off per conversation | none; the closest is `agent_config.orchestrated.enabled`, which is global | `RunPolicy.routing: enabled · disabled` at conversation scope; disabled sends to `default-agent` with workspace defaults plus the conversation policy |
| Pinned agent per conversation | `agent_id` per request, `AgentSessionConfig.agent_id` per session | persist it as `RunPolicy.agent_id` on `PUT /conversations/{id}/policy` so every turn uses it |
| Knowledge base maintenance | no KB update, delete or re-embed | `PATCH` and `DELETE /knowledge-bases/{id}`, `POST /knowledge-bases/{id}/reindex` with progress events; mismatch detection already exists via `indexed_embedding` |
| A2UI registries | presentations and component library exist; design systems have no route; agents bind only through `RunPolicy.presentations` | `GET /api/uar/a2ui/design-systems`; a registry grouping over catalog, components and presentations |
| AG-UI per agent | none; AG-UI is a stream mode of a run | documented two-call recipe (`POST /runs {agent_id}` → `/stream?stream_mode=agui_spec`), or a thin `POST /api/uar/agents/{id}/agui` alias |

## 6. Tauri considerations (summarized; detail in `tauri-integration-notes.md`)

Overlay titlebar with the sidebar under the traffic lights, 1180×760 default and 960×600 minimum
so the desktop layout is the one that launches, window permissions for drag/minimize/close, ⌘, ⌘B
⌘K ⌘⇧O ⌘\ as app-level accelerators, remembered sidebar and panel widths via the store plugin,
⌥Space quick entry with a tray item, system theme sync. The prototype detects the Tauri host
(`window.__TAURI__`) and adds a 38px drag strip; all shell CSS is plain and framework-neutral so it
ports to the React app as CSS Modules or Tailwind utilities on the existing tokens.

## 7. Prototype files

- `index.html` — launcher and overview of the screens.
- `chat-workspace.html` — the shell: sidebar, thread, composer, right inspector (Run · Files · Browser · Plan).
- `models-and-providers.html` — Settings → Models & providers with the add-provider flow in the right panel.
- `tools-and-mcp.html` — Settings → Tools & MCP: native tools, MCP server health, per-tool approval tiers (auto · ask · deny · inherit, Cedar denies locked), connection and audit tabs, add-server flow in the right panel.
- `projects.html` — Workspace → Projects (proposed): work board, chats, files and linked knowledge, agent and tool ceiling, instruction stack, memory mode, members and roles, budget; design in `project-management-design.md`, schema in `proposed-schema/`.
- `knowledge.html` — Workspace → Knowledge: create knowledge bases, upload and index documents, retrieval tester, embedding-mismatch guard, re-embed and maintenance.
- `agents.html` — Workspace → Agents: compile an agent from a description (compiler session, completeness, eight-stage pipeline), attach model and fallbacks, knowledge bases, skills, tools and MCP, A2UI presentation libraries, AG-UI access recipe.
- `a2ui-registries.html` — Workspace → A2UI registries: catalogs, custom components, presentation templates and design systems; which agents may render them; surface preview and test trigger.
- `runs.html` — Workspace → Runs: live, waiting, failed and finished runs with sub-agent nesting; trace, approvals, admission evidence, usage and checkpoint resume in the inspector.
- `chat-workspace.html` also carries the conversation target: Orchestrator (routing on or off) or a pinned agent, a per-message route line, and a new-chat chooser at `chat-workspace.html?new=1` (`?agent=<id>` pins an agent).
- `skills.html` — Settings → Skills: installed skills by source, triggers, required-tool availability, scope and model override, a matching tester against the threshold, versions and Cedar-governed evolution, import flow with compiler checks.
- `knowledge-and-memory.html` — Settings → Knowledge & memory: knowledge bases (documents, retrieval tester, indexing with re-index warning) kept separate from agent memory (off by default, blend, injection, storage, forget).
- `governance.html` — Settings → Governance: effective enforcement state, Cedar policies with a request tester, pending approvals, guardrails, budget, access.
- `appearance.html` — Settings → Appearance: dark · light · high contrast · system, conversation text size, density, motion; applied live across every screen and stored per device.
- `desktop.html` — Settings → Desktop (Tauri): in-process runtime and port fallback, bundled sidecars, window, logs and data, build identity; tray, shortcuts, autostart and updater shown as not in this build.
- `assets/settings.css`, `assets/settings.js` — the settings layer shared by the new pages: one route table renders the settings nav, plus the icon sprite and shared controls (switches, segmented choices, detail tabs, master–detail rows).
- `assets/shell.css`, `assets/shell.js` — shared tokens, components and shell behaviour; `assets/fonts/` localized brand fonts; `assets/brand/` UAR marks.
- `brand-spec.md` — the token bindings; `tauri-integration-notes.md` — desktop build notes.

## Sources

Research agent sources (verified during the survey): developers.openai.com/codex/app,
learn.chatgpt.com/docs/reference/commands and /settings, help.openai.com (Canvas, Projects),
community.openai.com desktop redesign thread, code.claude.com/docs/en/desktop,
support.claude.com (Quick Entry, Artifacts), kimi.com/code/docs (interface-and-sessions,
settings-and-extensions), github.com/deepseek-ai/deepseek-harness, docs.cherryai.com.cn providers,
docs.openwebui.com connections and workspace models, jan.ai custom-endpoint docs, cursor.com/docs/models,
docs.continue.dev/reference, librechat.ai custom endpoints, docs.msty.ai online providers,
lmstudio.ai/docs/app. UAR sources: `frontend/src/app/shell/*`, `frontend/src/features/{providers,models,credentials}`,
`src/llm/{registry,router,catalog,health}.rs`, `src/uar/api/providers.rs`, `catalog/SNAPSHOT.md`,
`docs/product-support-matrix.json`, `docs/uar-next.md` §9.2, `src-tauri/tauri.conf.json`.
