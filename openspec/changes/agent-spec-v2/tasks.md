## 1. IR fields

- [x] 1.1 `ModelRequirementsSection` (mirrors `RouteRequirements`)
- [x] 1.2 `PromptDialectSection` (mirrors `DialectRequest` + optional
      explicit dialect override)
- [x] 1.3 `RagConfigurationSection` (mirrors CH-11's hardening knobs)
- [x] 1.4 `ContextStrategySection` (tagged enum mirroring `ContextStrategy`
      variant-for-variant, including `Auto`)
- [x] 1.5 `ApiHarnessSection` (protocols + `stream_mode`)
- [x] 1.6 Added all 5 to `AgentDescriptorIR` with `#[serde(default)]`

## 2. Backward compatibility

- [x] 2.1 5 new `Option<T>` fields on `PartialAgentDescriptorIR`
- [x] 2.2 `try_into_complete()` uses `.unwrap_or_default()` for the 5 new
      fields (not `?` — doesn't gate readiness)
- [x] 2.3 `From<AgentDescriptorIR> for PartialAgentDescriptorIR` wraps the
      5 new fields in `Some(...)`
- [x] 2.4 `SectionName` gained 5 new variants, deliberately excluded from
      `SectionName::ALL` (completeness gating unaffected)

## 3. Parser wiring

- [x] 3.1 `SectionName::from_heading` recognizes the 5 new headings
- [x] 3.2 `SectionName::display_name` for the 5 new variants
- [x] 3.3 `parser.rs::deserialize_section` match arms for the 5 new
      variants
- [x] 3.4 `completeness.rs::is_section_present` match arms (never reached
      via `ALL`, added for match exhaustiveness)

## 4. Verify

- [x] 4.1 Fixed a pre-existing test in `completeness.rs` that constructed
      a full `PartialAgentDescriptorIR` literal without the new fields
      (`..Default::default()`).
- [x] 4.2 3 new tests in `parser.rs`: v1.1 backward-compat, full v2
      round-trip, heading recognition.
- [x] 4.3 `cargo test --lib compiler::` 30/30 green.
- [x] 4.4 Full suite `cargo test --lib` 333/333 green.

## 5. Follow-ups (not this change)

- [ ] CH-13: compiler stage validation for the 5 new sections + emit-stage
      `schema_version` bump when any v2 section is non-default.
- [ ] CH-14: conformance harness comparing declared `model_requirements`/
      `prompt_dialect`/`context_strategy` against actual runtime behavior.
