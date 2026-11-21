## ADDED Requirements

### Requirement: Design authority names the redesign sources and retires superseded documents
`docs/ui-design-authority.md` SHALL name `docs/design/` as the visual authority for the operator console and `DESIGN.md` as its derived record, SHALL reproduce the scope and rationale of the 2026-11-19 redesign decision without depending on a working-state path, and SHALL list the documents it retires. `docs/UI_DESIGN.md` and `docs/admin-aesthetic-spec.md` SHALL open with a retirement notice naming `docs/design/`. Retired documents SHALL NOT be deleted.

#### Scenario: A contributor asks which document governs visuals
- **WHEN** a contributor reads `docs/ui-design-authority.md`
- **THEN** it names `docs/design/` and `DESIGN.md` as the visual authority
- **AND** it lists `docs/UI_DESIGN.md` and `docs/admin-aesthetic-spec.md` as retired

#### Scenario: A retired document is opened
- **WHEN** a contributor opens `docs/UI_DESIGN.md` or `docs/admin-aesthetic-spec.md`
- **THEN** its first paragraph says it is retired and points to `docs/design/`

#### Scenario: The decision record is not present
- **WHEN** KBD working-state files are absent from a packaged checkout
- **THEN** the authority document still states the redesign decision's scope and rationale

#### Scenario: The Base UI exception is unaffected
- **WHEN** the authority document is updated for the redesign
- **THEN** decision D1 and its scope text remain unchanged

### Requirement: DESIGN.md derives from docs/design
`DESIGN.md` and its `.impeccable/design.json` sidecar SHALL be written from `docs/design/` and SHALL carry its token values unchanged. Impeccable `document` SHALL be used only to report drift between `DESIGN.md` and the shipped code, and its output SHALL NOT rewrite `DESIGN.md`. Drift SHALL be recorded as work items.

#### Scenario: Tokens are compared
- **WHEN** the colour and radius values in the `DESIGN.md` frontmatter are compared with the `docs/design/brand-spec.md` tables
- **THEN** every value is equal

#### Scenario: A drift check runs
- **WHEN** Impeccable `document` runs in scan mode after `DESIGN.md` exists
- **THEN** differences from the shipped code are recorded as a work list
- **AND** `DESIGN.md` is byte-identical before and after the run

#### Scenario: The code lags the design
- **WHEN** the shipped code differs from `DESIGN.md`
- **THEN** `DESIGN.md` is not edited to match the code

### Requirement: Design context is refreshed without changing operator settings
Refreshing design context (`impeccable update`, `init`, `document`) SHALL NOT edit user or harness settings, including the `IMPECCABLE_BIN` engine pin. When the installed skills and the pinned engine disagree, the work SHALL stop at the first command that needs the other engine and report the mismatch to the operator.

#### Scenario: Skills and engine disagree
- **WHEN** a design-context command reports that the pinned engine cannot serve the installed skills
- **THEN** the work stops and records the observed versions
- **AND** no settings file is modified

#### Scenario: PRODUCT.md is rewritten
- **WHEN** `impeccable init` rewrites `frontend/PRODUCT.md`
- **THEN** the statement that the work does not redesign the application is absent
- **AND** confirmed product facts about Presentations, entity ownership and the trusted host remain
