## ADDED Requirements

### Requirement: Native provider seed contains only concrete supported models
The native seed SHALL include the discovered local OpenAI proxy inventory, Kimi Coding `kimi-for-coding/k3`, MiniMax `minimax/MiniMax-M3`, and credential-matched Alibaba/Qwen, Z.AI/GLM, and Moonshot catalog models. It SHALL exclude tool-only credentials and providers lacking a concrete endpoint/model.

#### Scenario: Local proxy inventory is refreshed
- **WHEN** `http://127.0.0.1:8181/v1/models` returns a model list
- **THEN** missing proxy model entries are merged into YAML without replacing existing provider, model, or default selections

### Requirement: Provider bootstrap preserves persistent settings authority
Native bootstrap SHALL merge missing YAML entries and SHALL NOT replace API/UI-created provider settings or the selected default model held in persistent storage.

#### Scenario: Persisted provider differs from catalog default
- **WHEN** the operator has edited a provider through the API or UI after initial seed
- **THEN** a service restart retains the persisted setting rather than restoring the YAML/catalog default
