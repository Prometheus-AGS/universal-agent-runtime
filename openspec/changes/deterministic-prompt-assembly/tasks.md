# Tasks — deterministic-prompt-assembly

scope: src/uar/runtime/manager.rs (1229-1477), src/uar/domain/artifact.rs, src/uar/compiler/to_artifact.rs, src/uar/defaults.rs, src/uar/runtime/skills/registry.rs, src/llm/prompt_dialect.rs, src/uar/runtime/prompt/**, tests/prompt_assembly.rs

## 1. Failing tests first

- [ ] 1.1 `tests/prompt_assembly.rs`: assembling the same artifact, three matched skills inserted in different registry orders, and the same RAG hits twice yields byte-identical system prompts and identical manifest hashes
- [ ] 1.2 A retrieved chunk renders inside `Retrieved` markers and its fragment authority is `Retrieved`; a skill body renders inside `Skill` markers
- [ ] 1.3 The manifest contains fragment ids, hashes, counts, and budgets and contains no fragment body text, credential, or retrieved content
- [ ] 1.4 The manifest is stored in `Run.context` and emitted as a `turn_manifest` artifact; the existing `effective_run_policy` artifact is still emitted
- [ ] 1.5 An artifact with non-empty `instructions` renders them as a `Host` fragment after policy and before the skill catalog
- [ ] 1.6 Add `insta` 1.48.0 as a dev-dependency; an `insta` snapshot records the diff between two successive rendered prompts for the same session so a prefix-stability regression is visible as a snapshot change

## 2. Fragment model

- [ ] 2.1 Add `src/uar/runtime/prompt/fragment.rs`: `PromptFragment`, `Authority`, `Retention`, markers, `content_hash` (SHA-256 over role, kind, and normalized text, `\r\n` to `\n`)
- [ ] 2.2 Add `src/uar/runtime/prompt/assemble.rs`: fixed section enum, per-section deterministic sort, `render(&[PromptFragment]) -> String`
- [ ] 2.3 Add `src/uar/runtime/prompt/manifest.rs`: `TurnManifest` with serde, redaction enforced by construction (no body field exists)

## 3. Wire the manager

- [ ] 3.1 Replace the `push_str` sequence with fragment construction: artifact system → `System`, effective policy summary → `Policy`, `instructions` → `Host`, RAG block → `Retrieved`, skill overlays → `Skill`
- [ ] 3.2 Sort skill fragments by skill id before rendering; `SkillRegistry::list` returns a sorted vector
- [ ] 3.3 Emit the manifest artifact and store it in `Run.context`
- [ ] 3.4 Feed `prefers_xml_envelope` and `markdown_averse` into `render`

## 4. Cleanup

- [ ] 4.1 Compiler writes `instructions` from the agent spec or omits the field; delete the hardcoded empty vector
- [ ] 4.2 Delete `src/uar/prompt_cache.rs` if no caller exists after this change (confirm with a call-site grep and record the result in tasks notes)

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test prompt_assembly`
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 `openspec validate deterministic-prompt-assembly --strict`
