# Plan — `browser-smoke-providers-and-agents`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/browser-smoke-providers-and-agents/assessment.md`

---

## Decisions locked (defaults applied)

| Q | Answer |
|---|--------|
| Q1 — browser | **Chrome** (MCP-probe friendly; same browser used for the prior screenshot smoke) |
| Q2 — force-failure technique | **Natural API rejection** when available; fall back to DevTools body edit only if no natural failure path exists |
| Q3 — smoke output | **Separate `smoke-log.md`** in the phase directory; reflection.md links to it |
| Q4 — regression triage | **Finish all 8 checks**, then batch-triage any failures as `TaskCreate` entries before reflect |

---

## Ordered change list (2 changes)

| # | Change ID | Title | Notes |
|---|-----------|-------|-------|
| 1 | `refresh-deployed-bundle-for-smoke` | Rebuild frontend + sync to `~/.uar/static/` + restart UAR — required because the running bundle predates the bridge bug fix | mechanical |
| 2 | `run-providers-and-agents-smoke-checklist` | Walk the 8 scenarios (P1–P3, A1–A3, R1–R2) in two browser windows; log each to `smoke-log.md` | manual; ~30 min |

Change 1 unblocks change 2. Change 2 is the actual smoke session.

---

## Per-change synopsis

### 1. `refresh-deployed-bundle-for-smoke`
Mechanical. No code changes — just rebuild + redeploy:

```bash
cd frontend
pnpm install --frozen-lockfile     # idempotent; in case lockfile drifted
pnpm run build                     # writes new bundle to ../static
cp -R ../static/* ~/.uar/static/   # mirror to UAR's static dir
PID=$(lsof -nP -tiTCP:1906 -sTCP:LISTEN)
kill "$PID"
sleep 2
cd ~ && UAR_BUILTIN_SKILLS_DIR=/Users/gqadonis/Projects/prometheus/universal-agent-runtime/crates/prometheus-skill-system/skills \
  nohup universal-agent-runtime > /tmp/uar.log 2>&1 &
# wait for /health
```

Acceptance: `grep -oE 'index-[A-Za-z0-9_-]+\.js' ~/.uar/static/index.html` returns a hash **different from** `index-Bg0JK_oV.js`.

### 2. `run-providers-and-agents-smoke-checklist`
Open two Chrome windows at `http://127.0.0.1:8088/` (through the JWT proxy). Walk these 8 scenarios in order, appending to `.kbd-orchestrator/phases/browser-smoke-providers-and-agents/smoke-log.md`:

```markdown
# Smoke Log — Providers + Agents Direct Migrations

## P1 Configure provider (cross-tab propagation)
- Tab A: Admin → Providers → click [+] on an unconfigured provider → submit valid api_key
- Tab B: Admin → Providers list (open before P1 starts)
- Expected: Tab B shows the newly-configured row within ~200 ms with no manual refresh
- Observed: ___
- Pass / Fail: ___

## P2 Set default provider (optimistic flip)
- Tab A: click "Set as default" on a configured provider that isn't the current default
- Tab B: Admin → Providers (open before)
- Expected (Tab A): default badge moves instantly (<1 frame); SSE reconciles
- Expected (Tab B): default badge moves on the new provider within ~200 ms
- Observed: ___
- Pass / Fail: ___

## P3 Remove provider (cross-tab removal)
- Tab A: click trash on a configured provider, confirm
- Tab B: Admin → Providers (open before)
- Expected (Tab A): row vanishes instantly
- Expected (Tab B): row vanishes within ~200 ms
- Observed: ___
- Pass / Fail: ___

## A1 Edit agent memory toggle (latent-bug regression guard)
- Tab A: Admin → Agents → pick an agent → flip "Memory Enabled" → save
- Tab B: Chat → open the AgentSelector dropdown (don't pick anything; just observe the list)
- Expected: Tab B's selector dropdown does NOT need a refresh; opening it after Tab A's save shows the agent's updated metadata
- Note: this is the latent-bug fix — before this phase's migration, Tab B's selector cached locally and ignored SSE
- Observed: ___
- Pass / Fail: ___

## A2 Delete agent (cross-tab removal)
- Tab A: Admin → Agents → pick an agent → click trash → confirm
- Tab B: open AgentSelector + Admin → Agents list (both windows visible)
- Expected (both Tab A and Tab B): row disappears from both admin list and selector dropdown ≤200 ms
- Observed: ___
- Pass / Fail: ___

## A3 Switch active agent in chat sidebar
- Tab A: Chat → AgentSelector → pick a non-default agent
- Expected: chat header model badge flips to the new agent's model; sending a chat message uses the new agent's policy (verifiable via UAR logs or via a model-specific reply pattern)
- Observed: ___
- Pass / Fail: ___

## R1 Force setDefault rejection (optimistic rollback)
- Setup: identify an unconfigured provider in the catalog (one with `configured: false`).
- Tab A: try "Set as default" on the unconfigured provider via direct API call OR via the UI if the button is exposed. (If the UI gates set-default to configured-only, use curl with the proxy JWT to hit `POST /api/uar/providers/<unconfigured-id>/default`.)
- Expected: optimistic flip happens for a frame, then UI reverts to the prior default
- Fallback: if no natural failure path, use DevTools to intercept the fetch and return a 500
- Observed: ___
- Pass / Fail: ___

## R2 Force patchAgent rejection (optimistic rollback)
- Tab A: DevTools → Network panel → "Override response" the next `PATCH /api/agents/{id}` with a 500 response
- Flip the memory toggle and save
- Expected: optimistic toggle revert; error toast/message shows
- Observed: ___
- Pass / Fail: ___
```

Acceptance: all 8 sections have entries, with `Observed` filled in and a Pass/Fail verdict each.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Set-default rejection R1 has no natural path (backend accepts unconfigured-id) | Fall back to DevTools network override; if neither path produces failure, mark R1 inconclusive and file a backend task |
| The "force PATCH 500" technique requires Chrome DevTools Local Overrides | Document the technique inline; if unfamiliar, use ModHeader extension or temporary backend stub |
| Tab B was opened BEFORE the bundle refresh — running stale JS | Hard-reload (Cmd+Shift+R) both tabs after change 1 lands and before P1 begins |
| Surreal or the bus aren't actually delivering SSE | Pre-check: open DevTools → Network → filter "live" → confirm 10 active `EventSource` connections |
| 30 min target slips into deeper investigation | Per Q4 decision: finish the 8 checks first, then triage |

---

## Acceptance gate before phase reflect

1. `smoke-log.md` exists and has 8 filled-in sections.
2. At least 6 of 8 scenarios Pass (75% baseline). Below that, escalate before reflect.
3. Every Fail logged as a `TaskCreate` task tagged with phase + entity + scenario id.

---

## Progress signal

Completed kbd-plan — browser-smoke-providers-and-agents
