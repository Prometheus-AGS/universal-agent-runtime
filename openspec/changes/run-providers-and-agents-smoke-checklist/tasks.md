## 1. Setup

- [ ] 1.1 Open two Chrome windows at `http://127.0.0.1:8088/` — pending human (Chrome MCP cannot reach localhost from the connected browser instance).
- [ ] 1.2 DevTools → Network → filter `live` → confirm 10 `EventSource` connections in each tab.
- [x] 1.3 Created `smoke-log.md` with the 8 scenario templates.

## 2. Provider scenarios

- [ ] 2.1 P1 — Configure provider, observe Tab B propagation.
- [ ] 2.2 P2 — Set default, observe optimistic flip + Tab B reconcile.
- [ ] 2.3 P3 — Remove provider, observe Tab B removal.

## 3. Agent scenarios

- [ ] 3.1 A1 — Admin edits memory toggle; AgentSelector in other tab reflects (latent-bug regression guard).
- [ ] 3.2 A2 — Delete agent; both views drop the row.
- [ ] 3.3 A3 — Switch active agent in chat sidebar; header updates + next message uses new policy.

## 4. Rollback scenarios

- [ ] 4.1 R1 — Force `setDefault` rejection; observe rollback.
- [ ] 4.2 R2 — Force `patchAgent` rejection (DevTools 500 override); observe rollback.

## 5. Triage

- [ ] 5.1 For each Fail: `TaskCreate` with `phase=browser-smoke-providers-and-agents`, `entity={provider|agent}`, `scenario_id={P1|P2|...}`.
- [ ] 5.2 If <6/8 pass, escalate before `/kbd-reflect`.

## Status

**AWAITING HUMAN WALKTHROUGH.** Smoke-log template is in place at `.kbd-orchestrator/phases/browser-smoke-providers-and-agents/smoke-log.md`. Chrome MCP could not reach `127.0.0.1` from the connected browser instance, so the visual + interactive scenarios must be driven manually. Each scenario has an `Observed:` field and `Verdict:` placeholder; fill in inline and run `/kbd-reflect` once complete.
