# PLAN: allow-loopback-tools-without-jwt

Project: Universal Agent Runtime
Date: 2026-08-27
OpenSpec available: YES
Changes to implement: 1

## CHANGE LIST (ordered)

1. `allow-loopback-tools-without-jwt`: Make governance optional only for verified loopback, JWT-disabled runtimes and expose the effective state to the operator
   - Scope: boot trust boundary | runtime governance | persisted settings | API | normalized frontend state | Governance panel | release/deployment
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: L
   - Customer value: HIGH
   - Details: Add one fail-closed runtime authority that derives eligibility from the exact configured host literal, installed authentication mode, and a sealed inventory of bound tool-capable ingress. Persist and live-publish the operator preference, bypass only the three governance decisions while Off, expose authoritative status to the existing Governance panel, warn once per process, and preserve all capability/execution boundaries.

## EXECUTION ROUND ORDER

Round 1 (sequential foundation): workflow/spec alignment, then runtime control and sealed boot boundary.

Round 2 (sequential state): persisted setting, serialized Governance mutation, status API, then tool-gate bypass and runtime events.

Round 3 (sequential UI): frontend contracts/entity projection, then the hand-authored Governance master-detail panel. UI implementation begins only after the required Impeccable, frontend-design, UI/UX Pro Max, React, composition, and entity-state guidance summary is recorded.

Round 4 (quality and release): isolated UI critics and adversarial review, Tier 2, the already-authorized Tier 3 milestone, release build, LaunchAgent update, deployed behavior proof, reflection, scoped version control, verification, and archive.

The rounds are sequential because settings publication consumes the runtime authority, the gate consumes its coherent snapshot, and the UI consumes the status/mutation contract. Parallel writes across those boundaries would violate the repository's one-thing-at-a-time and single-writer build rules.

## WORK PACKAGES

1. Workflow and normative alignment — reconcile proposal/spec wording with the approved design and retain the completed provider-settings history.
2. Fail-closed runtime authority and boot boundary — implement coherent snapshots, exact eligibility, ingress proofs/tokens, storage-failure behavior, and warning cardinality.
3. Persisted setting and status API — add posture-derived seeding, serialized namespace mutation, per-key outcomes, normalization, and read-only status.
4. Tool-gate bypass — add `GovernanceBypassed`, read the snapshot before policy/Cedar/risk, and preserve On/in-flight semantics plus ordinary tool failures.
5. Frontend state/API — add typed service contracts, normalized status ingestion, boot-instance/revision acceptance, deadlines, and draft reconciliation.
6. Governance panel — implement the approved master-detail hierarchy, authoritative/draft states, warning and announcement semantics, accessibility, and responsive styling.
7. Isolated UI quality gates — run two isolated Impeccable critics, a fresh-context adversarial review, focused Playwright contract authoring, and polish.
8. Phase/release/deployment verification — run exact Rust/frontend Tier 2, rollback compatibility, authorized Tier 3, release build, install, restart, and live trust-boundary proof.
9. Completion and handoff — append memory/evidence, reflect, commit/push/PR only affected repositories not on main, verify and archive OpenSpec, and report remaining risk.

The authoritative implementation checklist is `openspec/changes/allow-loopback-tools-without-jwt/tasks.md`; its 42 checkboxes are completed and verified individually. KBD tracks the nine cohesive work packages above so cross-tool progress does not replace OpenSpec task-level evidence.

## ACCEPTANCE AND VERIFICATION

- Eligible means configured host exactly `localhost` or `127.0.0.1`, installed JWT authentication disabled, and every sealed bound ingress loopback. Configured `::1`, wildcard, private, unresolved, missing, or late ingress proof is ineligible.
- Initializing and every persistence read/seed/normalization failure gate On. Off requires a durable `false` or a successful eligible default insertion.
- Off bypasses effective run-policy denial, Cedar authorization, and risk approval only; it cannot register/select a tool, validate arguments, repair transport/provider errors, or turn an execution failure into success.
- A successful save linearizes durable write, cache, coherent snapshot, notification scheduling, and response under one Governance mutation mutex.
- The UI distinguishes Unknown, mutation unavailable, Required, durable On/Off, draft, Saving, confirmed, partial, changed-elsewhere, rejected, and timeout states without claiming success before authoritative confirmation.
- Tier 0 runs after each cohesive edit, focused Tier 1 after each work package, Tier 2 at phase completion, and the previously authorized Tier 3 only at the milestone/release gate.
- The release binary is installed only after all required local gates pass; the LaunchAgent must point to that exact verified artifact before restart.

## TRADE-OFFS AND SCOPE CUTS

- The sealed-ingress boot reorder is larger than a host-string conditional, but the simpler conditional cannot prove on-device-only reachability and would weaken the real network trust boundary.
- This change does not live-apply saved listener/JWT configuration, add IPv6 eligibility, redesign unrelated settings, alter provider/model behavior, or weaken governance for remotely reachable or JWT-required deployments.
- Existing unrelated worktree changes and existing generated KBD updates belong to the operator. Execution edits only this change's files and directly affected production/test artifacts.

## COMMANDS TO RUN

The OpenSpec change already exists and validates strictly, so do not run `/opsx:new` again.

```text
/kbd-execute allow-loopback-tools-without-jwt
```

## SYCOPHANCY REVIEW

The optional sycophancy-correction MCP tool is unavailable. Manual review rejects the flattering short path: the plan names the configured-host-only implementation as insufficient, retains storage and ingress failure modes, and cuts unrelated feature work rather than expanding the operator request.

## PLAN COMPLETE
