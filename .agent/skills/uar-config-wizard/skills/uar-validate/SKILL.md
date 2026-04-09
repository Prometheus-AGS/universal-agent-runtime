---
name: uar-validate
version: 1.0.0
description: >
  Validates an existing UAR config.yaml and/or .env file. Checks YAML syntax,
  required fields, type correctness, deprecated keys, security issues
  (default JWT secret detection), and candle-vllm integration correctness.
  Reports errors (❌), warnings (⚠️), and info (ℹ️) with exact fix instructions.
triggers:
  keywords:
    - "validate config"
    - "check config"
    - "is my config correct"
    - "config errors"
    - "validate uar config"
    - "/uar-validate"
  when_to_use: >
    Use when the user has an existing config.yaml or .env and wants to check
    it for errors before running UAR.
---

# UAR Config Validator

I will check your `config.yaml` and/or `.env` for errors and provide exact fix instructions.

## What I check

- **YAML syntax** — Parse errors with line numbers
- **Required fields** — `persistence.database_url`, `llm.model`, `security.jwt_secret`
- **Format validation** — `llm.model` must be `provider/model` format
- **Security** — Default JWT secret placeholder detection (critical)
- **Deprecated keys** — `LLM_*` env vars flagged with migration path
- **candle-vllm** — `protocol: chat` required (not `responses`)
- **Type checks** — Port as integer, rates as floats, booleans correct

## Usage

Point me at your files:
```
Validate ./config.yaml and .env
```
Or specify paths:
```
Validate /etc/uar/config.yaml
```

## Entry point

On invocation, load `prompts/validate.md`.
Invoke subagent: `agents/validator.md`
