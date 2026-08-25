## Why

Provider Overrides currently accepts an arbitrary default-model string even though the configured provider already declares its valid enabled models, and its API-key field always renders three mask characters regardless of the stored key length. This makes invalid model configuration easy and prevents the settings surface from accurately representing the current credential without exposing it.

## What Changes

- Replace the provider default-model text field with the repository-owned shadcn select control.
- Populate each provider's model options from every enabled model in that configured provider's persisted `models[]` list.
- Obscure every stored API-key character with one mask character while returning no plaintext secret material.
- Preserve an unchanged masked credential when another field in the provider object is saved, so the mask cannot replace the stored API key.
- Add focused frontend and settings API regression coverage for model selection, mask length, absent keys, and masked-key round trips.

## Capabilities

### New Capabilities

- `agent-ui-design-workflow`: Define the repository's precedence for installed UI-design skills and require dual-agent Impeccable critique plus fresh-context adversarial review as the standard UI quality gate.

### Modified Capabilities

- `frontend-configuration-surfaces`: Require bounded shadcn provider model selection and length-preserving, non-destructive API-key masking on the Provider Overrides surface.

## Impact

- Runtime UX: Provider Overrides gains a keyboard-accessible bounded model selector and an API-key mask whose visible length matches the stored credential.
- Provider compatibility: The control uses the configured provider's enabled model inventory and does not broaden or reinterpret global catalog support.
- Realtime state: The existing settings entity projection, draft cache, store, save/reload actions, and realtime updates remain unchanged.
- Backend API: Sensitive settings retrieval and preservation logic changes without exposing plaintext or changing endpoint and payload shapes.
- Dependencies: No dependency changes; the project already owns the required shadcn primitives.
- KBD: Work is tracked by phase `fix-provider-model-picker-key-mask`; assessment and plan artifacts must remain synchronized with this OpenSpec change.
- Agent guidance: `AGENTS.md` records the operator-requested UI skill precedence and independent review standard; the `CLAUDE.md` symlink inherits the same rule.
