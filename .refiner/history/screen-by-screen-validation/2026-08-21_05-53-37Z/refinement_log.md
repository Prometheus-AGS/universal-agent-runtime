# Refinement log — screen-by-screen-validation

## Iteration 1 — 2026-08-19T00:38:15Z

### Actions Taken

- Replaced all superseded candidates with a fresh `CI=1` certification run at
  source commit `0c8f968e`.
- Added explicit global, agent, and user memory writes/reads through the real
  MCP Streamable HTTP endpoint while retaining same-tenant user isolation.
- Bound process provenance to committed source, recursive submodule pins, free
  ports, a stable source fingerprint, and an immutable Git-tree fingerprint.
- Added the retained transcript to the bundle manifest, then regenerated the
  rendered report from that finalized manifest.
- Replayed bundle hashes, bytes, codecs, durations, paths, both fingerprints,
  the rendered report, and an intentional one-byte tamper control.
- Recorded the uncomfortable chronology: the operator approved corrected-plan
  execution first, but canonical plan revision 7 was projected after the three
  product repairs; immutable decision `screen-validation-plan-projection-lag`
  accepts the bounded delivery without making retroactive planning precedent.

### Constraint Status

- `screen-evidence-complete`: satisfied — 20 screen rows and 29 evidence paths.
- `interaction-strength`: satisfied — named primary interactions are asserted.
- `memory-and-fail-closed-controls`: satisfied — all three memory scopes and
  typed paired rejection observations pass.
- `bundle-process-source-integrity`: satisfied — provenance, report, and replay
  checks pass.
- `scope-process-and-truth`: satisfied — limits, chronology, waiver, and strict
  OpenSpec validation are explicit.

### Reflection Summary

- Convergence: terminate.
- Reason: all five blocking constraints pass and the final artifact is
  internally consistent.

### Files Modified

- Artifact-refiner manifest, constraints, state, checkpoints, log, decisions,
  and `dist/verification-summary.md`.

### Content Type

- Type: `direct:content`.
- Evaluation: `output_inspection`.

## Cycle 2, iteration 1 — 2026-08-20T06:19:59Z

### Actions Taken

- Invalidated the prior `0c8f968e` artifact after later provider/settings
  source changes made that commit ineligible as final-candidate evidence.
- Committed the strengthened product-screen interactions, including a
  reversible Providers default-route mutation.
- Ran the selected 32-scenario suite with `CI=1`, fresh processes, free ports,
  locked dependencies, and source commit `7736c797`; all 32 passed.
- Minted `docs/certifications/product-screens/7736c797`, added the retained
  process transcript to its manifest, and regenerated its report from the
  finalized manifest.
- Replayed the stable `src` fingerprint, immutable Git-tree fingerprint, 54
  artifact hashes and byte counts, 32 H.264 positive-duration videos, 20
  screenshots, duplicate-path rejection, and one-byte tamper rejection.
- Revalidated the OpenSpec change in strict mode.

### Constraint Status

- `screen-evidence-complete`: satisfied — 20 screen rows and the retained
  cross-screen evidence resolve to the final bundle.
- `interaction-strength`: satisfied — Providers now changes and restores the
  default route, and every previously strengthened interaction remains present.
- `memory-and-fail-closed-controls`: satisfied — all three memory scopes and
  typed paired rejection observations passed in the retained run.
- `bundle-process-source-integrity`: satisfied — provenance, report, hashes,
  codecs, durations, paths, both fingerprints, and tamper rejection passed.
- `scope-process-and-truth`: satisfied — profile limits, defect chronology,
  waiver, and strict OpenSpec validation remain explicit.

### Reflection Summary

- Convergence: terminate after independent history-free artifact review.
- Uncomfortable fact: the first full run at the preceding source commit failed
  because its Providers assertion looked for model text after restore instead
  of the row's restored-default marker. No bundle was minted from that run; the
  assertion was corrected and the entire selected suite reran from fresh
  processes.

### Files Modified

- Final bundle, validation matrix, OpenSpec verification, and the active
  artifact-refiner manifest, constraints, state, checkpoints, log, decisions,
  and `dist/verification-summary.md`.

### Content Type

- Type: `direct:content`.
- Evaluation: `output_inspection`.

## Cycle 3, iteration 1 — 2026-08-20T18:23:08Z

### Actions Taken

- Invalidated the `7736c797` artifact after the root lock reconciliation and
  exact-answer locator correction moved the executable candidate to
  `9859b998`.
- Preserved the failed `88edc7d5` run, which observed 31 passes and one exact
  answer failure because the locator included a valid sibling A2UI artifact.
- Retargeted the assertion to the assistant markdown text part, retained the
  exact regex, passed focused Tier 0 and the failed scenario, then committed the
  correction before certification.
- Ran the selected 32-scenario suite with `CI=1`, fresh processes, free ports,
  locked dependencies, and source commit `9859b998`; all 32 passed with no
  retry.
- Transcoded the 32 Chromium VP8 recordings into uniquely named H.264 staging
  files because the mint helper cannot copy VP8 into MP4 and repeated
  `video.webm` basenames would collide. The helper then minted the final bundle.
- Replayed the stable `src` fingerprint, immutable Git-tree fingerprint, 54
  artifact hashes and byte counts, 32 unique H.264 positive-duration videos,
  20 screenshots, duplicate-path rejection, finalized report, and one-byte
  tamper rejection.
