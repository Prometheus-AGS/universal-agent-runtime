---
name: uar-config
version: 1.0.0
description: >
  Main entry point for the UAR Configuration Wizard. Auto-detects intent
  and routes to the appropriate mode: wizard (first-time setup), validate
  (check existing config), migrate (upgrade legacy env vars), model-select
  (choose a model), k8s (generate Kubernetes manifests), or stack (full bundle).
triggers:
  keywords:
    - "configure uar"
    - "uar config"
    - "set up uar"
    - "uar setup"
    - "config wizard"
  when_to_use: >
    Use when the user needs any kind of UAR configuration help and has not
    specified which mode. This skill detects intent and routes automatically.
---

# UAR Configuration Wizard — Auto-Router

Invoke this skill to get started. I will detect what you need and route you to the right mode.

## What I can do

- **First-time setup**: Generate `config.yaml` and `.env` through guided Q&A
- **Validate**: Check your existing `config.yaml` / `.env` for errors
- **Migrate**: Upgrade `LLM_*` env vars to the `UAR_*` prefix convention
- **Model selection**: Help you choose the right LLM for your hardware
- **Kubernetes**: Generate K8s `Secret` + `ConfigMap` manifests
- **Full stack bundle**: Configure UAR + candle-vllm together (all files)

## Getting started

Tell me what you're trying to do, or use a specific command:
- `/uar-wizard` — Guided first-time setup
- `/uar-validate` — Check an existing config
- `/uar-migrate` — Upgrade legacy env vars
- `/uar-model-select` — Choose a model
- `/uar-k8s-config` — Generate Kubernetes manifests
- `/uar-stack` — Full UAR + candle-vllm bundle

## Entry point

On invocation, load `prompts/meta-controller.md` and analyze the user's intent.
