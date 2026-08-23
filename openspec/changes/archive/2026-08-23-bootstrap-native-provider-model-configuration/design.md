## Context

The Bash login environment includes both model credentials and unrelated tool credentials. UAR's embedded catalog already owns provider endpoints, key variable names, and models. Bootstrap should transfer only approved names, then let catalog enrichment and persistent settings do their existing jobs.

## Decisions

1. Canonical variables win; aliases fill only a missing canonical variable. For Kimi, `KIMI_CODING_API_KEY` wins over `KIMI_CODING_KEY`; for Qwen, `QWEN_API_KEY` wins over `QWEN_TOKEN_PLAN_API_KEY`.
2. The allowlist contains KIMI, MINIMAX, DASHSCOPE, MOONSHOT, and ZAI credential names only.
3. Kimi Coding, MiniMax, Alibaba/Qwen, Z.AI/GLM, and Moonshot entries are emitted only with matching credentials; local proxy models come from `/v1/models` at install/refresh time.
4. RunPod and tool-only credentials are excluded without a concrete catalog endpoint/model.
5. YAML contains references and non-secret metadata, never literal keys.

## Risks

- Cross-endpoint aliasing can send a key to the wrong service; Moonshot and Z.AI have no aliases.
- Existing database provider rows intentionally win after seed; bootstrap must not overwrite them.
- A failed proxy inventory request must not fabricate model names.
