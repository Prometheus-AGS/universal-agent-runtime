# ASSESSMENT: fix-provider-model-picker-key-mask

Project: universal-agent-runtime
Date: 2026-08-25
Codebase baseline: The provider overrides panel is implemented and uses the existing settings entity/store path, but model selection is free text and secret masking discards key length.
Cross-tool progress: none

## IMPLEMENTATION STATUS

- Provider default-model selection: **MISSING** — `ProviderPanel` in `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx` renders `default_model` with a free-text `Input`. Each provider settings value already contains its configured `models[]`, and `frontend/src/components/ui/select.tsx` plus `SettingSelect` in `frontend/src/features/settings/ui/settings-primitives.tsx` provide the repository-owned shadcn control.
- Provider model inventory: **DONE** — provider settings are persisted as one object per provider and include model ids, display names, and enabled state. That provider-owned enabled list is the execution-valid set for this configured provider; the global `catalog/provider_catalog.json` can contain models that are not enabled for the current provider record. No new fetch, graph entity, store, or transport is required to populate the control.
- API-key retrieval masking: **PARTIAL** — `mask_sensitive` and `mask_setting_data` in `src/uar/api/settings.rs` replace every sensitive string with the fixed value `"***"`, so a key of any length renders as three password glyphs. Object masking also inserts a mask for an absent sensitive property instead of preserving absence. The provider schema that marks `api_key` sensitive lives in `src/uar/settings/manager.rs`.
- API-key save preservation: **PARTIAL** — scalar sensitive settings preserve an empty string or literal `"***"`, but provider settings are object rows (`provider.<id>`). The object row itself is not marked sensitive, so saving another provider field can submit the visible mask as the new `api_key`.
- Focused regression coverage: **MISSING** — the settings page test proves navigation only. The settings API unit test asserts the fixed three-character mask and does not cover variable-length masking or nested-object mask preservation.

## CROSS-TOOL PROGRESS

- NONE — the new phase ledger has no registered changes or tasks.

## SPEC GAP SUMMARY

- `frontend-configuration-surfaces` requires provider/model semantics and visible controls to be preserved, but it does not specify that provider default models come from a bounded shadcn control.
- No current scenario requires a sensitive settings response to retain the original secret's character count while obscuring every character.
- No current scenario requires an unchanged nested API-key mask to round-trip without replacing the stored credential.

## BUILD HEALTH

- frontend typecheck: **PASS** — `pnpm -C frontend typecheck` ran `tsc -b` and exited 0.
- frontend lint: **PASS** — `pnpm -C frontend lint` ran `eslint .` and exited 0.
- focused frontend test: **PASS** — `pnpm -C frontend test src/features/settings/ui/settings-page.test.tsx` passed 1 file and 4 tests.
- Rust baseline test: **UNKNOWN** — `cargo test retrieval_masking_redacts_scalar_and_object_secrets` selected the broad default feature graph and was interrupted after more than ten minutes of cold compilation; no test result was produced.
- Rust verification risk: the plan must use the repository-prescribed `--locked --no-default-features --features server-full` profile for T0 and scope T1 to the just-written settings API test. Cold compile cost remains a scheduling risk, not permission to report an unrun check as passing.
- known violations: the interrupted broad Rust compile emitted pre-existing dead-code warnings for `MAX_BODY_BYTES` and `MAX_REDIRECTS` in `src/uar/tools/fetch_guard.rs`. They are outside this request and must not be changed here.
- test coverage: **MINIMAL** — the existing backend masking test covers retrieval shape only; the requested provider control and exact-length behavior are uncovered.
- required frontend tier: T0 is `pnpm typecheck` plus `pnpm lint` after the edit; T1 is a focused Vitest component test for the provider panel. T2 (`pnpm build` and the full `pnpm test`) is required only when the phase implementation is complete.

## CONSTRAINT CHECK

- AGENTS.md violations: NONE in the proposed path. The implementation can keep the existing `component -> useSettings -> settings store -> settings API` layering and derive model options from the provider settings object without new business-state authority.
- `.kbd-orchestrator/constraints.md` violations: N/A for the inspected surface.
- UI routing gap: UI/UX Pro Max, Context7 shadcn docs, Vercel React Best Practices, Vercel Composition Patterns, frontend-design, and entity form guidance were consulted. The prescribed `ux-designer` and Impeccable commands were not installed or exposed in this session.

## GOAL PROGRESS

- Use a shadcn UI picker populated with every valid provider model: **NOT MET** — the control is currently free text.
- Mask API keys with exactly one obscuring character per current key character: **NOT MET** — all values currently become `"***"`.
- Verify behavior locally at the required frontend tier: **NOT MET** — no implementation or new regression tests exist yet.

## ASSESSMENT COMPLETE

The minimum coherent change is one OpenSpec delta, a bounded provider model select with focused React coverage, and length-preserving settings API masking with nested-object preservation and focused Rust coverage. The uncomfortable case is saving a harmless field such as protocol while a masked key is present: unless nested preservation is fixed at the API boundary, the real credential can be overwritten by the mask.
