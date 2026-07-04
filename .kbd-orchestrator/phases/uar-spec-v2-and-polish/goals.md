# Goals

Phase: **uar-spec-v2-and-polish**

Seeded from `uar-next-harness`'s reflection (2026-07-04). Carries forward
this project's G4 (Specification & Distribution) and G5 (Polish & Release)
goals, deliberately deferred (not abandoned) when `uar-next-harness` closed
G1-G3 at 16/24 tracked changes.

## G4 — Specification & Distribution (P2, primary — sequential chain)

- **CH-12 agent-spec-v2**: RFC + IR fields — `model_requirements`,
  `prompt_dialect`, `rag_configuration`, `context_strategy`, `api_harness`.
  Extend `AgentDescriptorIR` with backward-compatible parsing (v1.1 still
  loads). Depends on CH-04 (dialect, done) and CH-03 (routing semantics,
  done).
- **CH-13 compiler-v2-stages**: PMPO compiler stages handle the v2 fields
  (s01_frontmatter validation + stage plumbing + descriptor emit/sign for
  the five new sections; round-trip tests). Depends on CH-12.
- **CH-14 conformance-testing**: harness validating a compiled descriptor's
  declared requirements (model, dialect, tools, context) are satisfiable at
  load and honored at run. Depends on CH-13.
- **CH-15 agent-template-library**: pre-built `.agent.md` templates
  (coding, vision, terminal, data) compiled + signed in CI as release
  artifacts. Depends on CH-13.
- **CH-17 eval-targeted-suites**: skill-activation / routing /
  context-efficiency eval suites using the existing harness + scorers,
  wired into the two-tier CI gate. Depends on CH-08 (activation metrics,
  done) and CH-09 (benchmark registry, done).

## G5 — Polish & Release (P3, after G4 lands)

- **CH-19 docs-overhaul-deploy-guide**: update ARCHITECTURE/README for
  router+dialect+health, consolidated production deployment guide,
  document the D-D dependency-pin rationale, document D-B MemPalace status.
- **CH-20 perf-security-load**: profile router/dialect/context hot path,
  1000-concurrent-agent load test, prompt-injection resistance review,
  `server.rs` (~4,922 LOC) split evaluation. Findings feed a
  release-candidate decision.

## Success criteria

- Every G4/G5 item is either implemented and verified (compile + test +,
  where applicable, a live smoke check — not just "it compiles"), or
  explicitly dispositioned with rationale if scope-cut.
- `AgentDescriptorIR` v1.1 backward compatibility is preserved (Rule 32) —
  existing compiled agents must still load after the v2 fields land.
- Carried-over debt from `uar-next-harness` gets addressed or explicitly
  re-carried with a reason, not silently dropped:
  - Artifact-refiner QA gate automation (carried 4+ phases now).
  - 17 pre-existing `bun run typecheck` errors (Base UI `Select`
    nullability, `react-resizable-panels` API drift, `recharts`
    type-export drift) — unrelated to `uar-next-harness`'s own work but
    now the only remaining typecheck noise in the repo.
  - CH-06 per-agent/per-task budget *configuration* surface (aggregation
    is done; only the global limit is currently configurable).
  - CH-08 activation-outcome correlation (recall half is wired; whether a
    model actually used an activated skill's tools is unsolved).
  - Durable (non-session-scoped) cost/spend history for the CH-07
    dashboard.

## Carry-over inputs (from `uar-next-harness` reflection)

See `.kbd-orchestrator/phases/uar-next-harness/reflection.md` for full
detail: goal-by-goal MET/PARTIAL/NOT MET assessment, lessons learned
(verify "committed" against git log not just progress.json; an
uninitialized git submodule can silently gate a whole build; a library's
`@deprecated` notice doesn't always point at the fix your app's
architecture actually wants; `tailwind-merge` doesn't resolve
responsive-modifier conflicts you don't spell out).

## Operator-only follow-ups (unchanged, still open)

- Activate the eval gate: set `UAR_LLM__API_KEY` secret +
  `vars.UAR_EVAL_MODEL`, run `eval-nightly` via `workflow_dispatch
  update_baseline=true`, commit `evals/results/starter.baseline.json`.
