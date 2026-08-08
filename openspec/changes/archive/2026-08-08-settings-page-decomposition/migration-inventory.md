# C-14b migration inventory

- Source: `frontend/src/features/settings/ui/settings-page.tsx` — 3,336 lines.
- Public contract: `features/settings/index.ts` exports only `SettingsPage` from `ui/settings-page`.
- Navigation: 29 items in five categories; default active key is `provider`.
- Shared responsibilities: navigation inventory, eight visual/status primitives, namespace wrapper, schema renderer, custom panel registry, route shell.
- Custom panel groups: AI/LLM (5), file processing (4), resilience (1), governance/agents (4), memory (1), caching/users (2). Remaining namespace keys render through `GenericSchemaPanel`.
- Existing focused settings behavior tests: model hook/store tests only; no route-level settings composition test exists before C-14b.
- Binding ceiling: no resulting settings page or panel module above approximately 600 lines.
- Protected scope inherited from C-14a: `.gitmodules`, `crates/prometheus-skill-system`, `src/uar/*`, staged `LICENSE-COMMERCIAL.md`, and staged `sdks/rust/LICENSE-AGPL` remain operator-owned.
