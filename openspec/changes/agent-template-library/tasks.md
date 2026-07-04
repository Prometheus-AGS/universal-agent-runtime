## 1. Templates

- [x] 1.1 `templates/coding.agent.md` — tools+reasoning-required, sliding
      window, A2A/AGUI/OpenAI, `agui_spec`
- [x] 1.2 `templates/vision.agent.md` — vision-required, `Auto` context
      strategy, OpenAI+REST
- [x] 1.3 `templates/terminal.agent.md` — tools+structured-output-required,
      small sliding window, A2A+REST, `dual`
- [x] 1.4 `templates/data.agent.md` — RAG fully enabled (decomposition,
      verification, audit, knowledge_base_ids), `Hierarchical` context
      strategy
- [x] 1.5 Each is a complete v1.1 document (all 15 required sections) —
      not a stub

## 2. `compile` CLI subcommand

- [x] 2.1 `Command::Compile { path, out }` added to `src/config.rs`
- [x] 2.2 `src/uar/compiler/cli.rs`: `run_compile()` — read, parse,
      compile via the existing `pipeline::compile()`, write signed JSON to
      `--out` or stdout
- [x] 2.3 Failing-diagnostic and parse/read-error paths return exit code 1
      with a clear stderr message (not a panic)
- [x] 2.4 Registered in `src/uar/compiler/mod.rs` (`pub mod cli;` +
      re-export) and wired into `src/main.rs`'s command dispatch
- [x] 2.5 3 unit tests: success-to-file, missing-file, unparsable-document

## 3. CI: compile+sign as a release artifact

- [x] 3.1 New `compile-agent-templates` job in
      `.github/workflows/release.yml`
- [x] 3.2 Builds the binary, compiles+signs all 4 templates, tars the
      output as `agent-templates.tar.gz`
- [x] 3.3 Uploaded via `actions/upload-artifact@v4` (matches the existing
      per-platform binary artifact pattern)
- [x] 3.4 Added to `update-release`'s `needs:` list so the tarball is
      attached to the GitHub Release alongside platform binaries
- [x] 3.5 YAML syntax-validated (`python3 -c "import yaml; ..."`) — no
      runner available in this environment to execute the workflow itself

## 4. Regression coverage independent of release time

- [x] 4.1 `tests/agent_templates_test.rs`: compiles all 4 templates on
      every `cargo test`, asserts clean compile + `/v2` schema bump
- [x] 4.2 Manually verified each template compiles standalone via the new
      CLI subcommand

## 5. Verify

- [x] 5.1 `cargo test --test agent_templates_test` 1/1 green
- [x] 5.2 `cargo test --lib compiler::cli::` 3/3 green
- [x] 5.3 `cargo test --lib` full suite 363/363 green
- [x] 5.4 `cargo test --test integration` 56/56 green (2 pre-existing
      `#[ignore]`d)
- [x] 5.5 `cargo clippy --lib --bin universal-agent-runtime` zero new
      warnings from this change's files

## 6. Not this change (disclosed, out of scope)

- [ ] Restructuring `main()` so config-light subcommands (`compile`,
      `eval`) skip full `AppConfig` loading — shared dispatch behavior,
      changes all subcommands, deferred as a follow-up
- [ ] Executing the release workflow end-to-end (requires a tag push /
      GitHub Actions runner, not available in this environment)
