EXECUTION: fix-pnpm-lock-submodule-consistency
Project: universal-agent-runtime
Date: 2026-08-20
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The observed blocker is one bounded lockfile repair whose negative control, dependency-preservation decision, evidence, and parent handoff must remain inspectable.
Backend entrypoint: `/opsx:apply fix-pnpm-lock-submodule-consistency`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency/plan.md`

EXECUTION SCOPE

- `fix-pnpm-lock-submodule-consistency`: adopt the exercised root lock candidate, prove frozen consistency without dependency drift, and return a clean source commit to parent certification.

DISPATCH CONTRACTS

- OpenSpec `tasks.md` is the working progress surface; canonical change and stage transitions go through `prometheus kbd`.
- Only the paths permitted by `scope.json` may be written. Parent certification artifacts and product source remain read-only.

APPROVAL GATES

- The operator approved creating this child and adopting the existing lock candidate.
- No publication, tag, release, or external deployment action is authorized by this child.

FALLBACK CONDITIONS

- Stop if frozen installation changes the lock digest or rejects the candidate.
- Stop if the candidate contains dependency movement beyond the recorded entity-management importer reconciliation.
- Stop if any manifest, submodule pin, product source, generated bundle, or parent certification evidence must change.

VERIFICATION REQUIREMENTS

- Retain the clean stale-lock command and non-zero output as the negative control.
- Observe frozen lock-only and full frozen installation exit 0 with identical pre/post SHA-256.
- Run `pnpm typecheck`, `pnpm lint`, scoped diff integrity, strict OpenSpec validation, and artifact-refiner validation.
- Require history-free critic and judge approval before archive.

PROGRESS LEDGER

- [COMPLETE] `fix-pnpm-lock-submodule-consistency` — Codex

OUTPUTS

- Root `pnpm-lock.yaml` repair, OpenSpec delta and verification, refiner artifact, child reflection/handoff, synced canonical spec, and one scoped commit.

BLOCKERS

- None. Parent certification remains intentionally deferred until this child
  commits a clean source candidate.

REFLECTION HANDOFF

- Record the delta between a frozen-compatible retained graph and a fresh allowed-range resolution, the child script banner defect, exact verification limits, and the parent recertification command.

EXECUTION READY
