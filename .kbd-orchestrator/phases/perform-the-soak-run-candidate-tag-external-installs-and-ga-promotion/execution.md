# EXECUTION: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

Project: universal-agent-runtime
Date: 2026-08-18
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The phase has spec-backed security, product, distribution, certification, and publication changes whose task and evidence history must remain inspectable.
Backend entrypoint: `/kbd-execute perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/plan.md`

## EXECUTION SCOPE

- `fix-user-isolation-sessions-memory-kb`: partition threads, memory, and knowledge by verified user identity.
- `harden-jwt-defaults`: reject the fallback secret when JWT is required and retain configured claim validation.
- `fix-skills-scope-semantics`: reconcile any original scope/UI requirements not delivered by `skill-scoped-governance`.
- `ship-skill-pack-install-path`: provide a customer installation path for the skill pack.
- `emit-rag-retrieval-events`: expose retrieval evidence and correct ingestion status behavior.
- `wire-orchestrator-delegation`: make the advertised orchestrator delegation path real or remove the unsupported claim.
- `complete-agui-event-parity`: reconciled complete from the archived 2026-08-07 change.
- `resolve-sdk-distribution`: make and execute the operator-owned SDK scope decision.
- `rewrite-readme-and-docs`: align customer documentation with the delivered product.
- `screen-by-screen-validation`: capture the planned screen and workflow evidence.
- `certify-operational-resilience`: rerun immutable-candidate resilience evidence.
- `produce-supply-chain-artifacts`: produce and independently verify release artifacts.
- `certify-release-candidate`: certify unchanged candidate source, including time-bound external evidence.
- `release-1-0-0`: promote only the certified source through an operator-authorized publication action.

## DISPATCH CONTRACTS

The backend is self-executing OpenSpec. For each change, its `tasks.md` is the working surface; canonical status transitions go through `prometheus kbd`; verification and artifact-refiner evidence are recorded before archive. Changes execute in the order in `plan.md` except evidence-backed reconciliation of historical completed work.

## APPROVAL GATES

- Operator decision for SDK distribution scope.
- Operator authorization before any candidate or GA tag, GitHub/GHCR publication, or signing-identity use.
- External-install and operating-period evidence must be observed, not inferred.

## FALLBACK CONDITIONS

- Stop if a required design decision changes the permitted implementation surface.
- Stop at a missing external credential, publication authority, time-bound condition, or supported-product defect.
- Do not substitute a completion-counter edit for an uncompleted change.

## VERIFICATION REQUIREMENTS

- Follow repository Tier 0, Tier 1, Tier 2, and Tier 3 timing from `AGENTS.md` and `.claude/rules/`.
- Run each change's OpenSpec verification and artifact-refiner gate before archive.
- Bind certification evidence to one immutable candidate commit; any source change restarts candidate certification.

## PROGRESS LEDGER

- [DONE] `fix-user-isolation-sessions-memory-kb` — OpenSpec; synced, independently verified, and archived 2026-08-18
- [DONE] `harden-jwt-defaults` — OpenSpec; synced, independently verified, and archived 2026-08-18
- [PENDING] `fix-skills-scope-semantics` — OpenSpec
- [PENDING] `ship-skill-pack-install-path` — OpenSpec
- [PENDING] `emit-rag-retrieval-events` — OpenSpec
- [PENDING] `wire-orchestrator-delegation` — OpenSpec
- [DONE] `complete-agui-event-parity` — reconciled archived evidence
- [PENDING] `resolve-sdk-distribution` — OpenSpec and operator decision
- [PENDING] `rewrite-readme-and-docs` — OpenSpec
- [PENDING] `screen-by-screen-validation` — OpenSpec
- [PENDING] `certify-operational-resilience` — OpenSpec evidence
- [PENDING] `produce-supply-chain-artifacts` — OpenSpec evidence
- [PENDING] `certify-release-candidate` — OpenSpec evidence and time-bound validation
- [PENDING] `release-1-0-0` — OpenSpec publication

## OUTPUTS

- OpenSpec implementation, verification, archive, and artifact-refiner artifacts per change.
- Canonical KBD events and generated waypoint/progress projections.
- Immutable candidate and release evidence only after the stated approval gates.

## BLOCKERS

- None for the first implementation change.
- Publication and time-bound gates remain intentionally unopened.

## REFLECTION HANDOFF

Reflect from the delta between this execution contract and delivered behavior, including every superseded assumption, unresolved external boundary, and the final independent critic/judge results.

## EXECUTION READY
