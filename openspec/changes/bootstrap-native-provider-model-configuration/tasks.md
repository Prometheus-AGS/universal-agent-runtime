## 1. Environment

- [ ] 1.1 Add a non-echoing allowlist generator with canonical-wins alias precedence.
- [ ] 1.2 Restrict generated environment permissions and retain only KIMI, MINIMAX, DASHSCOPE, MOONSHOT, and ZAI canonical variables that resolve.

## 2. Provider/model YAML

- [ ] 2.0 Set native `server.host: 127.0.0.1`, `server.port: 1906`, and `server.grpc_port: 50051` without overriding existing operator values during merge.
- [ ] 2.1 Add the loopback OpenAI-proxy provider and discover its model inventory without fabricating unavailable results.
- [ ] 2.2 Add credential-conditional Kimi K3, MiniMax M3, Alibaba/Qwen, Z.AI/GLM, and Moonshot catalog entries.
- [ ] 2.3 Exclude RunPod, tool-only credentials, and endpoint/model-less entries.
- [ ] 2.4 Merge missing entries into existing YAML while preserving all existing values and persisted database authority.

## 3. Safety and cheap verification

- [ ] 3.1 Prove generated outputs and retained artifacts contain no literal credential values.
- [ ] 3.2 Strict-validate the change before committing it independently after code completion.
- [ ] 3.3 Integrate the bootstrap helpers into the native installer entrypoints and structurally confirm every referenced helper path exists; do not duplicate supervisor lifecycle logic.
