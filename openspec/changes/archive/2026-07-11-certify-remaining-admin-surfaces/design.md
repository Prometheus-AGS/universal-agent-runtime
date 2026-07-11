## Context

The product inventory still labels every remaining administration surface
Preview, but the implementation quality is mixed: some pages already use the
required store/service boundaries while others perform I/O in components. GA
must distinguish executable customer actions from diagnostics and developer
workflows instead of treating every visible control as equally supported.

## Remaining action inventory

| Surface | Customer action contract | Current boundary | Certification disposition |
|---|---|---|---|
| Agents / editor | list, create, edit, delete, status | page imports service; builder fetches | Stable after migration and success/failure tests |
| Skills | list, create, edit, toggle, import | page/dialog import services | Stable after migration and rollback/error tests |
| Tools | discover, inspect, execute | detail component fetches | Discovery stable; execution stable only through governed runtime path |
| MCP Health | inspect configured transport health | hook/store/service | Stable diagnostic with healthy/degraded/transport-failure tests |
| Compiler | list and create sessions | page imports service | Experimental authoring workflow until compile/package lifecycle is certified |
| Memory | list, filter, stats, delete, clear | page and hook import services | Stable when enabled; disabled/empty/error states required |
| API Keys | list, create, revoke | hook/store/service | Stable after authorization and one-time-secret tests |
| Credentials | list, upsert, delete masked provider keys | page imports service | Stable after migration, masking and authorization tests |
| Cost | inspect usage and budget alerts | graph-backed projection | Stable read-only diagnostic; budget mutation is not advertised |
| Agent selector | choose the agent for a thread | component fetches | Stable after store/service migration and persistence tests |
| Session config | persist model, tools, memory and retrieval intent | component fetches | Stable after store/service migration and failure tests |

## Decisions

Stable actions remain in the customer UI only when their backing API, error
path, and reactive reconciliation are executable. Compiler is visibly
experimental because this phase does not certify the complete compilation and
package-distribution lifecycle. Tool execution uses the runtime governance path;
the Tools page cannot provide an ungoverned administrative bypass.

## UI/UX routing distillation

KBD memory recall returned the documented unreachable-endpoint stub. UI/UX Pro
Max recommends a dense drill-down operations pattern with explicit loading,
error, focus, and responsive states. The Impeccable technical audit and
single-context critique (degraded because repository policy prohibits spawning
the two critique agents) found a coherent terminal/operator vocabulary and
useful empty states, but identified four release-critical clarity issues:
component-owned I/O, an unlabelled experimental compiler lifecycle, an
ungoverned tool-execution affordance, and controls that need reliable
loading/error/disabled feedback. The deterministic detector reported no visual
anti-pattern hits. `frontend-design` favors preserving the existing operational
identity rather than restyling it; React Best Practices and Composition
Patterns reinforce narrow store subscriptions, parallel independent loads,
stable callbacks, and explicit variants instead of boolean-heavy components.
Accordingly this phase changes ownership and maturity semantics while reusing
existing tokens, typography, navigation, and shared feedback components.
