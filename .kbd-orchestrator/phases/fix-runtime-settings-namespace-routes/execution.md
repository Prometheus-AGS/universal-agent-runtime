EXECUTION: fix-runtime-settings-namespace-routes
Project: universal-agent-runtime
Date: 2026-08-25
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The repository requires spec-backed traceability, and this phase already has one validated OpenSpec change with a bounded implementation and local verification surface.
Backend entrypoint: `/opsx:apply fix-settings-namespace-read-routes`
OpenSpec available: YES
Source plan: .kbd-orchestrator/phases/fix-runtime-settings-namespace-routes/plan.md

EXECUTION SCOPE

- fix-settings-namespace-read-routes: merge the review branch baseline, pin the exact KBD rollover commit, canonicalize settings namespace GET routes, add focused and installed-service regression proof, certify locally, install, and capture live evidence.

DISPATCH CONTRACTS

- fix-settings-namespace-read-routes → Codex
  Entry: `/opsx:apply fix-settings-namespace-read-routes`
  Progress file: `.kbd-orchestrator/phases/fix-runtime-settings-namespace-routes/progress.json`
  Handoff: Update OpenSpec tasks and canonical KBD tasks after each verified unit; report blockers through KBD.

APPROVAL GATES

- The operator-approved plan authorizes the origin/main merge, exact upstream gitlink pin, native rebuild/install, UAR LaunchAgent restart through the installer, and pushes of the two review branches only.

FALLBACK CONDITIONS

- Stop if the origin/main merge conflicts, KBD audit becomes conflicted or migration-dirty, provider identity changes across installation, or the existing installer cannot preserve configuration/static-bundle backups.

VERIFICATION REQUIREMENTS

- `pnpm typecheck`
- `pnpm lint`
- focused settings API Vitest command
- `pnpm frontend:boundaries`
- `pnpm test`
- `pnpm build`
- `node scripts/validate-static-bundle.mjs static`
- `openspec validate fix-settings-namespace-read-routes --strict`
- `cargo build --locked --release --no-default-features --features server-full`
- installed-service Playwright proof against port 1906 plus health, readiness, route, provider identity, console, and network checks

PROGRESS LEDGER

- IN_PROGRESS fix-settings-namespace-read-routes — Codex

OUTPUTS

- OpenSpec change `fix-settings-namespace-read-routes`
- frontend API regression test
- installed-service Playwright regression test
- native release installation and `.prometheus/` evidence

BLOCKERS

- NONE

REFLECTION HANDOFF

- Compare planned and delivered files, list every local gate with observed results, compare pre/post provider IDs and count, record installer/service evidence, and name any residual risk before completing Reflect and the phase.

EXECUTION READY
