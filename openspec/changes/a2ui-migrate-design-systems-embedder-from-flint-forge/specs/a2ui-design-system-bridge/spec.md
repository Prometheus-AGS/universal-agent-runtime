# A2UI design-system bridge

## Purpose

Give UAR a design-system import pipeline (parse `DESIGN.md` or raw
W3C token JSON, store it as a `DesignSystem`), a per-design-system
component override store, and a component-catalog embedding pipeline
— ported from `flint-forge`'s `fdb-app`/`fdb-gateway` A2UI
component-registry use cases, adapted to UAR's backend-agnostic
persistence conventions (SurrealDB default, Postgres optional).

## ADDED Requirements

### Requirement: `DESIGN.md` parses into structured tokens and component overrides
The system SHALL parse a 9-section `DESIGN.md` document (Color,
Typography, Spacing, Layout, Components, Motion, Voice, Brand,
Anti-patterns) into a `DesignMd` value carrying: a `name` from the H1
heading, a `tokens` JSON object merging §1–§4 (JSON-fenced or
`key: value` per section), a list of `ComponentOverride` entries from
§5 (one per `### <slug>` sub-heading, with up to two fenced JSON
blocks for `prop_defaults` and `css_vars`, plus optional
`react_component:` / `flutter_widget:` / `htmx_template:` directive
lines), a `motion` object from §6, and raw text for §7–§9.

#### Scenario: A well-formed DESIGN.md parses successfully
- **WHEN** `design_md_parser::parse` is called with a document
  containing an H1 title and all 9 numbered `##` sections
- **THEN** it returns `Ok(DesignMd)` with `name` set from the H1,
  `tokens.color` / `tokens.typography` / `tokens.spacing` /
  `tokens.layout` populated from §1–§4, `component_overrides`
  populated from §5, and `voice` / `brand` / `anti_patterns`
  populated from §7–§9

#### Scenario: A document missing the H1 title is rejected
- **WHEN** `design_md_parser::parse` is called with a document that
  has no `# ` heading
- **THEN** it returns `Err(ParseError::MissingTitle)`

