## 1. Domain (`platform/entities/teams`)

- [x] 1.1 `contracts.ts`: types for TeamDefinitionSummary, TeamPackage, package export, DeploymentBinding, TeamInstance, TeamMember, TeamTask, TeamAttempt, TeamWait, TeamArtifact, PeerMessage, ExecutionOwnerView, capabilities, preflight responses, command requests, the attempt status set and its terminal subset
- [x] 1.2 `digest.ts`: canonical JSON, SHA-256 `contentDigest`, byte digest, document reference helper, with a known-answer test taken from an existing fixture document
- [x] 1.3 `api/teams-transport.ts`: collaboration API client taking the workspace as an argument, with typed errors (`unauthorized`, `workspace_required`, conflict with the runtime's `detail`, rate limited, runtime code)
- [x] 1.4 `registration.ts` and the `entities/bootstrap.ts` registration: schemas and transports, with workspace-qualified ids for workspace-scoped types
- [x] 1.5 `domain.ts`: load actions with one atomic ingest each; commands for package preflight, install and export, binding preflight and install, team create, task create, claim, admit, attempt cancel; `watchTeamRun` (3 s, 15 s backoff, artifacts and peer messages only on attempt-set change)
- [x] 1.6 `use-teams.ts`: narrow hooks (definitions list, team list, one team, members, tasks by member, attempts by member, artifacts, peer messages, execution owner, capabilities)
- [x] 1.7 Export from `platform/entities/index.ts`

## 2. Page, readiness and lists (`features/teams`)

- [x] 2.1 Workspace from the URL with a commit-on-Enter control
- [x] 2.2 `teams-page.tsx`: list plus detail, URL selection, empty states
- [x] 2.3 Readiness strip: identity, workspace, instance id, execution ownership (read-only), each ok, blocked, unknown or checking, with what it gates
- [x] 2.4 Definition detail: members, coordinator role, requested aliases, limits, budget, package, this workspace's bindings, next step
- [x] 2.5 Team detail header: status, revision, binding, budget used against effective budget, task slots left
- [x] 2.6 Nav destination and `admin-page.tsx` page map entry

## 3. Flows

- [x] 3.1 Install package sheet: files, manifest and digest check, grouped preflight diagnostics, permanent-install confirm
- [x] 3.2 Binding sheet: step one alias to provider and model; step two prefilled limits, budget and explained references; preflight diagnostics by field; install; unavailable states
- [x] 3.3 Launch sheet: definition, binding, slots, input

## 4. Routing test and run view (`features/teams/ui/run`)

- [x] 4.1 Routing test console: role (coordinator default), request, reservation beside remaining budget and task slots; create, claim, admit with resumable owed action; conflict detail
- [x] 4.2 Member lanes per the status table: attention-first order, summary counts, latest attempt open, clamped output, waiting and approvals link, blocked dependents, cancel, run trace link
- [x] 4.3 Peer messages and artifacts per member
- [x] 4.4 Start and stop `watchTeamRun` with the view; last-updated, error state, manual refresh, polite live region

## 5. Quality gate (after all of 1 to 4 are implemented)

- [x] 5.1 Two isolated Impeccable critiques and one fresh-context adversarial review of the implementation diff
- [x] 5.2 `pnpm -C frontend typecheck`, `pnpm -C frontend lint`
- [x] 5.3 Build the bundle, install it, and exercise in the in-app browser what the running service allows: readiness states, empty states, package install. Record bind, launch, routing test and run view as not exercised unless the service has an instance id and owns execution
- [x] 5.4 `openspec validate console-agent-teams`
