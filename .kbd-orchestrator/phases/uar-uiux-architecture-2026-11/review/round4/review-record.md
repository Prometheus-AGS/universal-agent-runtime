# Repair round 4: review record (2026-11-22)

Writers: lead (T1, C1 amendment, dialog-after-write, onCancel guard, guard cleanup), uar-console teammate (P1),
harness teammate (H1, H2, H3, SCN-THR-001/002). Nothing was run: no test, build, typecheck or lint.

## Review 1 (artifact-critic, diff.patch): FAIL, 1 CRITICAL
- CRITICAL reload after unawaited PGlite write could drop the rename/delete: FIXED in product. setTitle and
  removeThread return the write; the Rename dialog and Delete confirmation close only after it resolves
  (try/finally, so a write error still closes). Steps wait for the dialog to hide before reloading.
- WARNING P1 lease is global, busy per thread; A2UI lease holders never set busy: ACCEPTED, out of scope
  (pre-existing; the change makes neither worse). Follow-up.
- WARNING H2 deferred render can trail data-status: FIXED (two equal reads of the newest reply).
- WARNING H3 new checks in a shared helper: KEPT as step mechanics (the step now does what it says); the GET is
  watched so a failure names its cause. No scenario assertion changed.
- WARNING rememberActiveTitle swallowed its timeout: FIXED (shared stableThreadTitle, also in renameActiveThread).
- SUGGESTIONS C1 token split, revealOption scroll from top: FIXED. Stop during persistence: see review 2.
  Two unnamed guards in chat-message-store: REMOVED.

## Review 2 (fresh artifact-critic, diff-r2.patch): PASS, no CRITICAL
- WARNING 1 reload proof holds for the embedded store only; D1/P2 need a server snapshot BDD never sends:
  DOCUMENTED in the feature comment and the amendment limitations. Product defect stays a follow-up (P2, D1).
- WARNING 2 late generated title could overwrite a rename: RESIDUAL. /api/generate-title is local and
  deterministic (src/server.rs api_generate_title) and starts at send; only the first thread per page load is
  titled (P3), so a wait for a non-placeholder title would hang on thread B. Watch for it in the rerun.
- WARNING 3 Stop shown during persistence acts on a finished run: FIXED, onCancel returns when the thread is
  no longer live-streaming.
- SUGGESTIONS failed write is silent; Save/Delete stay enabled while pending: NOT DONE, follow-up.

Final diff: diff-r3.patch. Not re-reviewed after the review-2 fixes (onCancel guard, comments).
