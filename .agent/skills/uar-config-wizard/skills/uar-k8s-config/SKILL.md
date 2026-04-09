---
name: uar-k8s-config
version: 1.0.0
description: >
  Generates Kubernetes manifests for UAR deployment: a Secret containing
  sensitive fields (jwt_secret, database_url, API keys) and a ConfigMap
  containing non-sensitive settings. Optionally also generates a candle-vllm
  Deployment manifest when used with the candle-vllm stack pattern.
triggers:
  keywords:
    - "kubernetes config"
    - "k8s manifests"
    - "k8s secret"
    - "configmap uar"
    - "deploy uar to kubernetes"
    - "/uar-k8s-config"
  when_to_use: >
    Use when the user wants to deploy UAR to Kubernetes and needs properly
    split Secret and ConfigMap manifests.
---

# UAR Kubernetes Config Generator

I will generate Kubernetes `Secret` and `ConfigMap` manifests for UAR deployment, correctly splitting sensitive and non-sensitive configuration.

## What I generate

### `k8s/uar-secret.yaml`
Sensitive fields encoded as Kubernetes Secret:
- `jwt_secret`
- `database_url`
- `redis_url` (if applicable)
- LLM API keys

### `k8s/uar-configmap.yaml`
Non-sensitive environment variables:
- `UAR_SERVER__PORT`
- `UAR_LLM__MODEL`
- `UAR_LLM__PROTOCOL`
- Rate limits, timeouts, feature flags

### `k8s/candle-vllm-deployment.yaml` (optional)
If using candle-vllm local inference, a Deployment spec for the candle-vllm server with the selected model.

## Usage

```
Generate k8s config
```
Or from existing config.yaml:
```
Generate k8s manifests from my ./config.yaml
```
Or as part of a full stack:
```
/uar-stack --with-k8s
```

## Security best practice

Sensitive fields are kept in `Secret`, non-sensitive in `ConfigMap`. The Deployment references Secrets via `secretKeyRef` — never plain text env vars for secrets.

## Entry point

On invocation, check if wizard answers exist in session state. If not, run a quick abbreviated wizard first. Then load `prompts/generate.md` in k8s mode.
Invoke subagent: `agents/generator.md`
