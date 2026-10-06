# Proposal

## Why

The actual packaged coordinator run d3d8b7e2-009e-4195-9e53-d89fe8a6f983 called team_roster and delegation but received host HTTP422, then completed without required worker/reviewer handoff. Source confirms a v1 wire-name mismatch and native coordination sent to filesystem host admission before native dispatch.

## What Changes

- Restore the PreparedToolInvocation governancePolicyRevision v1 wire name without changing authority hash fields.
- Capture explicit runtime-control ownership from trusted registered native handlers; only team peer handlers select local admission.
- Route complete admission lifecycle by captured ownership while preserving all current runtime policy, governance, budget, approval, cancellation and claim revalidators.
- Add authoritative admissionOwner to pending approvals for Boss to distinguish local approvals from bridge acknowledgments.

## Capabilities

### New Capabilities

- tool-admission-ownership: Trusted runtime control ownership and existing paired-host effect ownership.

### Modified Capabilities

None.

## Impact

UAR production: runtime/tool_admission/{mod,owned,lifecycle}.rs, runtime/native_skill.rs, runtime/native_skills/team_tools.rs, llm/orchestrator.rs, runtime/manager.rs runtime/thread/approvals.rs and persistence/tool_admission.rs. Root owns the matching Boss adapter and its schema. No new scheduler, catalog/status change, config, quota or filesystem grants. Work keeps its existing approval display and event/cursor decisions; local decisions acknowledge the UAR waiter, paired-host decisions additionally acknowledge Boss. Provider serialization unchanged. No KBD transition by this worker. Native build and real packaged operation remain parent-owned and unverified at this source boundary.
