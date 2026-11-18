## Why

The operator console has no agent-team support. A grep for "team" across `frontend/src` finds nothing, while the runtime already serves a full team API under `/api/v1/collaboration`: package install, deployment bindings, team instances, tasks, admission, execution summaries and artifacts. Today the only way to install, launch or observe a team is a script (`scripts/behaviors/steps/math-writing-team.steps.mjs`). The operator asked on 2026-11-18 to configure teams, test routing to them and see feedback per team member in the UI.

## What Changes

- Add a **Teams** destination to the console (`/admin/teams`, in the Work group next to Agents).
- **Readiness strip:** verified principal, selected workspace, whether the runtime has an instance id, and whether it owns team execution, each with what it blocks. Ownership is shown read-only; recovering it is a privileged host procedure.
- **Install:** upload a team package (manifest plus definition files), run preflight, show diagnostics, install.
- **Bind:** create a deployment binding for an installed package by choosing a provider and model for each requested model alias, plus limits and budget. The console computes the canonical `contentDigest`, runs preflight and installs.
- **Launch:** create a team instance from a definition and a binding, with member slot counts and input.
- **Routing test:** send a task to the team's coordinator role (or another chosen role) with an explicit reservation, through task create, claim and admit.
- **Per-member run view:** one lane per member showing status, current task, attempts, model, usage, output, diagnostics and peer messages, refreshed by polling while any attempt is live, with a link to each attempt's run trace.
- Team records live in the normalized entity graph behind typed hooks exported by `@/platform/entities`.

Scope decisions the operator made on 2026-11-18:

- "Configure" means install and launch. In-console authoring of team definitions is out of scope; definitions stay immutable packages.
- The routing test targets the team coordinator through the existing API. Making the global `orchestrator-agent` route chat turns to teams is a separate backend change and is not part of this change.
- Per-member feedback appears on the Teams page run view, not in the chat thread or the Runs page.

## Capabilities

### New Capabilities

- `console-agent-teams`: operator console support for installing, binding, launching, exercising and observing agent teams.

## Impact

- Frontend only: `frontend/src/platform/entities/teams/`, `frontend/src/features/teams/`, `frontend/src/app/shell/nav-destinations.ts`, `frontend/src/pages/admin-page.tsx`.
- No Rust, API, persistence or dependency changes. SHA-256 uses the browser's Web Crypto API.
- Every team route requires a verified owner, and team-instance routes require an `x-uar-workspace-id` header. The console therefore only works for teams when it is opened through an identity-providing host (for local use, `uar-jwt-proxy`).
- No live team topic exists, so the run view polls. A live topic is a backend follow-up.
- Binding, launch and routing tests need a runtime with a configured instance id that owns team execution. The service this was planned against has neither, so only install and the blocked states can be exercised there.
- Known limits, stated rather than hidden: no delete or edit of packages, bindings or teams (the API has none); `TeamDefinitionSummary` does not expose `coordinatorRole`, so the console reads it, and each agent's requested model aliases, from the package export and falls back to an explicit choice.
