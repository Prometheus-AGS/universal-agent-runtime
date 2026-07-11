# EXECUTION: uar-final-production-hardening-2026-07

Project: Universal Agent Runtime
Date: 2026-07-11
Selected backend: openspec
Dispatched to: Codex in isolated `production-release` worktree
Backend rationale: OpenSpec is present and all remaining production changes have validated proposals, tasks, and spec deltas. Self-execution preserves task-level traceability and KBD remains the canonical ledger.
Backend entrypoint: `/kbd-apply <change-id>` in plan round order
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/uar-final-production-hardening-2026-07/plan.md`

## EXECUTION SCOPE

Completed and preserved: seven changes through `docs-site-github-pages`.

Remaining scope:

- `establish-react-product-contract`
- `certify-provider-model-settings-flow`
- `certify-knowledge-rag-flow`
- `certify-agui-chat-flow`
- `certify-a2ui-react-flow`
- `certify-runtime-console-governance`
- `certify-remaining-admin-surfaces`
- `close-react-boundary-gate`
- `publish-capability-support-matrix`
- `modularize-release-capabilities`
- `make-build-offline-reproducible`
- `reconcile-product-documentation`
- `align-release-workflow-platforms`
- `certify-operational-resilience`
- `produce-supply-chain-artifacts`
- `certify-release-candidate`
- `release-1-0-0`

## DISPATCH CONTRACT

For every change:

1. Read waypoint, plan, proposal, tasks, and spec delta.
2. Set the change `IN_PROGRESS` in `progress.json`.
3. Implement tasks in order without expanding scope.
4. Check off each OpenSpec task only after direct verification.
5. Run project-specific tests plus `openspec validate <change-id>`.
6. For changes touching three or more files, run artifact-refiner when available; otherwise record equivalent manual constraint/build/test evidence.
7. Run OpenSpec verification, archive the change, commit it, and reconcile KBD progress/waypoint.
8. Do not advance on a blocking failure; record the blocker and focused remediation.

## APPROVAL GATES

- External account/repository settings, release tag publication, GitHub release publication, GHCR publication, signing identity, and external-adopter certification require operator approval at the point of effect.
- Dependency additions require version/security/compatibility verification and an explicit rationale.
- GA publication is forbidden unless the source commit is identical to the certified RC commit or the full RC certification is rerun.

## FALLBACK CONDITIONS

- If a change cannot stay within one agent session, split it through a new OpenSpec change and update the plan before continuing.
- If concurrent work overlaps files, stop and reconcile rather than overwriting.
- If an advertised platform or capability cannot pass its gate, downgrade it in the support matrix instead of waiving the failure.

## VERIFICATION REQUIREMENTS

- Rust: `cargo fmt --all -- --check`, supported-bundle `cargo check`, targeted tests, full library/integration suites, clippy under the repository policy.
- Frontend: `pnpm --filter ./frontend typecheck`, `test`, `build`, boundary checks, targeted Playwright journeys.
- Protocol: Rust/TypeScript golden fixtures plus live reconnect/replay/action tests.
- Docs: Docusaurus build, link/truth gates, metadata/version/license consistency.
- Release: locked offline build, supported platform artifact install/start/health, vulnerability audits, resilience reports, checksums, SBOM, provenance, signatures.

## PROGRESS LEDGER

- DONE: first seven phase changes through `docs-site-github-pages`.
- IN_PROGRESS: `establish-react-product-contract` after execution initialization.
- PENDING: all subsequent changes in plan order.

## OUTPUTS

- OpenSpec implementation and archived change per slice.
- Per-change verification evidence and commits.
- Release evidence manifest for RC and GA.

## BLOCKERS

- None at dispatch time.

## REFLECTION HANDOFF

KBD Reflect must compare every stable support-matrix row with its executable evidence, confirm all 24 changes are archived, audit release artifacts against the certified source SHA, and report any downgraded/deferred capability explicitly.

## EXECUTION READY
