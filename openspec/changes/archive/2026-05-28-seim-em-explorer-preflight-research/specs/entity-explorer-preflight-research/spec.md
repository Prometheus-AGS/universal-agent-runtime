# entity-explorer-preflight-research Specification

## Purpose

Capture the UI/UX routing discipline's mandatory seven-step pre-work for the Entity Explorer
devtools surface as a committed documentation artefact. The deliverable is
`docs/devtools-design-notes.md` in `prometheus-entity-management`. It gates W5
(`seim-em-explorer-event-bus-registry`) and W6 (`seim-em-explorer-panel-components`): neither
change may begin until this file exists on `feat/seim-entity-management-impl`. No production
TypeScript/JSX/CSS files are created or modified by this change.

## Requirements

### Requirement: File Existence

`docs/devtools-design-notes.md` SHALL exist in the `prometheus-entity-management` worktree on
`feat/seim-entity-management-impl` after this change is applied.

#### Scenario: File is present and non-empty
- **WHEN** `feat/seim-entity-management-impl` is checked out
- **THEN** `docs/devtools-design-notes.md` MUST exist and contain all eleven sections defined
  in the proposal (§1–§11).

---

### Requirement: Memory Recall Digest (§1)

The file SHALL contain a section documenting prior UI/UX decisions from `prior-context.md`
that are relevant to a devtools panel surface in this project.

#### Scenario: §1 present
- **WHEN** `docs/devtools-design-notes.md` is opened
- **THEN** a section titled "Memory recall digest" (or equivalent) MUST be present with at
  least one bullet summarising a prior decision or a note that no relevant prior decisions exist.

---

### Requirement: UI/UX Pro Max Summary (§2)

The file SHALL contain a section documenting palette, font pairing, spacing scale, and
accessibility recommendations for a dark devtools panel surface sourced from the
`nextlevelbuilder/ui-ux-pro-max-skill` analysis.

#### Scenario: §2 present with palette recommendation
- **WHEN** §2 is read
- **THEN** it MUST name at least one colour palette recommendation suitable for a dark
  devtools panel, and include an a11y observation (contrast, focus state, or touch target).

---

### Requirement: Impeccable Audit + Critique + Distill Findings (§3)

The file SHALL contain a section recording the output of `/impeccable audit`,
`/impeccable critique`, and `/impeccable distill` applied to the Entity Explorer panel
concept.

#### Scenario: §3 covers all three commands
- **WHEN** §3 is read
- **THEN** it MUST contain findings from each of the three Impeccable commands: audit (a11y /
  performance / responsive), critique (UX review), and distill (complexity-reduction pass
  appropriate for a devtools panel).

---

### Requirement: Anthropic Skill Notes (§4)

The file SHALL contain a section recording insights from the Anthropic `frontend-design` skill
(intentional design) and `ux-designer` skill (UX-engineer review perspective).

#### Scenario: §4 present with attribution
- **WHEN** §4 is read
- **THEN** it MUST attribute findings to both `frontend-design` and `ux-designer` skill sources
  and contain at least one actionable recommendation each.

---

### Requirement: Vercel React Composition Notes (§5)

The file SHALL contain a section summarising Vercel React Best Practices + Composition Patterns
recommendations relevant to the Entity Explorer panel component tree.

#### Scenario: §5 covers component-boundary guidance
- **WHEN** §5 is read
- **THEN** it MUST address at least one of: server vs. client component boundary, performance
  default, or tab-panel composition pattern.

---

### Requirement: Web-Search Citations — Runtime Devtools Best Practices (§6)

The file SHALL contain a section with web-search results for
`"runtime devtools page best practices 2026"`, following the cache discipline
(URL + anchor keyword(s) + fetch date per source).

#### Scenario: §6 cites at least one source
- **WHEN** §6 is read
- **THEN** it MUST include at least one cited source entry in the format:
  `- URL | anchor: <keywords> | fetched: YYYY-MM-DD`.

---

### Requirement: Web-Search Citations — Chrome MV3 Devtools Panel Patterns (§7)

The file SHALL contain a section with web-search results for
`"Chrome MV3 devtools panel patterns"` and/or `"react-devtools bridge architecture"`,
following the same citation discipline.

#### Scenario: §7 cites at least one source
- **WHEN** §7 is read
- **THEN** it MUST include at least one cited source entry (URL + anchor + fetch date).

---

### Requirement: Distilled One-Paragraph Summary (§8)

The file SHALL contain a single paragraph of 3–5 sentences distilling the most relevant
best-practice findings for the Entity Explorer surface. This paragraph is the prompt context
passed to the W6 implementation step.

#### Scenario: §8 length constraint
- **WHEN** §8 is read
- **THEN** the distillation MUST be a single paragraph containing between 3 and 5 sentences.
  It MUST NOT be a bullet list. It MUST reference at least two of the sources consulted in §2–§7.

---

### Requirement: Downstream Implications (§9–§11)

The file SHALL conclude with explicit implications sections for W5, W6, and W8 (stretch).

#### Scenario: §9 — W5 event bus implications
- **WHEN** §9 is read
- **THEN** it MUST contain at least one design hint for the event-bus / multi-store registry
  architecture (W5), derived from the preflight findings.

#### Scenario: §10 — W6 panel component implications
- **WHEN** §10 is read
- **THEN** it MUST contain at least one design hint for the 5-tab FAB panel component tree
  (W6), derived from the preflight findings.

#### Scenario: §11 — W8 extension scaffold implications (bonus)
- **WHEN** §11 is read
- **THEN** it MUST contain at least one note about Chrome MV3 devtools panel architecture
  relevant to the extension scaffold (W8).

---

### Requirement: No Production Code Introduced

This change SHALL NOT introduce or modify any TypeScript, TSX, CSS, or JSON build-configuration
file in `prometheus-entity-management`.

#### Scenario: Only docs/ is touched
- **WHEN** `git diff --name-only HEAD~1 HEAD` is inspected for this change's commit
- **THEN** the only file path MUST match `docs/devtools-design-notes.md`; no `src/`, `test/`,
  or config-file paths MAY appear.
