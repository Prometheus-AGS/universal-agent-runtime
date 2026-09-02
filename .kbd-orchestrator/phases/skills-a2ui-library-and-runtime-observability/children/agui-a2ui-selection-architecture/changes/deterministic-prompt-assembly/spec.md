<!-- mirror of openspec/changes/deterministic-prompt-assembly/proposal.md and specs/*/spec.md -->
# deterministic-prompt-assembly

Rank 3 of the codex-harness-comparative-analysis change set. Source: gap G3 and the observability items of G11 in the phase `analysis.md`.

## Why

The system prompt is one `String` built by `push_str` (`src/uar/runtime/manager.rs:1229-1477`): artifact system text, RAG citations, then one overlay per matched skill. Overlay order comes from `HashMap` iteration (`src/uar/runtime/skills/registry.rs:16`, `:209-215`), so two identical turns can produce byte-different prefixes and defeat the Anthropic cache controls applied at `src/llm/anthropic_cache.rs:55-73`. Tools were sorted for exactly this reason (`src/llm/orchestrator.rs:511-516`); prompt sections were not. Nothing marks which text is operator-authored, retrieved, or skill-contributed, so a later injection screen has nothing to key on. `AgentPrompt.instructions` is write-only (`src/uar/domain/artifact.rs:118-122`; `src/uar/compiler/to_artifact.rs:148`). There is no per-turn record of what was assembled.

Codex renders every section as a `ContextualUserFragment` with a role, a stable content kind, and markers (`context-fragments/src/fragment.rs:56-119`) and hashes rendered fragments (`core/src/context/world_state/mod.rs:267-283`). The design is ported; the code depends on Codex protocol types and is not. Codex paths are outside this repository; verbatim excerpts for the cited lines are in the phase `analysis.md` appendix "verified Codex excerpts".

## What changes

- `PromptFragment { id, source, authority, role, retention, content_hash, content }` with `authority ∈ {System, Policy, Host, Skill, Retrieved, User}` and start and end markers per fragment.
- Fixed section order: agent identity, enforced policy summary, host and project instructions, skill catalog, active skill bodies, world-state changes, memory and RAG, then history and the current input. Within a section, deterministic sort by fragment id. Retrieved and skill fragments carry markers and `Retrieved` or `Skill` authority.
- A redacted `TurnManifest` (fragment ids, hashes, counts, budgets, provenance, warnings, never bodies) stored in `Run.context` and emitted as an additive `turn_manifest` artifact.
- `AgentPrompt.instructions` renders as a `Host` fragment or is removed; the compiler stops writing an empty vector.
- The prompt-dialect predicates `prefers_xml_envelope` and `markdown_averse` (`src/llm/prompt_dialect.rs:64`, `:71`) become rendering inputs so dialect affects fragment rendering, not only request parameters.

## Scope

- `src/uar/runtime/manager.rs` (`:1229-1477`)
- `src/uar/domain/artifact.rs`, `src/uar/compiler/to_artifact.rs`, `src/uar/defaults.rs`
- `src/uar/runtime/skills/registry.rs` (`list` ordering)
- `src/llm/prompt_dialect.rs`
- new: `src/uar/runtime/prompt/{fragment.rs,assemble.rs,manifest.rs}`
- tests: `tests/prompt_assembly.rs`

Out of scope: the contributor registry and step snapshots (typed-turn-assembly), skill catalog content (progressive-skill-runtime), world-state discovery (project-instructions-world-state). This change defines the fragment type those changes fill.

## Dependencies

None. Creates the second seam typed-turn-assembly needs: `assemble(inputs) -> (Vec<PromptFragment>, TurnManifest)` as a pure function.

## Verification

Tier 0 per edit; Tier 1 the new tests including a byte-equality test on two identical inputs; Tier 2 at the boundary.

## The uncomfortable thing

Section order becomes a contract. Any operator who relied on skill overlays appearing before RAG citations, or on a particular overlay order, loses that. The order is chosen for cache stability (most static first), and the manifest makes the actual order visible per run.


## Spec delta: prompt-assembly

## ADDED Requirements

### Requirement: Prompt sections are typed fragments in a fixed order
The runtime SHALL assemble the model prompt from typed fragments, each carrying an id, source, authority, role, retention, and content hash, rendered in a fixed section order: agent identity, enforced policy summary, host and project instructions, skill catalog, active skill bodies, world-state changes, memory and retrieval, then conversation history and the current input.

#### Scenario: Identical inputs render identically
- **WHEN** two turns are assembled from the same artifact, the same eligible skills in any discovery order, and the same retrieval hits
- **THEN** the rendered system prompt is byte-identical and every fragment hash matches

#### Scenario: Within-section ordering is deterministic
- **WHEN** several fragments belong to the same section
- **THEN** they are ordered by fragment id, not by discovery order

### Requirement: Fragment authority is explicit
Every fragment SHALL carry an authority of `System`, `Policy`, `Host`, `Skill`, `Retrieved`, or `User`, and retrieved documents, attachments, and skill bodies SHALL be rendered inside markers that identify them as data rather than instructions.

#### Scenario: Retrieved chunk cannot masquerade as policy
- **WHEN** a retrieved chunk contains text formatted like a system instruction
- **THEN** it is rendered inside `Retrieved` markers with `Retrieved` authority and the policy fragment is unchanged

### Requirement: Every run records a redacted turn manifest
The runtime SHALL record, for each turn, a manifest of fragment ids, hashes, counts, budgets, provenance, selected skills and tools, and warnings, SHALL store it in the run context, SHALL emit it as an additive artifact, and SHALL NOT include prompt bodies, credentials, hidden reasoning, or raw retrieved content in it.

#### Scenario: Manifest is emitted alongside the effective policy artifact
- **WHEN** a run starts
- **THEN** clients receive the existing effective-policy artifact and a `turn_manifest` artifact, and the manifest carries no fragment body text

### Requirement: Artifact instructions are rendered
Instructions declared on an agent artifact SHALL be rendered as a host-authority fragment; the runtime SHALL NOT accept an instructions field it never reads.

#### Scenario: Artifact with instructions
- **WHEN** an artifact declares one or more instructions
- **THEN** they appear in the host-and-project-instructions section of the rendered prompt
