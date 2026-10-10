# Design

## Context

Native C14 source d8896d743cd945d40f918ff8ca397909f6c1fe22 selects team_roster, team_send, team_delegate and team_wait in compiler/collaboration/team_execution/resolution.rs. manager.rs resolves policy before registering their attempt-bound handlers. Its first universe includes only global tools; its second adds only discovered MCP identities. domain/policy.rs intersects selected identities with that universe, warning they are unavailable. The coordinator selects no filesystem tools. manager.rs then converts empty tool IDs to None and skips native peer registration. This source path explains missing model tools; the coordinator text alone is not evidence of the gateway wire request.

## Goals / Non-Goals

Make attempt-native identity inventory available before policy resolution while retaining every higher-scope restriction. Do not put native peer capabilities into the filesystem bridge, broaden ordinary agents, add a scheduler or change model routes.

## Decisions

Use the existing additional_tools universe input in both policy passes. Populate it only for a collaboration binding carrying a team attempt while team execution B is qualified. Merge discovered MCP names into that same set on the second pass. One central TEAM_TOOL_NAMES constant also drives handler registration. Registration accepts the effective tool ID set and omits every name removed by policy. Registering unconditionally after resolution would bypass denied/selected policy for BuiltIn ModelOnly descriptors and is rejected.

The existing attempt/session/owner checks in team tools and kernel roster/delegation/wait checks remain authoritative. Inventory discovery is not an authority grant. Orchestrator exposure already advertises registered BuiltIn ModelOnly descriptors, and the Liter driver forwards the assembled tools; those paths need no change.

## Risks / Trade-offs

Source correction cannot prove that the configured gateway retains tool schemas or that the model delegates. The real packaged operation must establish that after the parent rebuild. Coordinator plain blocked text currently satisfies its string output contract and may settle succeeded; this change does not redefine task outcomes.

## Migration Plan

No data migration. Parent pins the completed native source and runs one release build, package and Work operation. Rollback uses the prior source pin. No worker Cargo, test, compiler or review command is authorized.
