---
sidebar_position: 1
title: Operate the Runtime Console
description: Read the packaged runtime projections for runs, approvals, providers, and protocols.
source_records:
  - openspec/specs/runtime-console/spec.md
  - docs/product-surface-inventory.md
current_authority: /docs/operations/runtime-console
---

# Operate the Runtime Console

The Runtime Console turns live execution signals into an inspectable entity graph. It is an operator view of the current browser and server, not a substitute for the runtime services that own execution.

:::warning Boundary statement
Console cards are projections. A visible status can be stale, incomplete, browser-local, or temporarily optimistic. Confirm consequential actions through their API response and the run's terminal event.
:::

## Packaged UI workflow

Open `/admin/runtime` for the cockpit. It summarizes active runs, tool calls, pending approvals, memory activity, the execution timeline, and provider health. Selecting **Inspect** opens `/admin/runs?run={id}`. Related admin destinations provide approvals and protocol views.

The Runs page combines a run list with its normalized trace, artifacts, and tool calls. The Approvals page can resolve pending requests. The Protocols page projects AG-UI events, A2UI surfaces, and model route decisions.

## Where the data comes from

The shared frontend entity graph receives normalized run, step, tool, approval, artifact, memory, routing, and provider events. The console also polls three current-server endpoints every 15 seconds:

- `/api/uar/providers/health` for provider health;
- `/api/uar/a2ui/schemas` for registered surfaces;
- `/api/uar/resolve-model` for the current default model route.

A provider health row reports the UAR registry's view. A model route row reports resolution from current settings and providers. Neither proves a new inference succeeded; use the [provider inference workflow](/docs/providers/inference) for that claim.

## State ownership and durability

The UAR run manager owns live server execution, cancellation tokens, event history for active process runs, and pending approval channels. The browser entity graph owns the rendered projection. The frontend also records chat run events in browser PGlite, which can support local trace inspection, but that browser store is not a server-wide audit database.

Different browser profiles and devices can therefore show different local history. Server restart, browser storage removal, or events produced outside the subscribed surfaces can leave gaps.

## Reload behavior

On reload, the console rebuilds its graph from browser-persisted run data, active streams, shared live-query updates, and the periodic server reads above. It does not issue a global server history query that reconstructs every past runtime entity. A pending in-process approval may still exist after a browser reload, but the UI must observe it again before it can render it.

## Interpret empty and degraded states

- No runs means no run entity has reached this browser graph; it does not prove the server never executed one.
- No provider health means the endpoint has not supplied a row or the request failed.
- A degraded or offline provider row reports registry health, not the cause of every model failure.
- An approval shown as approved before the request completes is optimistic UI state; failure restores the prior pending entity.

## Profile limits

The branded Runtime Console is a `server-full` capability. `minimal` can expose runtime APIs and streams without this UI. `embedded-mobile` owns its own surface, persistence, and offline lifecycle. The console makes no durability, immutable-audit, tenant-isolation, or successful-inference claim by itself.

Continue with [run inspection and cancellation](/docs/operations/runs), [approvals](/docs/governance/approvals), and [observability](/docs/operations/observability).
