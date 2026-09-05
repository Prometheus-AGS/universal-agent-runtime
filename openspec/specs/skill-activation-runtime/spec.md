# skill-activation-runtime Specification

## Purpose

Define budgeted skill discovery, narrow-only activation, retention and usage attribution.

## Requirements

### Requirement: Eligible skills are presented as a budgeted catalog
The runtime SHALL present eligible skills to the model as a catalog of one line per skill within a budget of the smaller of two percent of the model context window and 10,000 tokens, using 8,000 characters when the window is unknown, and SHALL truncate descriptions round-robin before omitting any entry.

#### Scenario: Large catalog under budget
- **WHEN** 2,000 skills are eligible for a run
- **THEN** the rendered catalog is within the budget and every skill id and title is present

#### Scenario: Extreme pressure
- **WHEN** even minimum lines exceed the budget
- **THEN** entries are omitted and the catalog ends with a note stating how many were omitted

### Requirement: Skills are activated explicitly
The runtime SHALL load a skill's full body only through an activation: a client attachment on the run request, a model activation through a model-only tool, or, in `legacy_overlay` mode only, an above-threshold implicit match. The runtime SHALL intersect every activation with the effective eligible set and SHALL return a typed activation failure for an ineligible, disabled, missing, or dependency-invalid skill without widening access.

#### Scenario: Client attachment
- **WHEN** a run request names a skill in `skill_attachments`
- **THEN** the skill body and its declared MCP tools are available before the first model call

#### Scenario: Model activation
- **WHEN** the model calls `activate_skill` with an eligible skill id
- **THEN** the next model step includes the skill body and its tools, and exact usage is recorded

#### Scenario: Ineligible activation
- **WHEN** the model calls `activate_skill` with a disabled or missing skill id
- **THEN** the result is a typed activation failure and no skill content or tool is added

### Requirement: Implicit matching ranks candidates and activates only in legacy mode
The runtime SHALL score implicit skill matches and enforce the configured threshold and margin. In `catalog` mode (the target default) implicit matching SHALL only rank candidates: above-threshold candidates are marked as suggested in the catalog and recorded for telemetry, and no body is loaded without an attachment or a model activation. In `legacy_overlay` mode (the migration default) above-threshold candidates SHALL be activated as a third activation path, subject to the artifact's `max_active` limit and `prefer` order. In both modes the runtime SHALL NOT activate or inject any skill whose score is below threshold.

#### Scenario: Above-threshold match in catalog mode
- **WHEN** the best candidate scores above the threshold and the harness is in `catalog` mode
- **THEN** the catalog marks it as suggested, the candidate is recorded, and no body is loaded

#### Scenario: Above-threshold match in legacy overlay mode
- **WHEN** the best candidate scores above the threshold and the harness is in `legacy_overlay` mode
- **THEN** the skill is activated as an implicit activation, counted against `max_active`, and recorded with invoke type `implicit`

#### Scenario: Below-threshold match
- **WHEN** the best candidate scores below the threshold in either mode
- **THEN** no skill is activated and the candidates are recorded for telemetry only

#### Scenario: max_active reached
- **WHEN** `max_active` skills are already active
- **THEN** a further activation of any kind is refused with a typed result naming the limit

### Requirement: Candidate reduction runs in shadow mode until recall is proven
Any statistical or embedding-based candidate reduction SHALL run in shadow mode, emitting recall telemetry against explicit activations, and SHALL NOT omit a catalog entry until measured Recall@10 is at least 99 percent.

#### Scenario: Shadow reducer disagrees with an explicit activation
- **WHEN** the reducer's top ten omits a skill the client attached
- **THEN** the catalog is unchanged and a recall miss is recorded

### Requirement: Activated skill bodies survive compaction under a budget
Activated skill bodies SHALL be reclaimable fragments that compaction removes first, and the most recent activation of each active skill SHALL be re-attached after compaction within a configurable budget.

#### Scenario: Compaction with an active skill
- **WHEN** compaction runs while a skill is active
- **THEN** the skill body is re-attached after the summary within the re-attachment budget

### Requirement: Skill use is attributed in telemetry
The runtime SHALL record an activation outcome for every activated skill, including prompt-only skills, and SHALL attribute each model request's tokens and cost to every skill active during that request through a per-skill attribution counter, leaving the existing unlabeled token and cost totals unchanged so that attribution never double-counts the totals.

#### Scenario: Prompt-only skill
- **WHEN** a skill with no tools is activated and the run completes
- **THEN** an outcome is recorded for it and the per-skill attribution counter for its id increases by the run's request tokens

#### Scenario: Two skills active
- **WHEN** two skills are active during one model request of 1,000 tokens
- **THEN** each skill's attribution counter increases by 1,000, and the unlabeled token total increases by 1,000 once
