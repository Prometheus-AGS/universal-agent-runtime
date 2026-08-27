# ASSESSMENT: allow-loopback-tools-without-jwt

Project: Universal Agent Runtime
Date: 2026-08-27
Codebase baseline: JWT-disabled requests can be admitted anonymously, but every selected in-run tool still enters the effective run-policy, Cedar, and risk-approval gate because no runtime governance master state exists.
Cross-tool progress: none for this phase; the KBD runtime created the phase at revision 892 with zero registered changes.

## IMPLEMENTATION STATUS

- Exact local eligibility predicate: **MISSING** — configuration exposes `server.host` and `security.jwt_required`, but no shared authority combines the exact configured literal, installed authentication mode, and sealed bound-listener inventory.
- Fail-closed boot boundary: **MISSING** — `src/server.rs` constructs governance and `RunManager` before the shared settings manager and binds listeners later; no Initializing state or admission token prevents a tool-capable ingress from admitting runs before governance finalization.
- Persisted `governance.enabled`: **MISSING** — `src/uar/settings/manager.rs` seeds only `default_mode`, `allowed_actions`, and `policy_reload_enabled` for the Governance namespace.
- Serialized Governance mutation and status projection: **MISSING** — the settings path has no governance-namespace mutex, posture validation, runtime publication token, boot-instance revision, mutation availability, or closed reason codes.
- God-mode tool bypass: **MISSING** — `src/uar/runtime/manager.rs` evaluates effective run-policy denial, Cedar, and heuristic approval in that order for each tool call. `ToolApprovalResult` in `src/llm/orchestrator.rs` has only `Approved` and `Rejected`, so bypass cannot be represented distinctly.
- One warning per process: **MISSING** — no `governance.inactive_local_mode` event or process-local warning guard exists.
- Governance settings UX: **MISSING** — the existing hand-authored `GovernancePanel` exposes default mode, allowed actions, and hot reload only. It has no authoritative state badge, master switch, locked reasons, inactive warning, pending-save distinction, or status revalidation.
- Existing anonymous request admission: **DONE** — the security middleware already permits an anonymous principal when JWT is not required; this phase changes downstream tool governance, not authentication admission.

## CROSS-TOOL PROGRESS

- NONE — `.kbd-orchestrator/phases/allow-loopback-tools-without-jwt/progress.json` contains no registered change or implementation work.
- Workflow note — phase creation could not reach the HTTP control plane at `127.0.0.1:7892`; the supported canonical local-runtime fallback committed revisions 891–892 and preserved the completed provider-settings phase in history.
- Progress projection defect — the newly created phase inherited completed evidence/certification/publication summaries from the prior provider-settings phase even though its counters are 0/0. Those summaries are not evidence for this phase and must not be used for completion claims.

## SPEC GAP SUMMARY

- `jwt-hardening`: anonymous admission exists, but the exact loopback-only governance-optional classification and mandatory cases are not implemented.
- `runtime-console-governance-certification`: every added requirement is unimplemented: posture-derived persistence, live On/Off state, complete three-gate bypass, fail-closed ineligible behavior, and one warning per process.
- Frontend impact estimate was superseded during design: the panel is hand-authored and effective state is runtime-derived, so a schema-only boolean cannot truthfully represent Required, Unknown, mutation unavailable, or draft-versus-durable state.
- The boot boundary spans primary HTTP, companion HTTP, and enabled A2A gRPC ingress. Configured-host checking alone would leave an unverified reachability gap.

## BUILD HEALTH

- build check: **UNKNOWN** — no build was run during fact-finding; Tier 0 begins after the first cohesive production edit.
- known violations: the new phase requirements are absent; no pre-existing build violation was inferred.
- test coverage: **NONE** for the new behavior — existing settings, security middleware, governance engine, and runtime tests do not cover the new master state or bypass.

## CONSTRAINT CHECK

- AGENTS.md violations: **NONE in new code** — no production code has been added. The existing order cannot satisfy the new sealed-ingress requirement and is therefore an implementation gap, not an unrelated policy violation.
- constraints.md violations: **N/A** — `.kbd-orchestrator/constraints.md` is absent.
- Capability inversion: the design keeps persistence and mutation in the trusted host/settings layer; the agent kernel receives no write authority.
- Scope: provider/model routing, tool registration and selection, argument validation, transports, and execution failures remain explicitly unchanged.

## GOAL PROGRESS

- Eligible exact-loopback, JWT-disabled runtimes default governance Off: **NOT MET** — no persisted master setting or eligibility authority exists.
- Operator can turn governance On or Off live: **NOT MET** — no validated runtime publication path exists.
- Governance Off bypasses policy, Cedar, and risk approval: **NOT MET** — all three gates still execute.
- Ineligible or unverifiable posture remains fail-closed: **NOT MET** — no special setting exists to validate or normalize.
- Operator receives one inactive warning per process: **NOT MET** — no warning event or cardinality guard exists.
- Existing capability and execution boundaries remain unchanged: **MET as a design constraint, unverified in implementation** — the approved design locates bypass inside the approval gate after selection and before governance decisions.

## RISK AND SCOPE CUT

- Trust-boundary risk: deriving eligibility from mutable settings rows or the configured host alone could make a remotely reachable ingress governance-optional. The implementation must use installed authentication plus a sealed inventory of successfully bound listeners.
- Consistency risk: a successful API response before durable write and coherent runtime publication could show Off while enforcement remains On, or the reverse. Governance mutations require one serialized linearization point.
- Scope cut: do not change provider compatibility, tool discovery, listener restart semantics, approval behavior while governance is On, or unrelated settings navigation.

## SYCOPHANCY REVIEW

The optional sycophancy-correction MCP tool is not available in this session. Manual S-02/S-03/S-06 review found no ungrounded feasibility claim: the assessment identifies missing behavior with file evidence, records build health as Unknown, and carries two trust-boundary risks that challenge an overly simple host-string-only implementation.

## ASSESSMENT COMPLETE
