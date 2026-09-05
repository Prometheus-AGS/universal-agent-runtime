# Catalog implementation source review — 2026-09-04

Artifact: Presentation platform domain, transport, draft hooks, catalog, editor and preview. Reviewers received source artifacts without generation history. Browser interaction and visual acceptance have not run; the user reserves tests for the end of the complete phase.

## Plan/delivery delta

The catalog is implemented as the approved production registry/detail-pane extension. The existing pure A2UI validator moved to `frontend/src/platform/a2ui/protocol.ts`; its former feature path re-exports it. The API lives under `presentations/api/presentations-api.ts` to preserve the actual repository I/O boundary. No dependency, palette, font, console-wide layout or development tester gating changed.

Back keeps the graph draft and returns to a recovery banner; Cancel offers explicit discard. This refines the original Back/discard wording without destroying work. The editor's preview is a sibling of the saving form, not a form descendant. Its controls only modify sandbox data. Explicit uncertainty acknowledgement is required before resubmitting an unconfirmed write; this is not an automatic retry or a claim that the original write failed.

## Independent findings and dispositions

- P1: Enter in a preview TextField could implicitly submit the editor. The preview now sits outside the save form.
- P2: `/items/length` could throw during a preview array write. Array writes now require a canonical existing/next numeric index; unsupported writes announce a read-only binding without assigning array length.
- P2: Back retained a draft then tried focusing the disabled row/New button. Focus restoration now detects disabled buttons and falls back to the registry heading.
- P2: A failed catalog reload hid admission and also removed the exit warning. A runtime-only set of already-admitted draft IDs now protects browser exit independently from display admission. Business records remain in the graph; no record copy or credential is stored in this set.

The reviewer found no additional concrete owner-admission, queued-replay or ordinary save-race defect in these inspected artifacts. This is a bounded source result, not runtime acceptance.

## Verification and unresolved evidence

Tier 0 typecheck and lint pass for the catalog, editor, preview, typed state and navigation integration before the final focus/exit correction; that correction's repeat gate is recorded in execution.md when complete. The standalone boundary scan returns 16 findings, all in untouched Providers/Settings files, and none in the Presentation or shared-protocol edits. Do not suppress or claim the whole boundary gate passed.

Browser coverage must still prove keyboard submission isolation, array editing, focus after recovery/deletion, loading/error transitions, owner re-admission, persisted drafts, uncertain writes, layout at desktop/narrow widths, both themes and visual finish. Impeccable polish/finish review and DESIGN.md remain phase-end work. No raster assets were added.

The uncomfortable limitation remains: template authors edit technical JSON. Clear validation, a starter and local preview do not constitute a no-code builder.

## 2026-09-04 — Session assignment source review

Dual independent Impeccable4.2.0 contract critiques and a fresh adversarial review preceded the session control. The plan records mode/exclusion semantics, owner gates, conditional-write prerequisites and instance-wide warnings. Standalone agent/global panels are still unimplemented.

The session implementation's independent reviewer found: a late admission GET failure could set the main draft status to error during POST; uncertain writes could be submitted again without checking storage; and hidden dirty assignment prevented the promised partial unrelated save. Admission errors now have separately stored generation-guarded state. Uncertainty is classified in the API, retained in the graph, and gates writes/edits until explicit reconciliation. Confirmed POSTs no longer become ambiguous because a subsequent derived read failed. The contract explicitly retains the entire single-save draft when hidden Presentation edits are dirty; it does not promise a partial commit. Unrelated saves without dirty Presentation intent still omit the field and preserve it atomically on the host.

Re-review found those blockers resolved and requested a focusable error summary. The summary now has tabIndex=-1 and receives focus after failed submission. `pnpm typecheck && pnpm lint` exited0 after the correction. Functional focus, interleaved requests, recovery and responsive/visual behavior remain unverified until phase-end browser/tests; this is source evidence only.

## 2026-09-04 — Standalone assignment source review

Agent/global panels now implement the approved assignment contract; the prior unimplemented note above is superseded. Independent review found an awaited agent preflight could dispatch after lost admission, successful non-Selected save erased remembered IDs, global copy overstated exclusion of preserved foreign IDs, and stale reads could leave an indefinite Verifying state. The domain now rechecks owner plus catalog generation immediately before dispatch, retains inactive draft metadata across saves/reloads, and publishes a guarded retry state for invalidated operations. Global copy says every ID outside the selection is excluded across the instance. All findings cleared in the final bounded source recheck.

Strict persisted-agent reads replace the fallback list in this editor. Backend review additionally found and cleared HTTP built-in resolution bypassing persisted restrictions. Failed saves focus the error or the unavailable-state paragraph if its footer was unmounted. Flat2.0 lint corrections use fills/spacing instead of line separators. Final typecheck/lint exited0. None of this establishes visual, browser or runtime acceptance; those checks remain at the whole-phase boundary.
