---
name: uar-stack
version: 1.0.0
description: >
  Full-stack bundle generator for UAR + candle-vllm. Runs the guided wizard
  and model-select advisor in sequence, then emits a complete,
  immediately-runnable bundle: UAR config.yaml, candle-vllm models.yaml with
  TurboQuant kvcache_compression, .env with all secrets, quickstart.sh CLI
  script, and optional Kubernetes manifests. One command to configure the
  entire local LLM inference + agent runtime stack.
triggers:
  keywords:
    - "full stack config"
    - "uar stack"
    - "configure everything"
    - "candle-vllm and uar"
    - "local llm stack"
    - "all config files"
    - "/uar-stack"
  when_to_use: >
    Use when the user wants to configure the complete UAR + candle-vllm stack
    in one pass. Best for users setting up local LLM inference for the first time.
---

# UAR Full Stack Bundle Generator

One command to configure the complete UAR + candle-vllm stack. I generate all configuration files you need to go from zero to running.

## What I produce

```
config.yaml               UAR configuration (candle-vllm as provider)
candle-vllm-models.yaml   Model definition + TurboQuant kvcache_compression
.env                      All secrets and keys
quickstart.sh             CLI commands to launch both services
k8s/                      Kubernetes manifests (optional, ask to include)
```

## How it works

### Step 1 — UAR Configuration Wizard
Collects your deployment scenario, database backend, security settings, and optional features.

### Step 2 — candle-vllm URL
I ask for your candle-vllm instance URL (e.g., `http://localhost:3000`).

### Step 3 — Model Selection + TurboQuant
I profile your hardware, search for compatible models, score candidates by VRAM fit and capability, then configure TurboQuant KV-cache compression for maximum context efficiency.

### Step 4 — Generate All Files
I emit all files using the collected information. Nothing is written to disk until you confirm.

### Step 5 — Validation
All generated YAML is validated before presentation.

## Quick start

```
/uar-stack
```

Or with your candle-vllm URL upfront:
```
/uar-stack --candle-vllm-url http://my-server:3000
```

With Kubernetes manifests:
```
/uar-stack --with-k8s
```

## Entry point

On invocation, load `prompts/meta-controller.md` with `mode: stack`.
Orchestration: `wizard → model-select → generate → validate`
Agents: `wizard-guide` → `model-advisor` → `generator` → `validator`