- Revalidated the OpenSpec change in strict mode and resolved all 29 matrix
  evidence paths.

### Constraint Status

- `screen-evidence-complete`: satisfied — 20 screen rows and all 29 evidence
  paths resolve to the final bundle.
- `interaction-strength`: satisfied — every named primary interaction remains
  present in the 32/0 report.
- `memory-and-fail-closed-controls`: satisfied — the explicit memory scopes,
  exact text boundary, JWT denial, and same-tenant isolation controls passed.
- `bundle-process-source-integrity`: satisfied — provenance, report, hashes,
  codecs, durations, paths, both fingerprints, and tamper rejection passed.
- `scope-process-and-truth`: satisfied — profile limits, defect chronology,
  waiver, and strict OpenSpec validation remain explicit.

### Reflection Summary

- Convergence: terminate after independent history-free artifact review.
- Uncomfortable fact: the required mint helper cannot directly package the
  browser's VP8 files and its basename rule would collapse all recordings to
  one path. The final evidence is valid only because the staging correction is
  explicit in the retained transcript and independently replayed.

### Files Modified

- Final bundle, validation matrix, OpenSpec verification/tasks, and the active
  artifact-refiner manifest, constraints, state, checkpoints, log, decisions,
  and `dist/verification-summary.md`.

### Content Type

- Type: `direct:content`.
- Evaluation: `output_inspection`.

## Cycle 4, iteration 1 — 2026-08-21T05:36:00Z

### Actions Taken

- Superseded the `9859b998` evidence after the provider/settings and nested
  lock reconciliation moved the executable candidate to `f8e203b6`.
- Created a detached fresh worktree, initialized every recursive submodule,
  completed frozen root and nested frontend installs, and observed both lock
  hashes remain unchanged.
- Built the production frontend and locked `server-full` binaries from the
  source checkout, then ran the focused Providers/Auth/MCP gate with fresh
  processes; all three scenarios passed.
- Ran the selected 32-scenario suite with `CI=1`, fresh processes, and all
  required ports free; all 32 scenarios passed in 4.8 minutes.
- Preserved and superseded a transcript cleanliness assertion that included
  expected generated `static/` outputs with an explicit committed-input check.
- Rejected three failed mint attempts, corrected only a temporary helper copy,
  and minted 32 uniquely named H.264 videos plus 20 screenshots.
- Replayed both source fingerprints, 54 artifact hashes and byte counts, video
  codecs and durations, unique video hashes, matrix links, report embedding,
  strict OpenSpec, and a one-byte tamper rejection.

### Constraint Status

- `screen-evidence-complete`: satisfied — 20 screen rows and 29 evidence paths.
- `interaction-strength`: satisfied — all named primary interactions are in
  the retained passed report and their source assertions remain explicit.
- `memory-and-fail-closed-controls`: satisfied — all three memory scopes,
  exact text, JWT denial, and same-tenant isolation controls are non-vacuous.
- `bundle-process-source-integrity`: satisfied — source, both locks,
  recursive pins, fresh processes, hashes, codecs, paths, report, and tamper
  rejection all replay.
- `scope-process-and-truth`: satisfied — profile limits, chronology, waiver,
  and strict OpenSpec validation remain explicit.

### Reflection Summary

- Convergence: terminate.
- Reason: all five blocking constraints passed deterministic replay.
- Uncomfortable fact: the repository mint helper cannot package the observed
  Playwright files without three contained corrections; the accepted evidence
  therefore depends on the final validator, not on an assumption that a
  successful helper exit implies a valid bundle.

### Files Modified

- Final `f8e203b6` bundle, matrix links, OpenSpec verification/tasks, replay
  scripts and receipts, and this artifact-refiner cycle's manifest, state,
  checkpoints, log, decisions, and summary.

### Content Type

- Type: `direct:content`.
- Evaluation: `output_inspection`.

## Cycle 5, iteration 1 — 2026-08-21T05:53:00Z

### Actions Taken

- Accepted the history-free critic's finding that cycle 4 copied constraints
  into state before the lockfile wording was strengthened.
- Corrected the evidence-commit design: the bundle report records the tested
  source, and a subsequent immutable receipt records the evidence commit.
- Started a new cycle from the canonical five constraints and copied the full
  objects, not only their IDs, into state before the Specify checkpoint.
- Replayed the bundle, OpenSpec, scoped diff, and full-object equality after
  each phase.

### Constraint Status

- All five canonical constraint objects are identical in `constraints.json`,
  state, and every progressive checkpoint.
- Bundle, interaction, fail-closed, scope, and process-source checks remain
  satisfied; no browser or source rerun was required because only evidence and
  design artifacts changed.

### Reflection Summary

- Convergence: terminate.
- Reason: the semantic constraint drift and evidence-commit self-reference are
  corrected without weakening any product assertion.
- Uncomfortable fact: comparing only constraint IDs allowed a materially
  weaker checkpoint to pass. The final validator must compare full objects.

### Files Modified

- OpenSpec design; Artifact Refiner manifest, state, checkpoints, log,
  decisions, summary, validation script, and validation receipt.

### Content Type

- Type: `direct:content`.
- Evaluation: `output_inspection`.
