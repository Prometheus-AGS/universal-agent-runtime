## Context

See `proposal.md`. The repository already has `frontend:boundaries`, platform
adapter checks, ESLint React hooks, and a mandatory UI/UX routing block. None
detects the exact failure path: render-body setters, per-row feature graph
mutation loops, or duplicate caches for graph-owned records. The installed
browser failure also had no focused functional scenario.

## Goals / Non-Goals

**Goals:**

- Turn the relevant React and Entity Management guidance into durable project instructions.
- Add deterministic, scoped checks for the observed architecture violations.
- Certify the repaired user workflow once, locally, after code completion.

**Non-Goals:**

- A heuristic ban on `useState`, Zustand, setters, effects, or rerenders.
- A repository-wide codemod or broad historical cleanup.
- Product testing in GitHub Actions, load testing, or soak testing.

## Decisions

### Extend the existing boundary gate instead of adding another checker stack

Extend `scripts/check-frontend-boundaries.mjs` and its negative-fixture runner
with stable rule identifiers for:

- a state setter invoked in a React component's render body;
- a loop in feature code whose body calls a raw graph mutation;
- a direct entity-management package import outside `platform/entities`;
- a named duplicate Zustand cache for configured Provider, Model,
  AgentSession, or AgentSessionDraft business records.

Use syntax-aware inspection when the existing parser supports it; do not rely on
unbounded text matching that rejects comments or unrelated names. Add one
failing fixture per rule and one allowed fixture showing UI-local popover state
plus an event-driven domain action. The existing `frontend:boundaries` command
remains the single entry point.

### Add one entity-first layering contract to all active instruction surfaces

Add concise project prose to both `AGENTS.md` and `CLAUDE.md`, and replace the
single-path wording in `.claude/rules/typescript.md` with two explicit legal
paths:

1. server business entities: component → domain hook/view model → Entity
   Management graph → registered transport → thin API service;
2. transient UI/process state: component → hook → Zustand store → thin service.

React/entity-state changes must consult the Vercel React Best Practices and
Composition Patterns skills plus the applicable Prometheus Entity Management
skill; business entity state belongs behind platform domain hooks; field
subscriptions must be narrow; and raw per-row graph writes from feature code are
forbidden. Components still import neither stores nor services, and hooks still
perform no direct fetch. Existing managed UI/UX routing remains untouched.

### Use one installed-browser scenario as the functional gate

Add one focused Playwright scenario against the installed release service. It
captures timing, console, network, computed styles, API state, the provider/model
selected by a genuine inference response, and the matching server-log interval.
It runs only after changes 1 and 2 and this change's instruction, gate, and
fixture implementation are code-complete. A two-second open bound is a
regression threshold for the local certification host, not a general
runtime-level performance claim.

The scenario is local and explicit. It is not added to `.github/workflows/**` and
does not invoke a broad suite.

## Risks / Trade-offs

- **Static checks produce false positives** → Scope rules to known frontend layers, parse syntax, and include allowed fixtures.
- **An instruction duplicates managed UI routing** → Add only the missing entity-state invariants outside managed markers and reference the existing skill names.
- **Inference is blocked by provider availability** → Report the exact unmet functional requirement; do not replace it with a mocked assertion or declare completion.
- **Timing varies by host** → Record host/source/profile limits and use the threshold only for this local certification lane.

## Migration Plan

1. Add the two-path standing instructions and boundary rules with observed failing negative fixtures.
2. Confirm the repaired source passes the scoped gate.
3. After code completion, build/install once and run the single browser scenario.
4. Remove no existing validation and change no GitHub Actions workflow.
