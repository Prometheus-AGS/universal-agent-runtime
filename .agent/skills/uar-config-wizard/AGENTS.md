# Contributor Guide

## Commit Conventions

Use conventional commits:
- `feat:` — New templates, modes, sub-skills, platform support
- `fix:` — Template fixes, validation corrections, broken references
- `docs:` — Reference updates, SKILL.md changes, CLAUDE.md updates
- `refactor:` — Internal restructuring without behavior change

## Branch Strategy

- `main` — Stable, tested
- `feat/*` — Feature branches

## Pull Request Checklist

- [ ] Templates use `{{variable}}` syntax consistently
- [ ] New files referenced in `SKILL.md` and `prompts/meta-controller.md`
- [ ] JSON schemas validate: `python3 -c "import json; json.load(open('file'))"`
- [ ] Scripts have `#!/usr/bin/env bash` shebang and `set -euo pipefail`
- [ ] Cross-references resolve (no dangling paths)
- [ ] Sub-skill frontmatter has `name`, `description`, `triggers`
- [ ] `plugin.json` updated if new sub-skills added

## Architecture References

- `SKILL.md` — Skill functionality and behavior (source of truth)
- `CLAUDE.md` — Development guidelines
- `references/config-reference.md` — Config option catalogue
- `references/candle-vllm-catalog.md` — Supported model architectures
- `references/turboquant-guide.md` — KV-cache compression guidance

## Config Reference Maintenance

The `references/` directory must stay in sync with `src/config.rs` in the UAR repo.

When UAR adds new configuration:
1. Add to `references/config-reference.md` (all fields: key, env var, default, type, description)
2. Add to `references/env-vars.md` if a `UAR_*` var is added
3. Add to `references/cli-args.md` if a `--flag` is added
4. Update `assets/templates/config.yaml.template` with the new section/field
5. Update wizard questions in `prompts/wizard.md` if user-visible

## candle-vllm Model Catalog Maintenance

`references/candle-vllm-catalog.md` tracks supported architectures. Sources of truth:
- `candle-vllm/README.md` Supported Models table
- `candle-vllm/example.models.yaml` for configuration patterns
- `candle-vllm/crates/candle-vllm-core/src/` model implementations

## TurboQuant Guide Maintenance

`references/turboquant-guide.md` documents KV-cache compression decisions.
Source of truth: `turboquant-rs/README.md` + `example.models.yaml` `kvcache_compression` field documentation.
