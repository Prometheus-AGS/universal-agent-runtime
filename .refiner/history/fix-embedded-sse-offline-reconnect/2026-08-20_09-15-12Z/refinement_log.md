# Refinement log — `fix-embedded-sse-offline-reconnect`

## Iteration 1 — 2026-08-20T08:55:43Z

- Specify: derived five blocking constraints from the child OpenSpec and KBD
  scope: named-event mapping, single-connection recovery, upstream projection
  reactivity, reproducible source build, and truthful bounded delivery.
- Plan: keep the Rust endpoint and shared remote adapter unchanged; implement
  the subscription-local state machine, prove it at the fake transport and live
  application boundaries, and fix the normalized projection at its source.
- Execute: aligned named payload mapping; added status/retry/cleanup; replaced
  the probe/replay browser fixture; repaired both upstream view hooks; widened
  tested pnpm consumer engines; and changed BDD preparation to build core before
  React declarations.
- Reflect: focused unit 3/0, upstream hook 2/0, upstream React 58/0, UAR
  typecheck/lint/build, dependency-aware BDD preparation, and exact Chromium
  1/0 passed. The full frontend command remains 328 passed/10 failed in two
  unrelated A2UI story files and is not represented as passing.
- Persist: wrote the OpenSpec verification receipt and this direct-content
  artifact. Independent history-free critic and judge remain the convergence
  gate.
- Uncomfortable result: the first repaired transport still failed visibly
  because the upstream hooks memoized full entities behind an unchanged ID
  list. A UAR refresh would have made the browser green while leaving every
  source-package consumer stale, so the repair moved upstream.
- Initial source hashes before independent review: adapter
  `fe8f4ddb7c3534925f34cdc6c06fa63234a3fa3dcb416b99d1e69fd24ebfa135`;
  adapter test
  `3ce856ee8d7afeb32cb93b55a297c3b9883085b3b41a9da5562ae35ae56a492c`;
  feature `8a5703ea4be9a014aba9bf9fd3ec9b04d86f8d0ac97caa6f3e3670106c1ea7f8`;
  steps `ab673e4df357706bff5374daa1030dfb123d8a6fd815f7058b55a54aa93196a9`;
  root manifest
  `b612aef90c6d8d08c5d768bf08c06683d40c127b0d591c957fc6d105ad470184`;
  submodule `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`.

## Iteration 2 — 2026-08-20T09:05:45Z

- Reflect: independent critic and judge both blocked the first candidate. A
  closed fake source could still deliver because cleanup retained its named
  listener, scalar `record` values passed validation, the proposal understated
  the upstream/submodule impact, and all five checkpoints falsely contained the
  same phase-complete state.
- Execute: added a shared source disposer that removes the named listener and
  callbacks before close; rejected non-object records; added scalar, missing-ID,
  stale-predecessor, and post-unsubscribe controls; corrected proposal impact.
- Verify: post-edit typecheck exit 0, lint exit 0, and focused adapter 3/0 passed.
- Decision: continue. The implementation blockers are corrected, but the
  invalid checkpoint set is removed rather than preserved as acceptable
  evidence.

## Iteration 3 — 2026-08-20T09:05:45Z

- Specify: retain the five blocking constraints and require actual progressive
  checkpoints for this correction cycle. The first checkpoint set is deleted
  because schema validity did not make its chronology true.
- Plan: bind the corrected artifact to the new source hashes, retain both
  independent BLOCK verdicts, replay strict/schema/hash/scope checks, then
  request fresh history-free review before Reflect and Persist checkpoints.
- Execute: refreshed the verification receipt and artifact hashes after the
  cleanup/validation repair. Typecheck, lint, and the focused 3-test adapter
  file pass on the corrected candidate.
- Reflect: fresh history-free artifact critic and judge both returned PASS on
  the corrected hashes. They independently replayed focused adapter behavior,
  strict OpenSpec, schemas, five constraint IDs, three progressive checkpoints,
  source hashes, submodule pin, upstream PR separation, and scoped diff checks.
- Persist: terminate iteration 3 with all five constraints satisfied. Preserve
  the first-review BLOCK findings and the unrelated full-suite failure in the
  final artifact; finalize only after the Persist checkpoint is written.
- Current source hashes: adapter
  `6b459d814c4741f329e037e62ec6394499f5bcc767f932d6bdc874534964e2f7`;
  adapter test
  `819075f331f2d3158fff9bbff9817eabcfa7ac24a736485c1c365bd1d5c992b5`;
  feature `8a5703ea4be9a014aba9bf9fd3ec9b04d86f8d0ac97caa6f3e3670106c1ea7f8`;
  steps `ab673e4df357706bff5374daa1030dfb123d8a6fd815f7058b55a54aa93196a9`;
  root manifest
  `b612aef90c6d8d08c5d768bf08c06683d40c127b0d591c957fc6d105ad470184`;
  submodule `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`.
