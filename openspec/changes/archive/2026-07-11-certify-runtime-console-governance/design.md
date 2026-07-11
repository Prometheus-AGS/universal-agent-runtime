## Context

The tool gate currently treats a Cedar deny as a request for human approval.
That lets an operator override an explicit policy prohibition and makes Console
approval state semantically ambiguous. The Runtime Console also mixes direct
page fetches and graph mutations with store-backed state.

## Decisions

### Governance has three non-overlapping outcomes

`Allow` executes immediately, `RequireApproval` creates exactly one pending
human decision, and `Deny` rejects immediately. Cedar forbid/default-deny maps
only to `Deny`; risk heuristics can elevate a Cedar allow to
`RequireApproval`. Human approval can resolve only the middle state.

### Events and APIs preserve the outcome

Approval-required and policy-denied events are distinct and carry the tool call
identity and reason. Resolution APIs report the resolved outcome and return 404
for absent, denied, terminal, timed-out, or already-resolved gates.

### Console state follows the React contract

Services own HTTP, stores own polling/replay/graph ingestion, hooks expose
narrow projections, and pages render. Live and replayed AG-UI frames enter
through the same validated adapter.

### The HTTP default is 1906, with explicit overrides

The compiled runtime, local proxy, containers, cluster manifests, SDK examples,
and operator documentation share port `1906` as their no-configuration HTTP
contract. `--port`, `PORT`, and `UAR_SERVER__PORT` remain supported so the
default does not become a deployment constraint.

## UI/UX routing distillation

The required memory consult (unavailable endpoint, recorded as no prior
context), UI/UX Pro Max audit, Impeccable audit/critique/harden/polish,
`frontend-design`, React Best Practices, and Composition Patterns reviews all
pointed to the same task-specific approach: keep the dense operations console
as a projection of one correlated entity graph, preserve stable navigation and
empty/error states across desktop and mobile, expose approval intent with
unambiguous controls, and keep I/O below the component/hook boundary. The
command palette must retain an actual `cmdk` root so its keyboard and dialog
semantics remain accessible and reliable.

## Verification

Unit tests cover allow, approval, deny, timeout, channel close, and duplicate
resolution. Browser tests cover correlated live/error state across Cockpit,
Protocols, Runs, and Approvals. Boundary and full release gates finish the
change.
