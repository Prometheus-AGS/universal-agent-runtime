# Spec packet — uar-uiux-architecture-2026-11

Lead: main session. Stage: Spec (KBD). Backend: OpenSpec 2.1.0 via the managed runner.
This packet is the shared brief for every spec writer. Read it fully before writing.

## Rules for every writer

- **Spec only.** Do not edit product code. Do NOT run builds, tests, typecheck, lint,
  cargo, pnpm scripts, servers or any verification (full-phase verification boundary,
  VLBV). Reading source and grep are fine.
- **Allowed commands:** the OpenSpec runner only:
  - create: `node "$O/shared/openspec/cli.mjs" run --project "$ROOT" -- new change <change-id>`
  - validate: `node "$O/shared/openspec/cli.mjs" run --project "$ROOT" -- validate <change-id> --strict`
  - where `O=$(readlink -f ~/.claude/skills/kbd-process-orchestrator)` and
    `ROOT=/Volumes/T7/Projects/prometheus/universal-agent-runtime`.
  - `openspec validate` is an artifact-format check on your spec files, not product
    verification; it is allowed. Iterate until it passes.
- **One writer per change directory.** Write only inside `openspec/changes/<your change-ids>/`.
  Other writers share the checkout; never edit or delete anyone else's files. Do not edit
  existing open changes or `openspec/specs/**` directly.
- **Every change needs at least one spec delta** under `specs/<capability>/spec.md`
  (ADDED/MODIFIED requirements). MODIFIED must copy the full existing requirement text
  from `openspec/specs/<capability>/spec.md` and then change it. Every `#### Scenario:`
  needs a WHEN (or GIVEN) step and a THEN step.
- **Files per change:** `proposal.md`, `design.md`, `tasks.md`, `specs/<capability>/spec.md`.
- **Inputs to read:** this packet; `analysis.md` (sections: Delta from the assessment,
  Build required, Unresolved review findings); `library-candidates.json`; the relevant
  goal sections of `assessment.md`; the 2026-11-19 entries in `.prometheus/decisions.md`
  (binding operator decisions). All phase files are in
  `.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/`.
- **Verify every code claim** you put in a spec against the source at file:line. Cite it.
  If you cannot confirm it, say UNVERIFIED rather than asserting.
- **Adopt, don't rebuild.** Candidates with verdict adopt/adapt become "reuse X" tasks.

## tasks.md contract

- Numbered OpenSpec tasks (`- [ ] 1.1 …`). Group as `## 1. …` sections.
- Implementation tasks first. Put the acceptance/verification tasks in a final section
  titled `## N. Boundary acceptance (runs only at the phase boundary)`; those tasks
  describe what the final integration gate exercises. They must not be executed early.
- Stable task references are `<change-id>#<n.m>`, e.g. `uiux-config-boot-safety#2.3`.

## Acceptance (in design.md, one table per behavior)

For every behavior, include an "Acceptance" table with the fields from
`~/.claude/skills/kbd-process-orchestrator/references/templates/acceptance.template.md`:
Canonical identity, Production entry point, Real collaborators, Boundary exercised,
Observable result, Negative control, Isolation, Prerequisites, Final local gate,
Evidence (pending), Limitations. Real collaborators only: unit, mocked or structural
checks cannot establish acceptance. Tests never require, mint or send credentials or
JWTs (operator rule; external identity is provided by the existing proxy).

Also add to design.md a section `## Scenario matrix rows` listing, for each scenario ID
assigned to you below: name, actor, useful outcome, source requirement
(`openspec:<capability>#<Requirement name>`), reason now, likelihood (qualitative or
UNKNOWN), mandatory (true/false), production entry, real collaborators, oracle
(observation / expected truth source / comparator), negative control (input /
expected rejection), fixtures (scratch roots, isolation), evidence required,
substituted boundaries (or none), limitations, invalidation keys (files/configs whose
change makes the evidence stale), restoration, owner task(s). The lead copies these
rows into the phase contract, so be exact and concrete.

## Related open changes

In `proposal.md`, add `## Related open changes` listing existing open changes under
`openspec/changes/` that overlap yours, with a suggested disposition (continue,
supersede, archive) and one-line reason. Do not modify them; the operator decides in
Plan (decision 7).

## Change assignments

Scenario IDs use `SCN-<AREA>-<NNN>`. Use these IDs exactly; add more in your area if needed.

| Change id | Writer | Build items (analysis.md) | Unresolved findings to resolve | Likely capability deltas | Scenario area |
|---|---|---|---|---|---|
| `uiux-local-first-persistence-worker` | console-A | 1, 2, 5 | — | frontend-local-first-persistence | PGL |
| `uiux-workers-and-pwa-cache` | console-A | 21, 22 | — | pwa-offline | WRK |
| `uiux-model-identity-graph-ownership` | console-B | 3, 4 | record-shape warning (providers-graph.ts:84) | frontend-react-entity-guardrails | IDN |
| `uiux-schema-controls-and-pickers` | console-B | 19, 20 | critical 2 (schema panel CSV/JSON/number rendering; hand-written free-text controls, assessment.md:211-223; path picker) | frontend-configuration-surfaces | PICK |
| `uiux-settings-truthful-status-and-removals` | console-C | 10, 11, 18 | critical 3 (llm.default_provider text control leaves the settings UI); Providers banner models[0] and no-op reload endpoint warnings | frontend-configuration-surfaces, remaining-admin-surface-certification | SET |
| `uiux-design-context-and-settings-regroup` | console-C | 6, 23 | Impeccable engine pin (operator-owned; do not edit user settings) | frontend-design-authority, runtime-console-ux | DSN |
| `uiux-config-boot-safety-and-live-reload` | state-D | 7, 8, 9 | critical 4 (fallback never overrides security refusals; nothing secret persisted to disk); cand-008 is not "adopt" | new capability `configuration-boot-safety` or extend provider-model-settings-certification | CFG |
| `uiux-realtime-topics-and-replica-refresh` | state-D | 14, 17, 24 | critical 1 design (cross-replica refresh, >=2 replicas on one remote store) | embedded-sse-sync, entity-surreal-live-adapter, new `realtime-replica-consistency` | RTM |
| `uiux-system-model-live-default` | providers-E | 12, 13, 16 | sub-agent inheritance rule: specify "an agent or sub-agent's explicit model wins over conversation and system defaults" consistent with the agent-wins decision, and mark it "pending operator confirmation" | session-configuration, provider-model-settings-certification | SYS |
| `uiux-team-model-adoption-at-boundary` | runtime-F | 15 | relax the binding-revision gate only for model-binding changes; skill/package changes still block | multi-agent-orchestration | TEAM |

## Binding operator decisions (2026-11-19, summary; read the full entries)

DB wins over YAML for operator-edited values. System level = registry default provider
plus its default_model, set through the live route. An agent's explicit model beats the
system default. Running teams adopt model changes at the next task boundary; a task in
flight finishes on its model; unavailable new route fails visibly. Realtime push must
work with remote Surreal and Postgres and across replicas. Redesign authorized;
`docs/design/` is the visual authority; `docs/UI_DESIGN.md` retired. Settings no code
reads leave the UI (YAML keys stay). Overlapping open changes dispositioned in Plan.

## Handoff back to the lead

When done, reply with: change ids written, files written, `validate --strict` result per
change (exact output line), scenario IDs defined, unresolved decisions, and anything
UNVERIFIED. Keep it under 400 words.
