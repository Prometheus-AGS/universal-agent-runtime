---
name: uar-migrate
version: 1.0.0
description: >
  Migrates legacy LLM_* and bare environment variable names to the UAR_*
  prefix convention. Scans .env files, detects deprecated variables, fixes
  provider/model format issues (e.g., "gpt-4o" → "openai/gpt-4o"), and
  produces an updated .env with a clear before/after diff. Backward
  compatible — both old and new vars work, UAR_* takes precedence.
triggers:
  keywords:
    - "migrate env vars"
    - "upgrade env"
    - "LLM_API_KEY deprecated"
    - "update env to uar"
    - "migrate to uar prefix"
    - "/uar-migrate"
  when_to_use: >
    Use when the user has legacy LLM_* or bare env vars and wants to upgrade
    to the UAR_* prefix convention used by UAR 2.x+.
---

# UAR Env Var Migration Assistant

I will scan your `.env` for legacy variable names and produce an updated version with the correct `UAR_*` prefix convention.

## What I migrate

| Legacy | UAR Equivalent |
|--------|---------------|
| `LLM_API_KEY` | `UAR_LLM__API_KEY` |
| `LLM_MODEL` | `UAR_LLM__MODEL` |
| `LLM_BASE_URL` | `UAR_LLM__BASE_URL` |
| `LLM_PROTOCOL` | `UAR_LLM__PROTOCOL` |
| `PORT` | `UAR_SERVER__PORT` |
| `JWT_REQUIRED` | `UAR_SECURITY__JWT_REQUIRED` |

## Model format fix

I also detect bare model names and add the provider prefix:
- `gpt-4o` → `openai/gpt-4o`
- `claude-3-5-sonnet` → `anthropic/claude-3-5-sonnet-20241022`
- `llama-3` → asks which provider (groq/together/local)

## Backward compatibility

Both old and new vars work simultaneously. `UAR_*` takes precedence over `LLM_*`. Safe to migrate gradually.

## Usage

```
Migrate my .env file
```
Or:
```
Migrate /path/to/.env
```

## Entry point

On invocation, load `prompts/migrate.md`.
Invoke subagent: `agents/config-advisor.md`
