## ADDED Requirements

### Requirement: macOS installed release is functionally verified
After code completion, the phase SHALL build and install the release binary and React bundle, load the LaunchAgent on port 1906, and observe health, readiness, UI/static assets, loopback-only listeners, provider/model visibility, genuine inference, persistence across one restart, database access, graceful shutdown, and required logging.

#### Scenario: Installed LaunchAgent is restarted
- **WHEN** the operator restarts the LaunchAgent after successful inference
- **THEN** it becomes ready again with configuration, provider visibility, database access, operational logging, and genuine inference intact

### Requirement: Native Alibaba configuration uses the released Qwen flagship
When an Alibaba credential is present, native bootstrap SHALL seed `qwen3.8-max` with the documented one-million-token context and 131,072-token maximum output. It SHALL migrate only the exact obsolete native selection `alibaba/qwen3.7-max`, malformed credential reference `QWEN_TOKENPLAN_API_KEY`, and phase-owned `qwen3-coder-plus` seed. Other operator selections and custom Alibaba provider blocks SHALL remain unchanged.

#### Scenario: Interrupted native installation is refreshed
- **WHEN** the existing native YAML contains the exact obsolete Alibaba values observed during this phase
- **THEN** refresh selects `alibaba/qwen3.8-max`, refers to canonical `DASHSCOPE_API_KEY`, updates the phase-owned provider seed, and leaves all unrelated YAML unchanged

#### Scenario: Operator owns a different Alibaba configuration
- **WHEN** the existing model, credential reference, or Alibaba provider block does not exactly match the obsolete phase values
- **THEN** bootstrap preserves that operator-owned value rather than applying a broad Qwen migration

### Requirement: Newly released native models enter through pinned catalog sources
The model API SHALL continue to use the compile-time catalog. When the pinned catalog sources have added a required newly released model, UAR SHALL advance the `models.dev` and `liter-llm` gitlinks and regenerate the reviewed offline UAR snapshot rather than introduce a second configured-model overlay.

#### Scenario: Updated catalog contains Qwen 3.8-Max
- **WHEN** the pinned `models.dev` and `liter-llm` commits contain Alibaba `qwen3.8-max`
- **THEN** the reviewed offline snapshot and release build expose that model through `/api/models` and the Models UI without changing the endpoint implementation or either submodule's source
