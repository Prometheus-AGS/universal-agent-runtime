# UAR core model policy

This policy keeps model identity native to each harness. It does not copy gateway aliases into unrelated provider namespaces. Catalog discovery records configured metadata; only a successful authorized request proves inference access.

## Role allocation

| Roles | Codex | Claude Code |
|---|---|---|
| `uar-lead`, `uar-trust-tools`, `uar-verifier` | `gpt-6-astra`, effort `high` | `opus`, effort `high` |
| `uar-runtime`, `uar-providers`, `uar-state`, `uar-console` | `gpt-5.6-terra`, effort `high` | `sonnet`, effort `high` |

Codex role TOML sets both `model` and `model_reasoning_effort`; changing only the model can retain an incompatible inherited effort. Both selected IDs and `high` were present in the installed Codex 0.154.0 model catalog. No inference entitlement was proven. Claude uses native aliases rather than assuming a provider-specific full ID; official documentation supports `opus`/`sonnet` and `high`, but aliases and account availability vary by provider.

The default concurrency budget is three active agents and the maximum is four, including the lead. A more expensive role is not a reason to create extra concurrent work. The lead may handle a narrow task directly rather than spawning every specialist.

## Kimi Code

The launcher defaults the invocation to the observed alias `kimi-code/k3`. Pass `--model <native-alias>` or `-m <native-alias>` to override it. Kimi ignores model and effort fields in custom-agent Markdown, so role files do not promise per-role routing.

Installed Kimi 0.42.0 exposed `kimi-code/k3`, `kimi-code/k3-256k`, `kimi-code/kimi-for-coding`, and `kimi-code/kimi-for-coding-highspeed`. Both k3 aliases declare `low`, `high`, and `max`, defaulting to `high`. The launcher does not invent a CLI effort option. A configured native secondary-model pool can change subagent selection; inherited or explicitly selected pool behavior remains native policy.

## OpenCode

Inherit the user's authenticated current model. Project agent files do not force a provider from a broad model catalog. If that provider is already configured and the account can use it, observed IDs such as `openai/gpt-6-astra` for demanding roles or `openai/gpt-5.6-terra` for implementation are candidates. `openai/gpt-5.5`, `openai-proxy/gpt-5.5`, `minimax/MiniMax-M3`, and Kimi provider IDs were also listed, but names alone do not establish credentials or compatibility.

Use native `provider/model` IDs and provider-supported variants. Do not assume Codex's `model_reasoning_effort` or Claude's `effort` field applies to OpenCode. Review actual authenticated provider metadata before adding an override.

## MiniMax Code

Inherit the model configured for the isolated `.minimax` account. `minimax/MiniMax-M3` is a source-documented candidate only after the matching provider has been configured and shown available in that isolated environment. A gateway alias such as `MiniMax-M3` alone does not satisfy the native `provider/model` syntax.

No installed MiniMax CLI model discovery or inference was possible at the research boundary. Do not claim that staging a model field authenticates the account. The source parser accepts an effort string, including tests for MiniMax M3 `on`/`off`; this is not a portable reasoning-effort enumeration and no default is forced here.

## Refreshing evidence

- Codex: `codex debug models`; report only model identifiers and supported efforts. `--bundled` reads bundled metadata without proving account access.
- Kimi: `kimi provider list --json`; project only model metadata before sharing because raw provider configuration can include sensitive values.
- OpenCode: `opencode models --pure`; catalog entries do not establish successful authentication. Avoid dumping resolved config.
- Claude: inspect native `/model` and official provider-aware alias documentation; do not infer entitlement from an alias.
- MiniMax: after installation, run the isolated launcher's `provider list --json` and sanitize the result. `provider test` and `provider add --use` can invoke inference and are separate from read-only discovery.

See [harness-research.md](harness-research.md) for exact sources, observed CLI versions, and verification limits. The research candidate roster is historical input; the seven-role roster in [README.md](README.md) is the installed team's policy.
