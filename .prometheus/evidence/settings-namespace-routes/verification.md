# Settings namespace route verification — 2026-08-25

## Candidate

- UAR branch: `codex/fix-settings-namespace-routes`
- KBD run: `fix-runtime-settings-namespace-routes-20260825T091750Z`
- KBD rollover pin: `f1e58b25b0a9926c24d1bb0ddb6c0678d16c6f49`
- Release binary SHA-256: `d4c91708afa4b173d7b9b5ff3ffaa917f02e0129804e1e206e622c7bcb0d7dcf`
- Installed binary SHA-256: `d4c91708afa4b173d7b9b5ff3ffaa917f02e0129804e1e206e622c7bcb0d7dcf`

## Local gates

- `pnpm typecheck`: passed.
- `pnpm lint`: passed.
- `pnpm -C frontend test settings-api.test.ts`: 4 passed.
- `pnpm -C frontend test src/shared/markdown/markdown-bubble.test.tsx`: 16 passed.
- `pnpm build`: passed.
- `node scripts/validate-static-bundle.mjs static`: passed; 11 referenced assets validated.
- `openspec validate fix-settings-namespace-read-routes --strict`: passed.
- `cargo build --locked --release --no-default-features --features server-full`: passed in 2m39s with three existing library warnings and two future-incompatibility notices.
- `pnpm frontend:boundaries`: failed on three per-row graph writes in `frontend/src/features/providers/model/providers-store.ts`; that file is unchanged from `origin/main` and provider graph behavior is outside this change.
- `pnpm test`: 328 passed and 12 failed. The failures are two unchanged provider-store mock failures, two unchanged ChoicePicker story validation failures, and eight unchanged entity-story Zod validation failures.
- `cargo fmt --all -- --check`: the changed RMCP registry is formatted, but the repository command reports pre-existing formatting drift in `src/server.rs`; that file is unchanged by this change.

## Installation and live proof

- Before install, `com.prometheus.universal-agent-runtime` was running on port 1906 with five configured providers.
- `packaging/native/macos/install.sh --binary target/release/universal-agent-runtime --static-dir static` completed and created `~/.prometheus/backups/uar/static.20260825T095750Z` plus the matching configuration backup.
- The config SHA-256 remained `9fe2e6ac9f75bdff2c8b05d6bfa502420b990b5f1da3f94f78f97c2b58063e46`.
- `/healthz` and `/readyz` both returned HTTP 200 on the first post-install probe.
- The canonical provider response still contains five durable provider IDs: `alibaba`, `kimi-for-coding`, `local-openai-proxy`, `minimax`, and `zai`. Startup logged `seeded=0`, confirming no provider rows were recreated.
- The outer settings-row UUID returned as `id` changes across process restarts because `surreal_value_to_setting()` intentionally constructs an in-memory proxy UUID on every read. It is not the provider ID; the durable provider ID is `data.id` and is also encoded by `key`.
- `pnpm -C frontend exec playwright test -c playwright.installed-settings-routes.config.ts`: 1 passed in 2.4s. The installed browser observed `/api/uar/settings/providers` and `/api/uar/settings/context-management`, observed no singular provider or underscored settings request, and observed no settings-route 404 or matching console error. Provider Overrides rendered all five configured providers without the misleading banner.
- Machine-readable browser evidence is in `playwright-report.json`.

## Residual risk

The full frontend suite and boundary checker remain red on defects already present in `origin/main`. This change does not certify those unrelated provider-store and A2UI surfaces. The installed settings-route scenario, focused API tests, release builds, static validation, service health/readiness, and provider-domain identity checks passed.
