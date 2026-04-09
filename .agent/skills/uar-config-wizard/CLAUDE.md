# Claude Code Development Guide

This file provides guidance for AI assistants working **on** this skill. For user-facing behavior, see `SKILL.md`.

## Architecture

PMPO (Prometheus Meta-Prompting Orchestration) adapted for configuration assistance and model selection:
- **Phase controllers** in `prompts/` drive each mode
- **Subagents** in `agents/` specialize per task (interview, explanation, validation, generation, model research)
- **References** in `references/` are the authoritative knowledge base — extracted from `src/config.rs` + live research
- **Schemas** in `references/schemas/` define session state contracts
- **Templates** in `assets/templates/` scaffold all generated output files
- **Scripts** in `scripts/` manage session state lifecycle
- **Sub-skills** in `skills/` are slash command entry points

## Key Files

| File | Role |
|------|------|
| `SKILL.md` | Canonical skill definition |
| `prompts/meta-controller.md` | Orchestration entry point — mode routing, PMPO loop |
| `prompts/model-select.md` | 5-phase model advisor loop with Tavily web search |
| `references/config-reference.md` | Complete config key → env var → default → description |
| `references/candle-vllm-catalog.md` | Supported architectures, VRAM requirements, quantization formats |
| `references/turboquant-guide.md` | KV-cache compression sizing, bits/policy decision table |
| `assets/templates/candle-vllm-models.yaml.template` | candle-vllm model definition output template |
| `assets/templates/quickstart.sh.template` | CLI quickstart script template |

## Config Reference Sync Rule

Files in `references/` are extracted from `src/config.rs` and `example.config.yaml`. When UAR adds new config fields:
1. Update `references/config-reference.md` (key, env var, default, notes)
2. Update `references/env-vars.md` for new `UAR_*` vars
3. Update `references/cli-args.md` for new `--flag` args
4. Update `assets/templates/config.yaml.template`
5. Update `prompts/wizard.md` question set if user-facing

## candle-vllm Catalog Sync Rule

`references/candle-vllm-catalog.md` tracks supported model architectures. When candle-vllm adds a new architecture:
1. Add a row to the Architecture Support table
2. Update VRAM requirements section
3. Add quantization notes if applicable
4. Update `assets/templates/candle-vllm-models.yaml.template` with a new example entry

## Modifying Phase Controllers

Each prompt in `prompts/` follows this structure:
1. Role/purpose
2. Inputs
3. Process (numbered steps)
4. Output contract (YAML)
5. Rules

Always update cross-references when renaming files.

## Model Selection Phase (`prompts/model-select.md`)

The model-select prompt runs a 5-phase loop:
1. **Hardware profile** — gather GPU/VRAM/RAM/platform
2. **Requirements** — context, capabilities, latency/quality tradeoff
3. **Live research** — Tavily web_search calls to models.dev, HuggingFace, liter-llm docs
4. **Scoring** — VRAM fit (0.35) × capability (0.30) × speed (0.20) × cost (0.15)
5. **TurboQuant config** — `kvcache_compression.bits` from VRAM headroom table

TurboQuant bits decision table (in `references/turboquant-guide.md`):
- `>30%` VRAM free → disabled
- `15–30%` → 4-bit + `memory_pressure`
- `5–15%` → 3-bit + `threshold_tokens: 4096`
- `<5%` → 3-bit + `always`

## Adding a New Sub-Skill

1. Create `skills/<command-name>/SKILL.md` with YAML frontmatter
2. Define setup, user input parsing, and behavior
3. Add the skill path to `.claude-plugin/plugin.json` `skills` array
4. Update `prompts/meta-controller.md` routing table

## Template System

Templates use `{{variable}}` syntax:

| Variable | Source | Example |
|----------|--------|---------|
| `{{uar_port}}` | Wizard output | `3000` |
| `{{jwt_secret}}` | Generated | `openssl rand -base64 64` |
| `{{candle_vllm_url}}` | User input | `http://localhost:3000` |
| `{{selected_model_hf_id}}` | Model-select output | `meta-llama/Llama-3-8B-Instruct` |
| `{{turboquant_bits}}` | VRAM analysis | `3` |
| `{{turboquant_policy}}` | VRAM analysis | `threshold_tokens: 4096` |
| `{{provider_id}}` | User/wizard | `candle-vllm` |

## Script Reference

| Script | Purpose |
|--------|---------|
| `scripts/state-init.sh` | Init / resume named config session |
| `scripts/state-checkpoint.sh` | Mid-phase snapshot |
| `scripts/state-finalize.sh` | Archive completed session |
| `scripts/validate-config.sh` | YAML syntax + required-field validation |
| `scripts/workflow-dispatch.sh` | Lifecycle event trigger dispatcher |

## Testing

```bash
# Validate YAML frontmatter in all sub-skills
for f in skills/*/SKILL.md; do head -3 "$f" | grep -q "^---" && echo "✅ $f" || echo "❌ $f"; done

# Validate all JSON schemas
for f in references/schemas/*.json; do
  python3 -c "import json; json.load(open('$f')); print(f'✅ $f')" 2>/dev/null || echo "❌ $f"
done

# Check script executability
for f in scripts/*.sh; do [ -x "$f" ] && echo "✅ $f" || echo "❌ $f"; done

# Validate plugin manifest
python3 -c "import json; json.load(open('.claude-plugin/plugin.json')); print('✅ plugin.json')"

# Cross-reference integrity check
grep -roh 'references/[a-zA-Z0-9/_.-]*' prompts/ | sort -u | while read f; do
  [ -e "$f" ] && echo "✅ $f" || echo "❌ $f"
done
```
