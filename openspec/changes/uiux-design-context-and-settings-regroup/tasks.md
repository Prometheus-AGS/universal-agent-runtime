## 1. Design context

- [ ] 1.1 Read-only: run `impeccable context` from the project to learn where PRODUCT.md, DESIGN.md and the sidecar resolve. Record the resolved paths in the change handoff. Do not rerun it.
- [ ] 1.2 Run `impeccable update`, then `impeccable doctor`. If doctor or any command reports that the pinned engine (`IMPECCABLE_BIN`, operator settings) cannot serve the installed skills, stop, record the versions seen, and ask the operator. Never edit `~/.claude/settings.json` or any harness setting.
- [ ] 1.3 Run `impeccable init` to rewrite `PRODUCT.md` (design D2): remove the "does not redesign" statement, keep the confirmed product facts, probe for a real operator answer before inferring, label inferred facts.
- [ ] 1.4 Write `DESIGN.md` and its `.impeccable/design.json` sidecar from `docs/design/` per the source table in design D3, in the OKLCH values of `brand-spec.md`. State the Inter and `--border` tensions. Use the `brand-spec.md` one-sentence definition as the Overview opening; label any named North Star provisional unless the operator supplies one.
- [ ] 1.5 Run `impeccable document` in scan mode as a drift check only. Store the drift list under the phase directory as work items. Confirm `DESIGN.md` is unchanged by the run. Answer no to any overwrite prompt.

## 2. Authority documents

- [ ] 2.1 Update `docs/ui-design-authority.md` (design D5): add `docs/design/` and `DESIGN.md` to the authority order, add the redesign decision with its rationale reproduced, list the retired documents. Leave D1 untouched.
- [ ] 2.2 Add a retirement notice naming `docs/design/` as the first paragraph of `docs/UI_DESIGN.md` and `docs/admin-aesthetic-spec.md`. No other edit.
- [ ] 2.3 Correct the stale Geist sentence in `docs/design/brand-spec.md` (Inter is shipped, `tokens.css:12-14`). Search the repository for documents that still cite the two retired documents as authority and list them in the handoff; fix only those that state authority.

## 3. Settings regroup

- [ ] 3.1 Add `frontend/src/features/settings/model/settings-routes.ts`: the single route table of design D6 (area, section, label, icon, path, namespace key or page, Tauri-only flag). Add `/settings/*` to the router in `frontend/src/App.tsx` with the redirect table (old `/admin/<section>` URLs, unknown area, unknown section, desktop outside Tauri). Depends on `uiux-settings-truthful-status-and-removals` tasks 2.1 and 4.2 (surviving inventory, relinked pages).
- [ ] 3.2 Replace `useState` selection in `settings-page.tsx` with a `SettingsLayout` reading `:area` and `:section`; area navigation works at compact width. Derive breadcrumbs and one command-palette entry per section from the route table (`command-palette.tsx`, `breadcrumb-header.tsx`). Provide a header slot per area; mount the system-default picker from `uiux-system-model-live-default` in the `models` slot when it exists.
- [ ] 3.3 Update `nav-destinations.ts` (design D7): one Settings destination; Runs matches `/admin/runtime`, `/admin/approvals`, `/admin/protocols`; Presentations and the dev A2UI destination stay. Update `mobile-sheet-host.tsx` if it assumes the old items, `pages/admin-page.tsx` (drop the moved sections from the page map), `nav-destinations.test.ts`, `app-shell.test.tsx`, and the e2e specs that assert the old inventory. Mount `app/shell/boot-status-banner.tsx` (created by `uiux-settings-truthful-status-and-removals#4.5`) in the shell above the route content, and add the `governance/config-status` section for the panel from `uiux-settings-truthful-status-and-removals#4.5`-`#4.7`.
- [ ] 3.4 Audit every retained control as primary or advanced; write the audit beside the route table. Add the `x-uar-ui-advanced` schema marker for schema-driven panels and wrap bespoke advanced groups in `AdvancedSection`. A raw JSON editor, where one remains after the picker change, moves inside Advanced. Keep every new file at or below about 600 lines.
- [ ] 3.5 Verify by reading, not running, that the regrouped layout uses existing tokens (no new colour), keeps visible status text, and keeps the high-contrast stroke exception. Source inspection only.

## 4. Critique gate

- [ ] 4.1 Run Impeccable critique twice with isolated subagents over `DESIGN.md`, its sidecar and `PRODUCT.md` as one artifact; store the files under `.impeccable/critique/` with `method: dual-agent`.
- [ ] 4.2 Run the same two-agent critique over the Settings regroup diff.
- [ ] 4.3 Run a fresh-context adversarial review that receives only the artifacts. Resolve P0 findings and re-review; resolve or record each P1 with an operator decision. Record the `cross_model_check` value.

## 5. Boundary acceptance (runs only at the phase boundary)

These describe what the final integration gate exercises. They must not be executed early. Expected values come only from `fixtures/` (design, "Frozen oracle fixtures"); no scenario reads a route table, audit file or critique header written during the phase.

- [ ] 5.1 SCN-DSN-001: cold-load, reload and back/forward on each deep-link sample; every palette entry in the fixture; every redirect; the four fallback controls and the unlisted deeper URL. Three samples.
- [ ] 5.2 SCN-DSN-002: compare the rendered navigation with `GET /api/uar/settings/types` using the fixture's maps; negative controls with a scratch console missing `rag` and one listing `sandbox`.
- [ ] 5.3 SCN-DSN-003: runner-computed visible-control count per section against the fixture limit; open Advanced by keyboard; negative control with a scratch section forced to 9 controls. Three samples.
- [ ] 5.4 SG-DSN-004 and SG-DSN-005 (supplemental): `impeccable context` loads both files; token comparison with the fixture; `document` leaves `DESIGN.md` byte-identical; PRODUCT.md contradiction absent and facts present; negative controls on scratch copies and the pre-phase file.
- [ ] 5.5 SG-DSN-006 (supplemental): read and search the authority documents against the fixture; negative control against the pre-phase tree.
- [ ] 5.6 SCN-DSN-007: the runner counts `[P0]` and `[P1]` markers in the independent reviewers' files and checks each packet manifest; negative controls with a forbidden-path manifest and a header that disagrees with its body. Three independent adversarial reviews per artifact.
- [ ] 5.7 SCN-DSN-008: Settings areas at 320, 768, 1024 and 1440 px in dark, light and high contrast with the accessibility Playwright config; negative control with a forced-width scratch element. Three samples.