#### Scenario: Malformed JSON in a token section is rejected
- **WHEN** a fenced ` ```json ` block in §1, §2, or §3 contains
  invalid JSON
- **THEN** `parse` returns `Err(ParseError::InvalidJson { section, .. })`
  identifying the offending section

### Requirement: W3C Design Tokens JSON maps into the same token shape
The system SHALL convert a W3C Design Tokens Community Group 2024
JSON document into the same flattened two-level `{ group: { name:
value } }` token shape produced by the `DESIGN.md` parser, so both
import paths feed the same `DesignSystem.tokens` representation.

#### Scenario: Nested W3C token groups flatten with a hyphenated prefix
- **WHEN** `design_md_parser::map_w3c_tokens` is called with a
  document containing a nested group (e.g.
  `color.brand.dark.$value`)
- **THEN** the result contains `color["brand-dark"]` set to that
  value, and top-level `$schema`/`$description` keys are dropped

### Requirement: A design system persists with import provenance
The system SHALL persist a `DesignSystem` record carrying `id`,
`name`, `tokens`, `source_format` (`design_md` | `w3c_tokens` |
`figma_tokens` | `manual`), `source_content` (the raw imported text,
if any), and `imported_at`, via a backend-agnostic
`DesignSystemStore` trait with in-memory, SurrealDB, and Postgres
implementations.

#### Scenario: Importing a DESIGN.md document creates a design system
- **WHEN** `import::import_design_md` is called with a valid
  `DESIGN.md` string against any `DesignSystemStore` implementation
- **THEN** a new `DesignSystem` is persisted with `source_format =
  design_md`, `source_content` equal to the input, and `tokens`
  matching the parsed document
- **AND** the persisted design system is retrievable via
  `DesignSystemStore::get_design_system` by its generated id

#### Scenario: Importing raw W3C token JSON creates a design system
- **WHEN** `import::import_w3c_tokens` is called with a name and a
  valid W3C tokens JSON string
- **THEN** a new `DesignSystem` is persisted with `source_format =
  w3c_tokens` and `tokens` equal to the flattened conversion

### Requirement: Component overrides resolve against a slug-matched base catalog
Each `ComponentOverride` parsed from a `DESIGN.md` §5 block SHALL be
applied to the base `Component` in the store whose `slug` matches the
override's slug, persisted as a `ComponentOverrideRecord` scoped to
the importing design system's id. Overrides whose slug has no
matching base component SHALL be skipped (not treated as an import
failure) and reported to the caller.

#### Scenario: A component override matches a known base component
- **WHEN** a `DESIGN.md` §5 block's slug matches an existing
  `Component.slug` in the store
- **THEN** `import_design_md` persists a `ComponentOverrideRecord`
  referencing that component's id and the new design system's id
- **AND** the returned `ImportReport.applied_overrides` count
  includes it

#### Scenario: A component override matches no known base component
- **WHEN** a `DESIGN.md` §5 block's slug has no matching
  `Component.slug` in the store
- **THEN** `import_design_md` does not error, does not persist a
  `ComponentOverrideRecord` for that slug, and lists the slug in
  `ImportReport.skipped_slugs`

### Requirement: Overrides merge with base components deterministically
Given a base `Component` and an optional `ComponentOverrideRecord`,
the system SHALL produce a `ResolvedComponent` whose `prop_defaults`
and `css_vars` come from the override when present (or an empty JSON
object when absent), and whose `react_component` /
`flutter_widget` / `htmx_template` come from the override when
present (or `None`, meaning "use the SDK default") — replicating the
`LEFT JOIN` + `COALESCE` semantics of flint-forge's
`flint_a2ui.resolve_components_with_overrides()` SQL function.

#### Scenario: Resolving a component with no override
- **WHEN** `types::resolve_component` is called with a base
  `Component` and `None` for the override
- **THEN** the resulting `ResolvedComponent.prop_defaults` and
  `.css_vars` are empty JSON objects and the renderer-override fields
  are `None`

#### Scenario: Resolving a component with an override
- **WHEN** `types::resolve_component` is called with a base
  `Component` and `Some(&ComponentOverrideRecord)`
- **THEN** the resulting `ResolvedComponent.prop_defaults` and
  `.css_vars` equal the override's values, and any
  renderer-override field set on the override (e.g.
  `react_component`) is carried through

### Requirement: Every store backend supports the same CRUD contract
`DesignSystemStore` SHALL have at least three implementations —
`InMemoryDesignSystemStore` (always compiled), `SurrealDesignSystemStore`
(compiled under the `surreal-backend` feature, UAR's default and
`server-full` release profile), and `PostgresDesignSystemStore`
(compiled under the `sqlx`/`postgres-backend` feature) — and all
SHALL satisfy the same put/get/list contract for design systems,
components, and component overrides.

#### Scenario: A design system round-trips through the in-memory store
- **WHEN** a `DesignSystem` is written via
  `InMemoryDesignSystemStore::put_design_system` and then read via
  `get_design_system` with the same id
- **THEN** the returned record is equal to what was written

#### Scenario: A design system round-trips through the SurrealDB store
- **WHEN** a `DesignSystem` is written via
  `SurrealDesignSystemStore::put_design_system` against an embedded
  SurrealKV instance and then read via `get_design_system`
- **THEN** the returned record's fields match what was written,
  including nested `tokens` JSON

### Requirement: Components missing an embedding can be discovered and backfilled
The system SHALL track, per store backend, which components do not
yet have a recorded embedding (`list_components_missing_embedding`),
and SHALL provide a `backfill_missing` operation that embeds every
such component using a configured `EmbeddingBackend`, continuing past
individual failures (logged, not propagated) so one bad component
does not abort the sweep.

#### Scenario: A newly created component is missing an embedding
- **WHEN** a `Component` is persisted and no embedding has been set
  for it
- **THEN** it appears in `list_components_missing_embedding()`

#### Scenario: Backfilling embeds every missing component
- **WHEN** `embedder::backfill_missing` is called against a store
  with two components missing embeddings and a working
  `EmbeddingBackend`
- **THEN** both components receive embeddings via
  `set_component_embedding`, the returned count is `2`, and
  `list_components_missing_embedding()` afterward is empty

#### Scenario: A single component's embedding failure does not abort the backfill
- **WHEN** `backfill_missing` processes multiple components and one
  component's embed call fails (e.g. the backend is unavailable for
  that text)
- **THEN** the failure is logged and the sweep continues to the next
  component rather than returning early

### Requirement: Embedding generation falls back to a secondary backend on failure
`embedder::embed_component` SHALL accept an optional fallback
`EmbeddingBackend`. If the primary backend's `embed_one` call fails,
the fallback (when provided) SHALL be tried before the call is
reported as failed — preserving flint-forge's
`text-embedding-3-large` → `text-embedding-3-small` fallback
behavior in backend-agnostic form.

#### Scenario: Primary backend succeeds
- **WHEN** `embed_component` is called with a primary backend that
  returns a vector
- **THEN** the component's embedding is recorded using the primary
  backend's `backend_name()` as the stored `model`

#### Scenario: Primary backend fails and a fallback is configured
- **WHEN** the primary backend's `embed_one` call errors and a
  fallback backend is provided
- **THEN** `embed_component` retries with the fallback and, on
  success, records the embedding using the fallback's
  `backend_name()` as the stored `model`

#### Scenario: Primary backend fails and no fallback is configured
- **WHEN** the primary backend's `embed_one` call errors and no
  fallback is provided
- **THEN** `embed_component` returns `Err(EmbedError::Backend(..))`
  and no embedding is recorded
