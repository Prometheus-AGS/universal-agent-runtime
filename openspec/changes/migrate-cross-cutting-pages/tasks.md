## Status — 2026-05-27

**DEFERRED to per-entity sessions.** Each entity needs to migrate **every** consumer in a single PR (admin page + all cross-view consumers) and be verified in the browser with a two-tab propagation test before the next entity starts.

## 1. Agents

- [ ] 1.1 Migrate `admin/pages/agents-page.tsx`.
- [ ] 1.2 Migrate `features/chat/agent-selector.tsx`.
- [ ] 1.3 Migrate `useAgentConfig` hook.
- [ ] 1.4 Migrate chat header agent label.
- [ ] 1.5 Migrate default-agent fallback in `chat-page.tsx`.
- [ ] 1.6 Delete `stores/agents-admin-store.ts`.
- [ ] 1.7 `git grep useAgentsAdminStore` returns zero.

## 2. Providers

- [ ] 2.1 Migrate `providers-page.tsx`.
- [ ] 2.2 Migrate `session-config-panel.tsx` provider section.
- [ ] 2.3 Migrate any header provider chip.
- [ ] 2.4 Delete `stores/providers-admin-store.ts`.
- [ ] 2.5 `git grep useProvidersAdminStore` returns zero.

## 3. Models

- [ ] 3.1 Migrate `models-page.tsx`.
- [ ] 3.2 Migrate model selector in header.
- [ ] 3.3 Migrate `capability-toggles.tsx` model picker.
- [ ] 3.4 Migrate `useProviderModels` derived hook.
- [ ] 3.5 Delete `stores/models-browse-store.ts`.
- [ ] 3.6 `git grep useModelsBrowse` returns zero.

## 4. Skills

- [ ] 4.1 Migrate `skills-page.tsx` (preserve Built-in badge).
- [ ] 4.2 Migrate `capability-toggles.tsx` skills section.
- [ ] 4.3 Migrate agent → skills binding.
- [ ] 4.4 Delete `stores/skills-admin-store.ts`.
- [ ] 4.5 `git grep useSkillsAdminStore` returns zero.

## 5. Settings

- [ ] 5.1 Migrate `settings-page.tsx`.
- [ ] 5.2 Migrate `session-config-panel.tsx` settings section.
- [ ] 5.3 Migrate global feature toggle consumers.
- [ ] 5.4 Delete `stores/settings-store.ts`.
- [ ] 5.5 `git grep useSettingsStore` returns zero.

## 6. Cross-view smoke

- [ ] 6.1 Two-tab provider-edit propagation test.
- [ ] 6.2 Two-tab agent-enable propagation test.
- [ ] 6.3 Two-tab setting-edit propagation test.
