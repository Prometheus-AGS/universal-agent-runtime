# Typed prompt template resolution and compiler override propagation

Task `harness-model-profiles-routing-2-1` adds a versioned, typed prompt-template contract without introducing a dependency. `PromptTemplateResolver` resolves a host-allowed compatible descriptor/operator override before an exact provider, endpoint, model and model-revision profile; it then considers only an explicitly verified family revision, and finally a generic endpoint contract only when the host marks that destination eligible. It never derives compatibility from a model-name substring. Duplicate matches, forbidden/unknown/incompatible overrides, unsupported destinations, and blank destination/profile identities or revisions return typed errors.

`PromptTemplateProfile` keeps prompt layout separate from driver-owned wire serialization and provider settings. Profiles declare required sections and supported roles. Rendering remains deterministic in fixed section/id order. Structured rendering escapes every fragment body. Plain rendering now gives user input its own host-owned envelope and escapes user, skill and retrieved bodies, so those bodies cannot close or fabricate host-owned marker elements.

The compiler IR preserves an optional data-only template identifier. Stage 08 counts a template-only prompt declaration as schema v2. The integration fixture parses a descriptor whose only non-default v2 prompt field is `template`, compiles it through all eight stages, and verifies schema v2 plus exact payload propagation.

Files changed:

- `src/llm/prompt_dialect.rs`: typed resolver, precedence, policy-governed overrides, identity validation and typed failures.
- `src/uar/runtime/prompt/assemble.rs`: template selectors, layouts, required-slot/role validation, fixed template rendering.
- `src/uar/runtime/prompt/fragment.rs`: escaped structured bodies and escaped host-owned envelopes for host, user, skill and retrieved content.
- `src/uar/runtime/prompt/mod.rs`: public exports for the template contract.
- `src/uar/compiler/ir.rs`: data-only `prompt_dialect.template` field.
- `src/uar/compiler/stages/s08_emit.rs`: template-only v2 schema detection. This was the one necessary execution-map deviation; without it, the new compiler field could survive while the emitted descriptor was mislabeled v1.
- `tests/prompt_assembly.rs`: precedence, explicit family/generic evidence, forbidden override, required slots, roles, adversarial escaping, blank revisions, deterministic rendering and full compiler/emitter propagation.

Verification on final code:

- `RUSTC_WRAPPER= cargo check --locked --no-default-features --features server-full`: exit 0; finished in 3m18s.
- `RUSTC_WRAPPER= cargo test --locked --no-default-features --features server-full --test prompt_assembly`: 12 passed, 0 failed.
- `RUSTC_WRAPPER= cargo test --locked --no-default-features --features server-full --test model_path_resiliency`: 12 passed, 0 failed.
- `cargo fmt --all -- --check`: exit 0.
- targeted `git diff --check`: exit 0.
- `openspec validate harness-model-profiles-routing --strict --json`: 1 passed, 0 failed.
- isolated artifact critic: first pass found three major defects; all three received regressions and fixes; fresh-context final pass returned PASS with no findings.

The initial `prompt_assembly` attempt failed at compile because the fixture compared a complete `Result` whose success type intentionally lacks `PartialEq`. The retained failure led to a test-only `matches!` correction; production code was unchanged by that correction.

No dependency, provider profile, endpoint setting or production dispatch wiring was added. The guards trace to explicit requirements or critic-reproduced failures: override policy and compatibility, required slots/roles, marker injection, and blank version identities. No speculative guard remains. Endpoint-specific serialization/settings/output ceilings belong to task 2.2, and production entry-point wiring belongs to task 2.3. Therefore this unit does not claim live-profile compatibility or final outbound-request certification.
