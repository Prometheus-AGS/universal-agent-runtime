# C09.1: Durable team planning board

## Why

The official collaboration catalog accepts immutable TeamDefinition documents, but no owner can create a durable team record or task dependency board. The Agent Fabric Convergence C09.1 deliverable requires those records before C09.2 can add claims and execution. The Boss needs a scoped administration interface that can enumerate exact definitions and show what has been planned.

## What changes

- Retain one exact TeamDefinition and its private DeploymentBinding revision in a durable TeamInstance.
- Materialize bounded member slots and revisioned queued tasks with input, output contract, and dependencies.
- Expose authenticated, workspace-scoped REST administration and honest planning-only capability metadata.
- Allow a TeamDefinition-only package to receive a private binding with execution explicitly unsupported.

## Scope

Repository: universal-agent-runtime. Initiative: `afc-c09-bounded-teams-and-shared-task-board`, task C09.1. Source ownership: `src/uar/domain/{collaboration,team_planning}.rs`, `src/uar/compiler/collaboration/{bindings,mod,team_planning}.rs`, `src/uar/api/{capabilities,administration_capabilities,collaboration}.rs`, `src/uar/api/collaboration/team_planning.rs`, and the draft.2 DeploymentBinding schema/README. C09.2 task claims, execution, routing and approval authority remain outside this change.

## Risk

The collaboration catalog persists one atomic state value, so a growing board increases its transaction payload. This is the existing storage boundary, appropriate for the bounded first slice; capacity and executor storage partitioning belong to the complete C09 gate. Planning records must never advertise a runnable team while execution is unavailable.
