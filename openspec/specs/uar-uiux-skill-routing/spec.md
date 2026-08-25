# uar-uiux-skill-routing Specification

## Purpose

UI/UX work routing discipline for the `universal-agent-runtime` repo: a project-scoped skill roster at `.kbd-orchestrator/references/uiux-skill-roster.md` plus a fenced-region routing block in `CLAUDE.md` / `AGENTS.md` that any AI tool reads on session start. Defines the seven-step pre-work pattern (memory consult → audit/critique → distill → write). The fenced-region machinery comes from `kbd-agent-rules-injector` (extended with a `--pack` flag).

## Requirements

### Requirement: Skill Roster Cache (this repo)
The UAR repository SHALL ship `.kbd-orchestrator/references/uiux-skill-roster.md` documenting the canonical UI/UX skill roster, organised by tier, with source URLs and fetch dates.

#### Scenario: File exists
- **WHEN** the repo is inspected after this change
- **THEN** `.kbd-orchestrator/references/uiux-skill-roster.md` MUST exist as a non-empty markdown file.

#### Scenario: Tier 1 entries
- **WHEN** the roster file is read
- **THEN** it MUST contain a "Tier 1 — Always consult" section listing at least UI/UX Pro Max (nextlevelbuilder) and Impeccable (pbakaus) with source URLs.

#### Scenario: Tier 2 entries
- **WHEN** the roster file is read
- **THEN** it MUST contain a "Tier 2 — Stack-specific" section listing Vercel React Best Practices, Vercel Composition Patterns, Vercel React Native, Vercel Web Design Guidelines, Anthropic frontend-design, and Anthropic ux-designer.

#### Scenario: Tier 3 entries
- **WHEN** the roster file is read
- **THEN** it MUST contain a "Tier 3 — Pre-work research" section that names `/kbd-memory-recall` plus two web-search targets: "runtime dev tools on a web page" and "Chrome MV3 devtools panel patterns".

#### Scenario: Impeccable command catalogue
- **WHEN** the Impeccable entry is read
- **THEN** it MUST list at minimum these commands by name: `/impeccable audit`, `/impeccable critique`, `/impeccable polish`, `/impeccable distill`, `/impeccable bolder`, `/impeccable quieter`, `/impeccable animate`, `/impeccable colorize`, `/impeccable normalize`, `/impeccable harden`.

### Requirement: Routing Fenced Region
The UAR repository's `CLAUDE.md` and `AGENTS.md` SHALL each contain a `<!-- uiux-routing:start v1 -->` / `<!-- uiux-routing:end -->` fenced region with the documented "UI/UX work routing" block.

#### Scenario: Region present in both files
- **WHEN** either file is read after this change
- **THEN** it MUST contain the documented marker pair surrounding a "UI/UX work routing" section.

#### Scenario: Region content lists ordered steps
- **WHEN** the fenced region is read
- **THEN** it MUST list the seven pre-work steps in order: (1) consult surreal-memory via `/kbd-memory-recall`, (2) UI/UX Pro Max analysis, (3) Impeccable audit/critique + work-specific commands, (4) Anthropic frontend-design + ux-designer, (5) Vercel skills + web search for runtime devtools + Chrome MV3 panel patterns, (6) summarise best practices, (7) write code.

#### Scenario: Region references the roster
- **WHEN** the fenced region is read
- **THEN** it MUST contain an explicit pointer to `.kbd-orchestrator/references/uiux-skill-roster.md`.

#### Scenario: Routing block separate from agent-rules block
- **WHEN** both fenced regions are present in the same file
- **THEN** the `agent-rules` region (from `kbd-agent-rules-injector`) MUST be byte-preserved when the `uiux-routing` region is written, and vice versa.

### Requirement: Injector --pack Flag
The `kbd-inject-agent-rules` skill SHALL accept a `--pack <name>` flag whose values include at minimum `agent-rules` (default) and `uiux-routing`.

#### Scenario: Default pack
- **WHEN** `/kbd-inject-agent-rules` is invoked without `--pack`
- **THEN** the skill MUST behave identically to the pre-change behavior (inject the agent-rules pack); existing scripts and documentation MUST continue to work unchanged.

#### Scenario: uiux-routing pack
- **WHEN** `/kbd-inject-agent-rules --pack uiux-routing` is invoked
- **THEN** the skill MUST locate `references/template-uiux-routing.md` and `references/cache-uiux-routing.md`, MUST use marker pair `<!-- uiux-routing:start v1 -->` / `<!-- uiux-routing:end -->`, and MUST manage that region independently of the agent-rules region.

#### Scenario: Unknown pack value
- **WHEN** `--pack <unknown>` is supplied
- **THEN** the skill MUST exit non-zero with a usage error naming the known pack values.

#### Scenario: Multiple regions co-exist
- **WHEN** both packs have been injected into the same file
- **THEN** running either pack again MUST manage only its own marker pair; the other pack's region MUST remain byte-identical.

### Requirement: Routing Discipline is Authoritative
The fenced region's content SHALL be treated by AI tools as authoritative process guidance for UI/UX work.

#### Scenario: Agent reads the file at session start
- **WHEN** any AI tool (Claude Code, Roo, Cursor, Codex, OpenCode) loads `CLAUDE.md` or `AGENTS.md` for context
- **THEN** the agent has visible, ordered instructions to follow before writing UI/UX code, and the roster file is a one-click-away reference.

### Requirement: First-Customer Adoption
The change SHALL be applied to this repo as part of its implementation, not deferred.

#### Scenario: Roster cache committed
- **WHEN** this change is applied
- **THEN** `.kbd-orchestrator/references/uiux-skill-roster.md` MUST be present at HEAD.

#### Scenario: Fenced regions present at HEAD
- **WHEN** this change is applied
- **THEN** `CLAUDE.md` and `AGENTS.md` MUST both contain the `uiux-routing` fenced region at HEAD.

### Requirement: Repository-owned UI/UX Pro Max skill
The UAR repository SHALL track one canonical UI/UX Pro Max skill payload, its upstream license, and reproducibility metadata. The mandatory UI/UX routing instructions SHALL resolve agents to that tracked payload rather than relying on a machine-local installation.

#### Scenario: Fresh checkout contains the skill
- **WHEN** a developer checks out the repository without a prior local skill installation
- **THEN** the canonical UI/UX Pro Max `SKILL.md`, searchable data, scripts, references, and upstream license are present in the repository

#### Scenario: Machine-local agent state remains excluded
- **WHEN** Git ignore rules are evaluated
- **THEN** unrelated `.agents/` state remains ignored while the canonical UI/UX Pro Max skill subtree is trackable

#### Scenario: Supported tool entry points resolve
- **WHEN** a tracked tool-specific UI/UX Pro Max entry point is inspected
- **THEN** it resolves to the single canonical repository payload without duplicating the skill data

#### Scenario: Reproducibility metadata is present
- **WHEN** the repository skill installation is audited
- **THEN** tracked metadata identifies the upstream source and computed payload hash

#### Scenario: Routing instructions identify the local skill
- **WHEN** an agent follows the UI/UX work routing block and its referenced roster
- **THEN** the roster identifies the canonical local skill path and requires the agent to read its query contract before UI/UX work

#### Scenario: Installed skill is operational
- **WHEN** the skill's integrity validator and a representative stack search run from the canonical payload
- **THEN** validation succeeds and the search returns a stack-appropriate result
