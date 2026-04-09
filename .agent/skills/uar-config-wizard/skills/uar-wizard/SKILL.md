---
name: uar-wizard
version: 1.0.0
description: >
  Guided first-time setup wizard for UAR. Asks targeted questions about
  deployment scenario, LLM provider, database backend, security, and optional
  features. Produces a tailored config.yaml and .env. If candle-vllm is
  chosen as the provider, automatically routes through model selection and
  TurboQuant configuration before generating the final files.
triggers:
  keywords:
    - "uar wizard"
    - "first time setup"
    - "set up uar for the first time"
    - "generate uar config"
    - "create config.yaml"
    - "/uar-wizard"
  when_to_use: >
    Use when the user is setting up UAR for the first time or wants a fresh
    guided configuration experience.
---

# UAR Guided Setup Wizard

I will guide you through configuring the Universal Agent Runtime step by step.

## What I collect

1. **Deployment scenario** — local dev, Docker Compose, Kubernetes, cloud VM, or candle-vllm stack
2. **LLM provider** — cloud API (OpenAI, Anthropic, Groq, etc.), candle-vllm (local), Ollama, or custom endpoint
3. **Database** — PostgreSQL (default) or SurrealDB, connection string pattern
4. **Security** — JWT settings, secret generation reminder
5. **Optional features** — Redis cache, memory system, file processing, vision
6. **Advanced resilience** — rate limits, retries, timeouts (opt-in)

If you choose **candle-vllm** as your LLM provider, I will also:
- Profile your hardware (GPU/VRAM/RAM)
- Search for the best compatible model
- Configure TurboQuant KV-cache compression
- Generate a `candle-vllm-models.yaml` alongside your UAR config

## Output

- `config.yaml` — Complete UAR configuration
- `.env` — All required environment variables
- `candle-vllm-models.yaml` (if candle-vllm chosen)
- `quickstart.sh` (if candle-vllm chosen)

## Entry point

On invocation, load `prompts/wizard.md` and start with Group 1 (deployment scenario).
Invoke subagent: `agents/wizard-guide.md`
