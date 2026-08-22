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

The backend is self-executing OpenSpec. For each change, its `tasks.md` is the working surface; canonical status transitions go through `prometheus kbd`; verification and artifact-refiner evidence are recorded before archive. Evidence and status transitions execute in the order in `plan.md` except evidence-backed reconciliation of historical completed work. Plan revision 9 permits source preparation required by changes 12–14 to land while change 11 is active so the candidate is frozen once; it does not permit later changes' evidence or publication to run early.

## APPROVAL GATES

- Operator decision for SDK distribution scope.
- Operator authorization before any candidate or GA tag, GitHub/GHCR publication, or signing-identity use.
- External-install and operating-period evidence must be observed, not inferred.

## FALLBACK CONDITIONS

- Stop if a required design decision changes the permitted implementation surface.
- Stop at a missing external credential, publication authority, or time-bound condition.
- Plan revision 8 and decision `deployment-only-actions-local-release-certification`
  require every product, installed-artifact, supply-chain, load, stress, soak,
  and release-certification check to run locally. Stop and correct any artifact
  that assigns those checks to GitHub Actions.
- Plan revision 9 and decision `freeze-after-local-release-tail-tooling` require
  every release-tail source/tooling change to land before the immutable
  candidate is frozen. After that freeze, stop and rerun the three-hour local
  certification if any implementation, dependency, script, or product-doc file
  changes.
- Plan revision 7 and decision `screen-validation-bounded-repairs` authorize only the
  three observed Skills graph-view, approval-event projection, and Knowledge
  nested-interactive repairs inside `screen-by-screen-validation`. Stop at any other
  supported-product defect and open a narrowly-scoped follow-up.
- Do not substitute a completion-counter edit for an uncompleted change.

## VERIFICATION REQUIREMENTS

- Follow repository Tier 0, Tier 1, Tier 2, and Tier 3 timing from `AGENTS.md` and `.claude/rules/`.
- Run each change's OpenSpec verification and artifact-refiner gate before archive.
- Bind certification evidence to one immutable candidate commit; any source change restarts candidate certification.
- Retain exact local commands and outputs; GitHub Actions evidence is valid only
  for actual deployment execution or deployment-specific validation.

## PROGRESS LEDGER

- [DONE] `fix-user-isolation-sessions-memory-kb` — OpenSpec; synced, independently verified, and archived 2026-08-18
- [DONE] `harden-jwt-defaults` — OpenSpec; synced, independently verified, and archived 2026-08-18
- [DONE] `fix-skills-scope-semantics` — OpenSpec; synced, independently verified, and archived 2026-08-18
- [DONE] `ship-skill-pack-install-path` — OpenSpec; public pinned installer,
  exact default inventory proof, independent review, and archive completed
  2026-08-18
- [DONE] `emit-rag-retrieval-events` — OpenSpec; provenance, hardened retrieval,
  embedded/PostgreSQL status transitions, exact Chromium scenario, independent
  review, and archive completed 2026-08-18
- [DONE] `wire-orchestrator-delegation` — OpenSpec; orchestrator-only graph,
  attributed non-empty specialist output, recorded/live HTTP proof, independent
  review, and archive completed 2026-08-18
- [DONE] `complete-agui-event-parity` — reconciled archived evidence
- [DONE] `resolve-sdk-distribution` — OpenSpec; all three SDKs selected at
  1.0.0/MIT, metadata/docs/local verification reconciled, legacy routine CI
  retired, Rust publication prerequisite chain recorded, independently reviewed,
  and archived 2026-08-18
- [DONE] `rewrite-readme-and-docs` — OpenSpec; customer docs, rendered Mermaid,
  runtime-aligned OpenAPI, exact root cleanup, focused verification,
  independent review, canonical spec sync, and archive completed 2026-08-18
- [DONE] `screen-by-screen-validation` — OpenSpec; three operator-approved bounded
  repairs, immutable local browser certification, independent review, sync, and
  archive completed 2026-08-21
- [CANCELLED] `certify-operational-resilience` — superseded by operator revision 11; not passed
- [CANCELLED] `produce-supply-chain-artifacts` — superseded by operator revision 11; not passed
- [CANCELLED] `certify-release-candidate` — superseded by operator revision 11; not passed
- [CANCELLED] `release-1-0-0` — superseded by operator revision 11; no publication performed

## REVISION 11 FUNCTIONAL CLOSEOUT

The phase closes only on the five operator-selected real-model functional paths,
each exercised through both the packaged API boundary and the shipped UI:
OpenAI proxy inference, skill activation, knowledge grounding, Kimi k3 provider
configuration and inference, and basic-agent creation and inference. The observed
results are recorded in `functional-verification.md`. No unit, synthetic,
recorded, soak, supply-chain, RC, GA, or GitHub Actions test belongs to this
closeout.

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
