# Approved-command repair implementation inventory

Phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Execute waypoint3184, plan80. Canonical task9.6 is source-complete; acceptance
and release obligations remain pending. This document records implementation,
not acceptance.

The operator authorized the ready commands in the command proposal, serially,
and repairs of their failures. General verification stays CLOSED. No additional
checks, feature profiles, preflights or review gates are authorized.

The pinned model preparation passed. The first embedding invocation could not
find Cargo (exit127); the same command with the installed Cargo directory on
PATH compiled and failed with three fixture LowerHex errors (exit101). No test
ran. Actual output is preserved in the attempt1 log and results document.

All repair writers released their files and supplied source handoffs:

- State: `lifecycle-approved-gate-state-repair-2026-10-21.md`. Explicit digest
  byte encoding in the embedding fixture; emitted memory/persistence/SDK warnings
  corrected while preserving protocol feature branches and test-only helpers.
- Runtime: `lifecycle-approved-gate-runtime-repair-2026-10-21.md`. Emitted unused
  imports, redacted Debug implementations and test-only compatibility helpers.
- Lead: `lifecycle-approved-gate-lead-repair-2026-10-21.md`. Persistence reconnect
  initialization and the seed tool's shadowed timestamp binding.

No dependency upgrade, new guard, lifecycle ordering change or wire change was
introduced. The restricted Surreal Memory export remains untouched. Existing
unrelated work is preserved. No gate ran during these edits.

The companion JSON records the base Git object, saved-file SHA256 identities,
canonical ledgers and writer handoffs. Saved bytes include the uncommitted
repairs; they are distinct from published production32cb3bcf, Boss a6098690 and
the previously delivered Docker image. The JSON excludes itself from its digest.

No source repair remains in this observed cohort. The next gate is the exact
failed embedding integration command with only the previously recorded PATH
repair. Owned-resource and sidecar commands follow serially if preceding gates
settle. Failed gates receive source repairs and a renewed inventory before rerun.

The uncomfortable limit is that these repairs have not compiled or executed.
Passing individual commands cannot satisfy all frozen embedding conditions or
whole-phase acceptance. The full release runner, public Boss package, Windows
packet and all extra checks remain closed. The approved native candidate pin
predates the uncommitted production repairs; hold it and its dependent local
package until immutable source binding is resolved explicitly.
