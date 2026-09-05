# Presentation workspace: implementation design

Scope: extend the existing production admin console with reusable UI template management, then scoped assignment and run-selection provenance. Visitor mode: Operate. The user approved reusable templates, autonomous code-first implementation and phase-end tests. No console redesign, no marketing surface, no promotion of the A2UI tester.

## Direction contract

THESIS: Manage what agents may present; distinguish template availability from an actual rendered result.

OWN-WORLD: Inherit KnowMe Flat 2.0 surface layers, ember primary actions, semantic light/dark tokens, Geist UI text and existing monospaced code styling. No new palette, fonts, borders or glass.

STORY: Scan templates, edit one, inspect a safe preview, save a revision, then assign eligibility separately.

FIRST VIEWPORT: Existing admin shell, title/New action and searchable registry. Opening a row reveals editor/preview columns and Back; narrow screens stack. Save/Cancel show draft status. Preview explicitly says actions do not run.

FORM: Existing registry/detail-pane extension; no concept seed. Signature interaction: validated preview beside source. Motion confirms state and respects reduced motion.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

## Task-specific skill distillation

Impeccable 4.2.0 new-work/Operate/audit/critique preserve the coherent incumbent world and evaluate task clarity before ornament. frontend-design contributes intentional hierarchy and complete states; UI/UX Pro Max contributes visible focus, 44px touch targets, labeled controls, accessible error announcements, preserved drafts and recovery actions. Its two design-system searches returned off-target landing patterns; neither palette/font/glass recommendation overrides the established console. Vercel React Best Practices and Composition Patterns require narrow field subscriptions, independent controls, explicit composition and no synchronous render setters. Entity Graph CRUD supplies normalized IDs, shared normalization and mutation boundaries, but this repository's stronger rule overrides its generic local edit-buffer suggestion: Presentation and PresentationDraft are graph-owned, components import only public platform domain hooks, and domain actions/registered transports own I/O and atomic ingestion. No unrelated legacy component state is refactored.

Consulted references: project Impeccable SKILL.md and reference/new-work.md, init.md, operate.md, audit.md, critique.md; project frontend-design; project UI/UX Pro Max searches (form recovery, React state and two design-system queries); installed Vercel React Best Practices and Composition Patterns; project entity-graph-crud and its layering/prompts/catalog references. UX-designer is not available in the installed skill inventory; the isolated design critic supplies the independent UX review perspective, not a claimed execution of that absent skill. Memory recall wrote prior-context.md with an unavailable-endpoint marker.

## Entity specification

- `Presentation`: id, owner_id, revision, content {title, description, enabled, template {version, catalog_id, components, default_data}}, created_at, updated_at. Full list rows; normalize through one id/data function. No secret fields. Owner and revision are read-only host values.
- `PresentationDraft`: editor ID, record ID or null, expected revision, editable title/description/enabled/template source, validation result, dirty state, save status and error. It is local to an editor but graph-owned; it has no remote transport. Never overwrite a dirty draft from a background catalog refresh.
- List membership holds IDs only. Stable list key includes verified auth generation/owner partition so sign-out or owner changes cannot reuse a previous principal's rows. Follow existing auth reset/bootstrap mechanisms after inspecting them.
- Source inspection found no reusable principal-reset mechanism: the existing graph persists under `uar:entity-graph`, and authenticated session settings use the build-time `VITE_UAR_API_KEY`. The new transport uses that configured credential (Bearer for JWT, x-api-key otherwise), never stores it in entities or keys, and does not decode it as verified authority. GET catalog returns a host-derived owner key with full records, including an empty catalog. A fresh, non-persisted admission token gates all public Presentation hooks until that response verifies the current owner; hydrated catalog/draft rows alone cannot grant display. Records and drafts carry owner_id, list keys include the verified owner, and hooks check both owner and admission before exposing data. Owner change/401 invalidates admission and selection; request generations discard late loads and mutation responses from the prior admission. No new sign-in UI or global auth refactor is included.
- API: GET/POST `/api/uar/presentations`; GET/PUT/DELETE `/api/uar/presentations/{id}`. POST sends draft content; PUT sends `{expected_revision, content}`; DELETE requires expected_revision query. Successful mutations return complete records (delete 204). Errors are `{error: string}` with 401/404/409/422/500 as appropriate. Do not automatically retry uncertain writes.
- No entity relations for catalog CRUD. Later policy fields reference Presentation IDs; disabled/missing choices remain visible as unavailable rather than silently removed from requested policy.
- POST/PUT/DELETE execute once through domain-owned API calls, outside the graph's pending-action/replay queue. Do not use queued graph mutation actions or automatic write retries. Atomically ingest only confirmed results. This matches the existing session-configuration separation and prevents the durable graph's global replay policy from duplicating a create or retrying an uncertain write.

