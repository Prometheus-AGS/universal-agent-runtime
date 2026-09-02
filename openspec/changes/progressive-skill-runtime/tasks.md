# Tasks — progressive-skill-runtime

scope: src/uar/runtime/skills/**, src/uar/runtime/manager.rs (skill block and outcome correlation), src/uar/domain/artifact.rs, src/uar/domain/skills.rs, src/uar/api/routes.rs (run request), src/uar/api/openapi.rs, src/uar/runtime/native_skills/activate_skill.rs, src/uar/telemetry/metrics.rs, tests/skill_activation_runtime.rs, tests/skill_scoped_governance.rs

## 1. Failing tests first

- [ ] 1.1 `tests/skill_activation_runtime.rs`: 2,000 synthetic skills render a catalog within `min(2% window, 10,000 tokens)` with every skill id present; descriptions are truncated round-robin before any entry is omitted
- [ ] 1.2 A run with `skill_attachments: ["s1"]` loads s1's body before the first model call; `skill_attachments: ["disabled"]` returns a typed activation failure and no widening
- [ ] 1.3 The model calls `activate_skill("s2")`; the next step's prompt contains s2's body and s2's MCP tools are in the tool set; `activate_skill("missing")` returns a typed failure result
- [ ] 1.4 With `max_active: 2`, a third activation is refused with a typed result naming the limit
- [ ] 1.5 A keyword match scoring below threshold activates nothing in either mode; the "include anyway" log line no longer exists; an above-threshold match activates in `legacy_overlay` mode and only marks the catalog entry as suggested in `catalog` mode
- [ ] 1.6 After compaction, the most recent activated body is re-attached within the re-attachment budget
- [ ] 1.7 An overlay-only skill activation records an outcome; with s1 and s2 both active during a 1,000-token request, `uar_skill_request_tokens_total{skill="s1"}` and `{skill="s2"}` each increase by 1,000 and `uar_llm_tokens_total` increases by 1,000 once

## 2. Catalog and activation

- [ ] 2.1 Add `src/uar/runtime/skills/catalog.rs`: eligible set → `CatalogEntry` lines, budget resolution, round-robin truncation, omission note
- [ ] 2.2 Add `src/uar/runtime/skills/activation.rs`: `activate(skill_id, ctx) -> Result<ActivatedSkill, ActivationFailure>` validating eligibility, enablement, dependencies, and `max_active`
- [ ] 2.3 Add `activate_skill` native tool with `Exposure::ModelOnly`; wire into descriptor assembly
- [ ] 2.4 Add `skill_attachments` to the run request; intersect with the effective eligible set; load before the first model call

## 3. Matching

- [ ] 3.1 Replace `Vec<Skill>` results with `SkillMatchResult { candidates: Vec<SkillCandidate>, accepted: Vec<SkillId> }`; enforce threshold and margin; delete the below-threshold inclusion branch; add `HarnessConfig.skill_activation_mode` with `legacy_overlay` as the default and route above-threshold matches to activation or catalog marking by mode
- [ ] 3.2 Read `_threshold` in `keyword_match`; enforce `max_active` and apply `prefer` as tie-break
- [ ] 3.3 Shadow reducers emit `uar_skill_shadow_recall` against explicit activations and never alter the catalog

## 4. Retention and telemetry

- [ ] 4.1 Activated bodies are `Skill` fragments with `Retention::Reclaimable`; compaction reclaims them first and re-attaches the latest under the budget
- [ ] 4.2 Record activation outcome for every activated skill including overlay-only; add the `skill` label to token and cost metrics

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test skill_activation_runtime --test skill_scoped_governance`
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 `openspec validate progressive-skill-runtime --strict`
