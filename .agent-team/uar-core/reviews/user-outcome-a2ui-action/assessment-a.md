# Isolated assessment A — standard A2UI action continuation

Execute, verification CLOSED. Reviewed only design.md in this directory and applicable root AGENTS constraints. No implementation diff, tests, browser, runtime, detector or audit CLI was inspected or executed. This is a source/design critique, not acceptance or a visual score. Only this assessment file is owned/authored by this critic; shared production edits are untouched.

The direction addresses the observed missing callback with appropriate scope: snapshot the emitting surface's current model, submit once against its actual source run, and attach the admitted continuation to canonical history. The distinction between admission and completed response is clear. Preserving existing renderer controls and typed platform entity boundaries fits the incumbent UI.

## Findings

- **P1 — Specify the global stream reservation lifecycle.** The domain paragraph promises “one pending submission globally” while the stream paragraph attaches an admitted response and the UI remains disabled through running. Pending POST ownership alone does not specify who owns the global stream slot after admission, on attach failure, cancellation, transport loss, or uncertain POST outcome. Correction: reserve before the POST using the same authority that normal chat start consults; retain ownership through attachment/running; define release on each terminal/failure path without cancelling another run. Treat uncertain POST as a consumed operation for that surface, while explicitly defining whether another unrelated chat can begin. Otherwise a second surface or normal composer can race the admitted continuation even though the originating fieldset is disabled. This is an unresolved contract, not an observed implementation defect.

- **P2 — Native fieldset disabling needs an action-boundary guarantee.** The artifact says native disabled behavior protects the renderer/native catalog, but does not establish that every upstream action target is a native form descendant affected by a fieldset. Correction: keep the synchronous domain-entry guard authoritative and specify that the surface callback also ignores actions while unavailable/pending/consumed. Communicate disabled state through supported renderer semantics where available. Do not assume the wrapper alone blocks a custom component or programmatic action. Visual disabled affordance and keyboard behavior remain unverified.

- **P2 — Missing-source and post-admission failure copy need explicit placement/state text.** The artifact names an honest unavailable-action message for unbound surfaces and visible stream failure after admission, but the minimal UI contract supplies neither text. Correction: use nearby explicit text such as “Actions unavailable: this surface is not linked to a run” for unbound display-only content. After admission, distinguish “The response was interrupted” from rejected submission and unknown admission; preserve the continuation link/identity in existing inspectable metadata and provide no POST retry. This prevents the user reading an interrupted accepted response as safe to resubmit.

## Source-supported strengths and limits

The immutable action receipt in normalized graph state, typed hook boundary, transport-owned HTTP, and upstream-owned current form model avoid duplicated business state. Explicit unknown-outcome copy and absence of automatic POST retry acknowledge the backend crash/transport uncertainty honestly. No fake user message and no new palette, motion, modal or focus move limit cognitive load.

Sam's keyboard/screen-reader risks are the actual effect of disabled controls, nearby status announcement and focus retention when the clicked control becomes disabled. Riley's edge cases are repeated/programmatic actions, competing chat start, uncertain admission and interruption after acceptance. Those behaviors remain unverified; source intent cannot certify browser or assistive-technology results. No additional visible choice set or navigation is introduced.

No production guards were added by this assessment. Findings trace to the required global stream reservation, unavailable action state and accessible submission behavior. Machine/visual audit, actual edited-input submission, persisted continuation history and all runtime acceptance remain deferred by the root lock.

Questions skipped: approved intent is fixed; the requested corrections complete its action/state contract without introducing a product preference decision.
