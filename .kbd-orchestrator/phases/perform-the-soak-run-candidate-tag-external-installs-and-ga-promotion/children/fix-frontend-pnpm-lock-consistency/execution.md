EXECUTION: fix-frontend-pnpm-lock-consistency
Project: universal-agent-runtime
Date: 2026-08-20
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The blocker is one bounded nested-lock repair whose stale-lock control, minimum-delta decision, clean installation, and parent handoff require spec-backed traceability.
Backend entrypoint: `/opsx:apply fix-frontend-pnpm-lock-consistency`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency/plan.md`

EXECUTION SCOPE

- `fix-frontend-pnpm-lock-consistency`: reconcile the nested frontend lock with
  the committed workspace manifests, preserve unrelated common resolutions, and
  return one clean child commit to parent screen certification.

DISPATCH CONTRACTS

- OpenSpec `tasks.md` is the implementation ledger; canonical change and stage
  transitions go through `prometheus kbd`.
- Only paths permitted by child `scope.json` may be written. Parent screen
  evidence, product source, manifests, the root lock, and submodule pins remain
  read-only.

APPROVAL GATES

- The operator explicitly requested execution of this child phase.
- No browser certification, publication, tag, release, deployment, or new PR is
  authorized by this child.

FALLBACK CONDITIONS

- Stop if pnpm 11.15.0 rejects the minimum-delta candidate in frozen metadata or
  empty-tree installation.
- Stop if any remaining common package or snapshot body movement cannot be tied
  to the current manifests.
- Stop if a manifest, product source file, root lock, submodule pin, generated
  asset, or parent screen evidence must change.

VERIFICATION REQUIREMENTS

- Retain the committed stale-lock command, non-zero output, and unchanged digest.
- Retain two independent clean regenerations and exact HEAD-to-candidate
  classification.
- Observe frozen metadata and empty-dependency-tree installs exit zero with
  identical pre/post nested-lock hashes.
- Observe `pnpm typecheck`, `pnpm lint`, and the focused SSE unit exit zero while
  both lock hashes remain unchanged.
- Run strict OpenSpec, scoped diff, artifact-refiner, and history-free
  critic/judge gates before archive.

PROGRESS LEDGER

- [IN_PROGRESS] `fix-frontend-pnpm-lock-consistency` — Codex

OUTPUTS

- Nested frontend lock repair, OpenSpec delta/evidence, refiner artifact, child
  reflection/handoff, synced canonical spec, and one scoped commit.

BLOCKERS

- None. Parent certification remains intentionally paused until this child
  produces a reviewed source commit.

REFLECTION HANDOFF

- Record the delta between the full deterministic resolver output and the
  minimum-delta accepted lock, the root-versus-nested workspace blind spot, the
  exact verification limits, and the parent recertification resume point.

EXECUTION READY
