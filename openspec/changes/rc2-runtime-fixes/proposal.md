## Why

Five open issues block 1.4.0-rc.2. Each is a configuration or contract the runtime already claims but doesn't show or enforce:

- **#419:** model precedence between `UAR_LLM__MODEL`, `LLM_MODEL` and the config file is fixed in source but has no behavior proof.
- **#420:** the startup log names `embedding_provider` even when `embedding_api` selected a different adapter.
- **#385:** a catch-all model-binding failure logs no cause. A budgeted agent is also refused because the output reservation ignores the request's `max_tokens`.
- **#396:** `/v1/messages` always runs an agent, but nothing documents this or proves parity with chat completions.
- **#391:** nothing stops a release from bumping the root version without the Rust SDK's exact runtime pin.

## What Changes

- **#419, #420 startup log:** reports the embedding adapter memory actually uses, through one shared resolver (`MemoryConfig::resolved_embedding_api`).
- **#385 cause logging:** the catch-all `client_capture_failed` binding failure logs its scrubbed cause at ERROR. The run-stream message stays generic.
- **#385 reservation sizing:** the non-team output reservation is the request's output cap when one is present, otherwise the catalog maximum.
- **#396 docs:** new `docs/ANTHROPIC_MESSAGES.md` covers agent selection, unsupported bound targets and mid-run switching, request fields, streaming, caller tools and errors.
- **#391 pin check:** new `pnpm sdk-runtime-pin:validate` checks the Rust SDK's exact runtime pin against the root version.
- **BDD:** `runtime-configuration.feature` covers #419, #420 and #385, and `anthropic-messages-parity.feature` covers #396. Both run on dedicated runtimes with the stub model and a stub embedder.

## Impact

- Code: `src/config.rs`, `src/server.rs`, `src/uar/memory/service.rs`, `src/uar/runtime/manager.rs`, `src/uar/runtime/cost_budget.rs`.
- Scripts and docs: `scripts/validate-sdk-runtime-pin.mjs`, `package.json`, `docs/ANTHROPIC_MESSAGES.md`, `docs/ANTHROPIC.md` (a pointer only).
- No API or storage changes.
