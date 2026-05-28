# Assessment — `browser-smoke-providers-and-agents`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `fix-skills-page-utils-test-fixtures` (reflect_complete, 100%)

---

## 1. Phase goal

Manually verify the user-visible behaviour of the two direct-entity migrations shipped over the last two phases (`direct-entity-migration-providers`, `direct-entity-migration-agents`). The contract tests prove the code paths; this phase proves the UX.

Specifically: confirm that cross-tab propagation and optimistic rollback **look right** in a real browser against the running UAR — not just in a mocked `EntityChange` test.

This phase is unusual because it's **manual-only**. There's no automated test to author beyond what the prior phase already shipped; the deliverable is a checklist run-through plus a short report on what was observed.

---

## 2. Current state inventory

### 2.1 Running stack (verified before assessment)

| Component | Status |
|-----------|--------|
| UAR (PID 29699) | listening on `*:1906` |
| `uar-jwt-proxy` (PID 65218) | listening on `127.0.0.1:8088` |
| SurrealDB | reachable on `localhost:28000` |
| `surreal-memory-server` | running in Docker (per the locked decision from earlier in the session) |

### 2.2 Deployed bundle

| Path | Bundle | Built at |
|------|--------|----------|
| Repo `static/index.html` | `index-Bg0JK_oV.js` | 2026-05-27 04:09 |
| Container mirror `~/.uar/static/index.html` | `index-Bg0JK_oV.js` | 2026-05-27 04:10 |

**Drift alert:** the deployed bundle is from the **`direct-entity-migration-agents`** phase (before the bridge bug fix that landed in `vitest-contract-test-suite`). The smoke needs to run against the **post-fix** bundle, so the first step of execute will be `pnpm build && cp -R static/* ~/.uar/static/ && restart UAR`.

### 2.3 What the contract tests already proved (no need to re-validate)

- Graph propagation across two `useGraphStore` consumers (3/3).
- Optimistic snapshot/upsert/rollback for both update and remove paths (5/5).
- `useGraphBridge` fires only on watched-type mutations (3/3) — and the once-spurious initial-fire bug is fixed.
- SSE adapter event-name → `EntityChange.op` mapping (5/5).

The smoke does NOT need to re-verify any of these in isolation. It needs to verify the **integration**: real SSE from SurrealDB → graph → page render → user sees fresh data.

### 2.4 What manual smoke must cover

Six scenarios across two entities, each verified across two browser tabs (or two browser windows for full isolation):

**Providers**
- P1. Configure a new provider in tab A → tab B's Provider list shows the new row within ~200 ms (no reload).
- P2. Set a different provider as default in tab A → tab B's default badge flips instantly.
- P3. Remove a provider in tab A → tab B's Provider list drops the row within ~200 ms.

**Agents**
- A1. Edit an agent's memory toggle in tab A's Admin → tab B's AgentSelector dropdown reflects the change (the dropdown is the latent-bug fix — it was silently stale before this migration).
- A2. Delete an agent in tab A → tab B's AgentSelector + Admin list both drop the row.
- A3. Switch the selected agent in tab A's chat sidebar → the chat header model badge updates within one frame; sending a message uses the new agent's policy.

Plus two rollback smokes (one per entity):
- R1. Force a `setDefault` failure (e.g. attempt to set an unconfigured provider as default) → optimistic flip in tab A rolls back; tab B sees no change.
- R2. Force a `patchAgent` failure (e.g. send an invalid body via DevTools) → the memory toggle reverts.

### 2.5 What's NOT in scope

- Visual pixel-equivalence beyond "the page renders without React errors" — there's no screenshot baseline.
- Performance benchmarking (latency under load) — the contract tests already establish the SSE pipeline works; manual smoke is freshness, not throughput.
- Edge cases beyond the 8 scenarios above — they're the headline value props of the migrations.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | Bundle deployed at `~/.uar/static/` is from a build that includes the `use-graph-bridge.ts` fix | `grep "let last = snapshot"` in the bundle (or new hash) |
| A2 | UAR + uar-jwt-proxy + SurrealDB + surreal-memory-server all running and healthy | `curl /health` × 3 ports |
| A3 | Provider scenarios P1–P3 all pass in two-tab smoke | smoke-log entries below |
| A4 | Agent scenarios A1–A3 all pass | smoke-log entries below |
| A5 | Rollback scenarios R1–R2 both pass | smoke-log entries |
| A6 | Smoke log captured to `.kbd-orchestrator/phases/browser-smoke-providers-and-agents/smoke-log.md` with per-scenario observed/expected/pass-or-fail | file present |
| A7 | Any regressions logged as new tasks before phase reflect | `TaskCreate` per regression |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | Deployed bundle predates the bridge bug fix | **High (blocking)** | Re-build + redeploy + restart UAR before opening tabs |
| G2 | No smoke-log template in the repo | Low | Inline the scenario list in the execute phase |
| G3 | Forcing failures (R1, R2) requires either backend cooperation (e.g. set an unconfigured provider) or DevTools edits to the request body | Med | Use the natural failure: try to set-default on a provider that hasn't been configured. If the API doesn't return an error, the rollback test fails by absence — record as "rejected by server" if applicable |
| G4 | The `AgentSelector` latent-bug fix (A1) requires both tabs to be on the chat page; the admin edit happens in another tab | Low | Use two windows or two profiles — keep both open |
| G5 | The browser MCP available in this session has been unreliable for localhost; manual run is the realistic path | Med | This phase is explicitly manual — no automation expected |
| G6 | No regression triage process if a scenario fails | Med | Define ahead: failed scenario → new task with phase tag → block next migration until addressed |

---

## 5. Sequencing recommendation

1. **G1 — refresh deployment.** Build + sync + restart so the bridge fix and all contract-test era work is in the running bundle.
2. **Open two browser windows** at `http://127.0.0.1:8088/` (through the JWT proxy).
3. **Walk P1 → P2 → P3 → A1 → A2 → A3 → R1 → R2** in order, logging each.
4. **If any fail**, capture screenshot + console log, file a task, mark the scenario blocked, continue with the rest.
5. **Reflect** with goal-achievement % + the smoke log appended.

---

## 6. Open questions for the user before planning

1. **Browser** — Chrome (preferred for the MCP probes available) or whichever you usually develop in?
2. **Force-failure technique** — natural API rejection (set-default on unconfigured provider) or DevTools-injected bad body? The natural path is cleaner if available.
3. **Smoke output** — embed the log in `reflection.md` or keep as a separate `smoke-log.md`? Recommendation: separate file so the reflection stays readable.
4. **Time budget** — 30 min target for all 8 scenarios. If significant regressions show up, do we stop and triage or finish the checklist first?

---

## 7. Progress signal

Completed kbd-assess — browser-smoke-providers-and-agents
