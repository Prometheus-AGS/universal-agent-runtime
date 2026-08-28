## Why

An explicitly anonymous UAR deployment limited to the operator's own device currently accepts the request but can still deny every in-run tool call because the server attaches governance to the tool gate. A user running UAR locally with JWT disabled needs an initial governance-inactive posture so configured tools such as `web_search` work without Cedar denial or approval friction, while retaining an explicit setting that can turn governance back on.

## What Changes

- Add a persisted `governance.enabled` setting that can turn the governance engine On or Off for a UAR process eligible for governance-optional local mode.
- Classify a process as eligible only when its boot configuration names `localhost` or `127.0.0.1` exactly, the installed authentication middleware does not require JWT, and every declared tool-capable ingress has registered a successfully bound loopback address in a sealed inventory. Default `governance.enabled` to Off in that verified posture until the operator turns it On.
- While governance is Off in eligible local mode, bypass Cedar tool authorization, effective run-policy tool denial, and risk-based human approval so an available configured tool executes immediately.
- When the operator turns governance On, apply the complete existing governance and approval path to subsequent tool calls without requiring JWT or a listener change.
- Emit one structured operational warning per process, when governance is first observed inactive, and do not repeat it per request, run, or tool call.
- Keep capability boundaries outside governance unchanged: a tool must still be registered and selected, and ordinary validation, transport, provider, and execution failures still fail normally.
- Gate as On while boot posture is Initializing. Default and force governance On whenever JWT is installed as required, the configured listener literal is neither `localhost` nor `127.0.0.1`, any declared ingress is missing or late, any bound address is non-loopback, or persistence cannot safely resolve the operator preference.
- Add focused regression coverage for the local default, live On/Off behavior, one-warning contract, persistence, and both fail-closed boundary conditions.
- Update KBD workflow state for this new change and retain the existing provider-settings phase as completed history.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `jwt-hardening`: Clarify that an unauthenticated request permitted by the boot-effective JWT mode defaults to governance-inactive local tool execution only when the configured literal and sealed bound-ingress inventory also prove loopback-only reachability, unless governance is explicitly enabled.
- `runtime-console-governance-certification`: Define the persisted governance switch, the verified local-only Off default, the one-warning contract, and governance-inactive bypass of Cedar, effective run-policy tool denial, and risk approval while preserving the full governance path when enabled or outside the eligible posture.

## Impact

- Runtime UX: local anonymous chat and agent runs initially use available configured tools such as `web_search` without a governance denial or approval prompt; the existing hand-authored Governance settings panel exposes the On/Off control and its authoritative runtime status.
- Backend: the persisted settings schema, runtime governance gate, and one-time operational warning will change; focused Rust tests will cover the eligibility predicate, default, persistence, live toggle, warning cardinality, and tool-decision behavior.
- Security: governance can be Off only after boot proves the exact allowed configured literal, JWT-disabled installed authentication, and a sealed inventory in which every bound tool-capable ingress is loopback. Restart-pending settings do not change this authority. Any missing/unverified/non-loopback ingress, other configured literal, required JWT, or unresolved persistence forces the complete governance path On.
- Provider compatibility: no provider, model-routing, credential, or liter-llm behavior changes.
- Realtime state: settings updates take effect for subsequent tool calls; tool-call and completion events remain authoritative, and governance-inactive local mode intentionally emits no governance-denial or approval-required event for an otherwise executable tool.
- Dependencies and frontend: no new dependency family is introduced for the Governance UI. Release certification made the already-imported `loro-crdt` package direct and exact, and aligned the existing Zod/Vitest pins after a clean install exposed unresolved/unsupported versions. The hand-authored Governance panel, settings API, typed service/store hooks, and normalized entity projection change narrowly so the UI can distinguish authoritative On, Off, Required, Unknown, mutation-unavailable, pending-save, partial, rejected, and changed-elsewhere states instead of rendering a schema-only boolean.
- Certification scope: the branch also contains observed release-gate repairs for existing A2UI/run-trace contracts, provider test fixtures, pinned skill-pack materialization, generated static output, and the disabled-telemetry facade. Those repairs are recorded separately from the Governance feature and were required only after the authorized exact release gates failed.
- Workflow: `.kbd-orchestrator/` must register and track this change through Spec, Plan, Execute, and Reflect.