## File map and composition

- `frontend/src/platform/entities/presentations/contracts.ts`: wire/entity/draft types.
- `frontend/src/platform/entities/presentations/api.ts`: HTTP and error normalization, using the existing authenticated API client.
- `frontend/src/platform/entities/presentations/registration.ts`: register schema and transport once through existing bootstrap.
- `frontend/src/platform/entities/presentations/domain.ts`: atomic list ingestion, drafts, save/delete sequencing and conflict handling.
- `frontend/src/platform/entities/presentations/use-presentations.ts`: public narrow list, row-field, draft-field, status and action hooks. No feature-level fetches.
- `frontend/src/platform/entities/presentations/index.ts`, platform index and entity bootstrap: public exports/registration.
- `frontend/src/features/presentations/ui/presentations-page.tsx`: shell/registry composition; child controls own field subscriptions. UI-local search text, selected ID and dialog-open state may be local; records and drafts may not.
- `frontend/src/features/presentations/ui/presentation-editor.tsx`: independently subscribing title/description/enabled/source controls, save status/actions and preview boundary. Reuse existing controls, not raw entity-library UI imports.
- `frontend/src/features/presentations/index.ts`, `frontend/src/pages/admin-page.tsx`, `frontend/src/app/shell/nav-destinations.ts`: production route/destination wiring. Preserve development gating on A2UI testing.

## Interaction and state contract

Registry: title search, clear search, loading skeleton, error with explicit Reload, empty state explaining templates and a New Presentation button. No invented live counters. Rows use real buttons/links with visible text; status includes text, not color alone. Disable from the editor, not an accidental one-tap destructive row action.

Editor: title, description, Available for future runs switch, declarative JSON source with a small valid starter template and inline guidance for data paths, safe preview and save/cancel. A collapsible supported-component reference names the actual nine component kinds, child/children relationships, required root and a data binding example; unsupported-component errors direct users to it. Do not require users to enter IDs, owners, revisions or profile internals for the starter. Validate syntax and supported shape before preview; host validation remains authoritative on save. Invalid content leaves the draft intact and replaces the current preview with a clear invalid-preview message, never stale content labeled current. Preview actions and data edits are sandboxed local draft-preview mechanics, never server calls or agent actions. Lock editable fields and disable close/navigation controls during an in-flight save so completing an older request cannot discard newer edits; never reset from a stale completion.

Recovery: field-associated errors plus a focusable summary after failed submission; polite success status, assertive error announcement; preserve unsaved text after 409/500. Conflict offers Reload saved version with explicit discard confirmation, not automatic overwrite. Delete confirmation names the selected template and explains that already-admitted runs retain their snapshots; cancel preserves the record. On uncertain network failure, explain reloading before retrying rather than presenting an unproven failure as definitely not saved.

Navigation: Back/Cancel with unsaved-discard confirmation; retain draft while confirmation is open. On narrow screens, editor then preview in normal document order; header/actions wrap and rows expose a single 44px-minimum opening target rather than copying Skills' compact multi-action footprint. No hidden primary action behind a fixed overlay. Focus returns to the initiating row/New action on close, or the registry heading after deletion. JSON input uses monospace but labels/body remain normal UI text. Empty library and zero search matches have distinct messages and actions.

Shell navigation must not silently destroy a graph draft; remount in the same verified owner restores the editor draft. Browser exit warns while dirty; persisted hydration remains hidden until fresh owner admission, after which recovery is offered rather than automatically overwriting saved content. During saving, submitted content remains recorded even if navigation/unmount occurs, and stale completions cannot change another editor.

## Evidence boundary

Preimplementation review is a source/design-contract review, not browser acceptance. Independent A and B critiques inspect the incumbent registry and this extension contract; B runs the detector on actual incumbent markup. Functional browser checks/captures wait until the phase-end test boundary requested by the user. No live-render score or contrast claim is inferred from source alone. A fresh adversarial review of the resulting design must precede UI code. The final implementation receives its own bounded desktop/narrow-screen inspection, detector pass and independent finish review. No raster assets are planned.

The uncomfortable trade-off: a declarative JSON editor is still a technical interface. A validated starter, preview, clear recovery and preserved drafts make the bounded first version usable, but do not make it a no-code builder. Do not hide this limitation behind a polished empty state.
