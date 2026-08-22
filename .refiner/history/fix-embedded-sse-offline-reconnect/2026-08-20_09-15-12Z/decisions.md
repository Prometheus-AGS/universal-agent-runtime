# Decisions — `fix-embedded-sse-offline-reconnect`

## Iteration 1 — 2026-08-20T08:55:43Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the final transport is small and bounded; the broader-looking
  work is the observed upstream source correction and reproducible build path,
  not a consumer workaround.
- **Uncomfortable result:** `pnpm test` still exits 1 on 10 unrelated A2UI
  Storybook validation cases. This artifact records the failure and makes no
  full-suite pass claim.
- **Independent review:** pending artifact critic and judge.
- **Publication:** upstream source/compatibility and generated rc.2 PRs are
  open. No npm publication, tag, or dist-tag occurred.

## Iteration 2 — 2026-08-20T09:05:45Z

- **Decision:** continue after correcting implementation and artifact blockers.
- **Iteration:** 2 of 5.
- **Blocking violations found by independent review:** 4: post-unsubscribe
  delivery, scalar-record acceptance, understated dependency-pin impact, and
  non-chronological checkpoints.
- **Rationale:** the first three are minimal source/spec corrections. The
  checkpoint files are discarded and rebuilt progressively in iteration 3;
  preserving them would turn a known false receipt into repository history.
- **Independent review:** first critic and judge both BLOCK.

## Iteration 3 — 2026-08-20T09:05:45Z

- **Decision:** continue through progressive checkpoints and fresh independent
  review.
- **Iteration:** 3 of 5.
- **Blocking violations remaining locally:** 0.
- **Independent review:** corrected-candidate artifact critic PASS with zero
  findings; independent judge PASS with no blocker.
- **Final decision:** terminate iteration 3 with 5/5 constraints satisfied.
- **Commit exclusions:** `.claude/settings.local.json`, both UAR lockfiles,
  static build output, parent screen-validation/certification files, and
  unrelated KBD projection churn remain outside this child.
