# UAR core task handoffs

Contract: [agent-team-creator task handoff reference](../../.agents/skills/agent-team-creator/references/task-handoff.md), read on 2026-09-24. Use its existing runtime. This team defines no additional transport or live peer teams.

## Operating pattern

Default activation is `uar-lead`, one of `uar-runtime`, `uar-providers`, `uar-trust-tools`, `uar-state` or `uar-console`, and `uar-verifier`. At most four agents run concurrently including lead. Each implementation assignment names disjoint exact files, requirements, evidence expectations and stop conditions.

A handoff continues an existing task. Independent verification uses a separate task owned by `uar-verifier`, with explicit `dependsOn` on completed implementation. Role dependencies do not create task dependencies. Give the verifier requirements and resulting artifacts without generation history. Reassigning the implementation task is not independent review.

## Existing entrypoint and request contract

Use Node.js 22 or newer and UTF-8 JSON request files:

```text
node .agents/skills/agent-team-creator/scripts/cli.mjs status --input STATUS_REQUEST.json
node .agents/skills/agent-team-creator/scripts/cli.mjs handoff-create --input HANDOFF_REQUEST.json
node .agents/skills/agent-team-creator/scripts/cli.mjs handoff-accept --input ACCEPT_REQUEST.json
```

Uppercase request filenames are placeholders. Read the actual state path from this team's installation/state record; do not invent or initialize a second store. Relative state, cwd and request paths resolve from the process working directory. Read status immediately before each mutation.

A creation request has top-level `state`, `expectedRevision`, `cwd`, and a `handoff` object containing all of:

- `taskId`, current `owner`, current `expectedTaskRevision`;
- `toOwner` (a registered role), `toHarness`;
- `context`, `evidence`, `remaining`, `memoryRefs`.

The final three fields are arrays and may be empty. Task actions use `task.id`; handoff creation uses `handoff.taskId`. Relevant supported harness identifiers are `codex`, `claude`, `kimi`, `opencode` and `minimax`. These identifiers do not prove those harnesses are installed, running, or have activated the role exports.

Make the packet self-contained: objective, exact owned paths, repository/branch, interface constraints, completed work, remaining work, actual checks and uncertainty. Evidence/memory references must be reachable by the destination. Include no credentials, private reasoning transcript or source-session approval claims.

## Creation, delivery and acceptance

1. Read current state/task revisions and confirm ownership. Coordinate yielding or stopping the source writer before another agent edits its files.
2. Create using current revisions. Creation records the packet and advances state revision; it does not transfer ownership or launch a harness.
3. Inspect the generated handoff ID and fresh-context prompt. The runtime captures Git root, HEAD, branch and dirty state read-only; it does not copy files or untracked artifacts. Null means unavailable, not clean. The snapshot can become stale.
4. Open or dispatch to the destination separately through an available, authorized native mechanism. Provide the prompt and reachable artifacts. This document supplies no automatic delivery mechanism and asserts no live teams in sibling repositories.
5. Destination reads its native instructions/permissions. Accept with top-level `state`, latest `expectedRevision`, generated `id`, and `destination: { owner, harness }` matching the packet.
6. Acceptance checks current source owner/harness/task revision and atomically transfers local ownership. A running task becomes pending; a blocked task stays blocked. Explicitly start the destination task before completing it.

Source identity, credentials, permissions, approvals and sandbox capabilities do not transfer. Acceptance records local coordination only. The state runtime cannot stop source processes; coordinate that through the actual harness.

## Staleness, completion and recovery

After a stale-revision failure, read status, inspect intervening changes and resubmit the intended operation with current values. Never guess revisions. Duplicate acceptance is idempotent only with a current state revision and unchanged accepted task. Old packets cannot reclaim changed or reassigned work; superseded packets remain history.

Completion needs a running task, complete dependencies, actual evidence and no remaining work. The runtime stores evidence claims but does not verify them; the verifier checks underlying artifacts/results. Terminal tasks stay terminal. Create a new task for subsequent work.

For tasks linked to canonical KBD identity, follow the contract's `complete-kbd` path and KBD workflow; ordinary task completion is refused. Local team state does not replace canonical OpenSpec/KBD authority. There is no distributed transaction: inspect both states after an uncertain external operation before retrying.

The runtime uses a local JSON state file, revision checks, an exclusive lock and same-directory atomic rename. It supplies no distributed leases, authenticated role enforcement, automatic lock takeover or multi-machine coordination. Never remove a live lock. Inspect an abandoned lock's process/time/token and verify no writer remains before authorized manual recovery. Do not use a network or cloud-sync folder as a shared mutable coordination backend.

Cross-repository work needs explicit upstream scope and a downstream integration obligation. A future peer proposal is not a registered role or accepted destination. Preserve ownership until a real registered destination accepts under the existing contract.

## Future peer request files

Before a peer team exists, write .agent-team/uar-core/handoffs/<task-id>-peer-request.json with schemaVersion: 1, id, sourceRepository, targetRepository, sourceTaskId, sourceHead, contract, requestedChanges, acceptanceEvidence, status: proposed, and replyPath. This is an inspectable proposal, not a creator handoff or permission transfer. The recipient may later return the same id with status accepted/rejected, its task/team identity, resulting commit and evidence. The UAR integration obligation stays owned locally until verified; never add an unregistered foreign owner to the creator ledger.
