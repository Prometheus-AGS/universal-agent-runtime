# CH-15 agent-template-library

## Why

CH-12 (agent-spec-v2) added the IR fields; CH-13 (compiler-v2-stages) taught
the compiler to validate and emit them. Neither produced anything an author
could actually start from — `assessment.md` found zero `.agent.md` files
anywhere in the repo. Without worked examples, "declare model_requirements /
prompt_dialect / context_strategy" is an abstract spec, not something an
author can copy and adapt.

## What changed

Four pre-built `.agent.md` templates under `templates/`, each a complete
v1.1 document (all 15 required sections) plus a deliberately distinct
combination of the five v2 sections, so together they demonstrate the
full v2 surface:

- **`coding.agent.md`** — tools-required, reasoning-required,
  128K min context; hard reasoning dialect hint; 60-message sliding
  window; A2A+AGUI+OpenAI protocols, `agui_spec` stream mode.
- **`vision.agent.md`** — vision-required; `Auto` context strategy
  (deliberately, to demonstrate a template that *doesn't* pin one); OpenAI
  + REST protocols only.
- **`terminal.agent.md`** — tools-required + structured-output-required
  (for reliable command/argument extraction); small 30-message window;
  A2A + REST, `dual` stream mode.
- **`data.agent.md`** — RAG fully enabled (decomposition, verification,
  audit, a `primary-kb` knowledge base id); `Hierarchical` context
  strategy (long-running analyst sessions); 200K min context.

New CLI subcommand `universal-agent-runtime compile <path> [--out <path>]`
(`Command::Compile` in `src/config.rs`, implementation in the new
`src/uar/compiler/cli.rs`) compiles+signs a single `.agent.md` document by
calling the existing `pipeline::compile()` — not a reimplementation. Wired
into `.github/workflows/release.yml` as a new `compile-agent-templates`
job: builds the binary, compiles+signs all four templates, tars the
output, and uploads it as a release artifact (`agent-templates.tar.gz`),
alongside the existing per-platform binary artifacts.

New `tests/agent_templates_test.rs`: compiles all four templates on every
`cargo test` (not just at release time), asserting a clean compile and a
`uar-agent-descriptor/v2` schema bump for each.

## Verification

- `cargo test --test agent_templates_test`: 1/1 green (compiles+signs all
  4 templates, asserts v2 schema).
- `cargo test --lib compiler::cli::`: 3/3 green (`compile` subcommand:
  success-to-file, missing-file error, unparsable-document error).
- Manually verified each template compiles standalone via `cargo run
  --bin universal-agent-runtime -- compile templates/<name>.agent.md`
  (all 4 pass, all emit schema `/v2`).
- `cargo test --lib` full suite: 363/363 green.
- `cargo test --test integration` (the broader integration binary):
  56/56 green, 2 pre-existing `#[ignore]`d.
- `cargo clippy --lib --bin universal-agent-runtime`: zero new warnings
  attributable to `cli.rs`, the `config.rs` `Compile` variant, or the
  `main.rs` dispatch arm.
- `.github/workflows/release.yml` YAML validated with `python3 -c "import
  yaml; yaml.safe_load(...)"` (no GitHub Actions runner available in this
  environment to execute the workflow itself — syntax-checked, not
  execution-tested).

## Known limitation (disclosed, not fixed here)

The binary's `main()` always loads the *full* `AppConfig` before
dispatching to any subcommand (`Eval` already had this same constraint)
— so `compile` requires a minimal persistence config
(`UAR_PERSISTENCE__PROVIDER`/`UAR_PERSISTENCE__DATABASE_URL`) even though
compiling a template touches no database. Restructuring `main()` to let
config-light subcommands skip full config loading would help both `eval`
and `compile`, but changes shared dispatch behavior for all subcommands —
out of scope for this change; documented as a follow-up.
