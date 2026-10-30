## 1. Implementation

- [x] 1.1 Shared `MemoryConfig::resolved_embedding_api`; the memory init log names the adapter in use (#420)
- [x] 1.2 Scrubbed cause on the `client_capture_failed` binding-failure log (#385)
- [x] 1.3 Output reservation sized by the request output cap (#385)
- [x] 1.4 `docs/ANTHROPIC_MESSAGES.md` (#396)
- [x] 1.5 `scripts/validate-sdk-runtime-pin.mjs` and `pnpm sdk-runtime-pin:validate` (#391)
- [x] 1.6 BDD: `runtime-configuration.feature`, `anthropic-messages-parity.feature`; fixture environment overrides and log entries; recording embedding stub

## 2. Verification

- [ ] 2.1 server-full and postgres-backend builds with zero warnings
- [ ] 2.2 `openspec validate rc2-runtime-fixes`
- [ ] 2.3 `pnpm sdk-runtime-pin:validate`
- [ ] 2.4 `runtime-configuration.cucumber.mjs` (five scenarios)
