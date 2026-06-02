# wire-runtime-console-events

## Why
The entire Runtime Console — 4 of 17 admin sections (Cockpit, Runs, Approvals, Protocols) — is a non-functional facade (assessment D4). It renders exclusively from `Runtime*` entity types (`RuntimeRun/Step/ToolCall/Approval/AgUiEvent/ModelRouteDecision/A2uiSurface`) that **no backend code ever emits** — they are fed only by a DEV-only `window` replay helper (`main.tsx:16-20`) and Vitest fixtures. In production these pages permanently show empty states, and the Approvals page's Approve/Deny buttons have **no onClick handlers at all** (`runtime-console-page.tsx:357-358`). This is the bulk of the "UI that isn't implemented" complaint, and it overlaps the observability + human-in-the-loop parity gaps (assessment D5).

## What changes
- Emit a **real runtime event stream** from the orchestrator: `RuntimeRun` (start/finish/error), `RuntimeStep`, `RuntimeToolCall` (request/result), and `RuntimeAgUiEvent`/`ModelRouteDecision` where already modeled. Source these from two places:
  1. The agent/LLM orchestration path (run + step + tool-call lifecycle).
  2. The parking-lot `Hook`/`LifecycleEvent`/`AuditSink` bus introduced by `fix-worker-pool-graceful-shutdown` (C1) for task-level events.
- Publish these as entity ChangeSets on the same realtime bus the frontend already subscribes to, so the Console populates live with zero frontend rework for the data path.
- Implement real **Approve/Deny** handlers (`runtime-console-page.tsx:357-358`) backed by a `RuntimeApproval` endpoint — a minimal human-in-the-loop gate (approve/deny a pending tool call), persisted and resolved by the run loop.
- For any `Runtime*` surface that cannot be backed this phase (e.g. `A2uiSurface`, full Protocols view), **gate or hide it** behind a feature flag rather than ship a dead panel.

## Impact
- Affected (backend): orchestrator/run manager (`src/uar/runtime/manager.rs`), event/entity bus, new approvals endpoint; reuses C1's hook bus.
- Affected (frontend): `frontend/src/admin/.../runtime-console-page.tsx` (wire Approve/Deny handlers; remove DEV-only dependency for prod), feature-flag gating for un-backed panels.
- Behavior: Cockpit/Runs show live runs; Approvals is a working HITL gate; Protocols either shows real data or is hidden.
- Risk: medium-high — emitting a full `Runtime*` model is large. **Ship a minimal real subset first** (runs + steps + tool calls + approvals) and gate the rest; do not fake-complete panels.
- Depends on: C1 (hook bus) for task-level events.
