# Proposal

## Why

The actual packaged C14 coordinator attempt 2af51778-8096-4679-ad24-bab27337865f produced a blocked text output without delegations, waits or worker effects. Source tracing identifies native peer identities omitted from the policy universe before their attempt-bound handlers are registered.

## What Changes

- Include the four qualified attempt-native peer tool identities in both existing policy inventory passes.
- Register only peer handlers retained by the effective policy, using one central name constant.
- Preserve all policy intersections, native attempt authority and filesystem admission.

## Capabilities

### New Capabilities

- team-native-tool-catalog: Attempt-scoped peer tool inventory and policy-governed model exposure.

### Modified Capabilities

None.

## Impact

Only runtime/manager.rs and runtime/native_skills/team_tools.rs production sources change. No provider, API, persistence, dependency, host MCP allowlist, quota or scheduler change. Work can receive actual peer execution events through its existing stream when a model calls admitted tools. The source fix does not prove delegation or gateway tool calling. KBD transitions remain exclusively parent-owned. Native rebuild and packaged operation are deferred to the parent delivery boundary.
