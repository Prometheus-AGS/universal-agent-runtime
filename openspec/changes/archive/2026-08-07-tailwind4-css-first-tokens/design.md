## Context

The frontend currently uses Tailwind 3.4.19 through `tailwind.config.ts`, `postcss.config.js`, and three `@tailwind` directives in `src/index.css`. The JavaScript config owns semantic utility aliases, fonts, radii, keyframes, and `tailwindcss-animate`; `index.css` owns dark, light, high-contrast, and legacy HSL-channel values. Live components use both the semantic utilities and animation classes, so deleting the configuration without porting those contracts would remove styles silently.

The approved target is Tailwind 4.3.3 with the Vite plugin and a CSS-first `src/shared/theme/tokens.css`. C-05 owns the later conversion of `hsl(var())` call sites, so C-02 must expose the new complete-color vocabulary without breaking legacy aliases.

## Goals / Non-Goals

**Goals:**

- Make Tailwind 4.3.3 and `@tailwindcss/vite` 4.3.3 the frontend build path.
- Establish `tokens.css` as the single utility-generating design-token source.
- Preserve dark, light, high-contrast, semantic utility, animation, font, radius, and source-scanning behavior.
- Add the new KnowMe complete-color and run-phase token vocabulary for downstream changes.
- Remove all live references to the deleted Tailwind and PostCSS config files.

**Non-Goals:**

- Rewrite the 337 legacy HSL-channel call sites.
- Purge existing Flat 2.0 violations or remove the admin terminal theme.
- Change React components, state layering, provider compatibility, realtime behavior, or persistence.
- Rework root-workspace Tailwind dependencies used outside the frontend package.

## Decisions

### Use the official Vite integration with exact phase pins

`vite.config.ts` will register `tailwindcss()` from `@tailwindcss/vite`, and the frontend package will pin both Tailwind packages to `4.3.3`. The npm registry reports `4.3.3` as current for both packages on 2026-08-07, matching the approved plan.

The repository maintains both root and frontend workspaces and lockfiles. C-02 pins Vite 8.1.4 in both workspace override maps, within every existing Vite 8 range, so the frontend, sibling A2UI packages, Storybook, and Vitest resolve one peer graph from either workspace root. The tracked TypeScript-generated `vite.config.js` is regenerated and verified alongside `vite.config.ts` because Vite resolves that artifact first.

Alternative considered: use the Tailwind PostCSS adapter. Rejected because the approved architecture explicitly deletes the PostCSS config and Tailwind documents the Vite plugin as its seamless Vite integration.

### Keep new canonical tokens and temporary compatibility aliases together

`tokens.css` will import Tailwind, declare fonts/radii/motion/run-phase tokens, and define the complete-color KnowMe ladder. It will also carry `@theme inline` aliases matching currently used utilities such as `bg-background`, `text-foreground`, and `bg-popover`. Legacy HSL channel variables remain temporarily because C-05 owns their call-site conversion; the new `--color-bg`, `--color-bg-2`, `--color-surface`, and related complete-color roles become the forward contract.

Alternative considered: convert all consumers in C-02. Rejected because the phase deliberately isolates the 30 non-admin call sites in C-05 and the 307 admin occurrences in C-14a.

### Preserve live animation utilities through the CSS-first plugin

The deleted JS config's `tailwindcss-animate` plugin currently supplies classes used by Base UI wrappers. C-02 will replace it with the Tailwind 4-compatible `tw-animate-css` import recommended by current shadcn documentation, while retaining project-specific accordion and shimmer keyframes in `@theme`.

Alternative considered: drop animation support until later component work. Rejected because live source uses those utilities and C-02 is a plumbing change that must preserve behavior.

### Preserve explicit non-default source coverage and tool references

The CSS source will retain explicit coverage for `frontend/packages/a2ui-uar/src`. Storybook path filters will point at `tokens.css`, and `components.json` will set `tailwind.config` to the documented empty value for Tailwind 4.

Alternative considered: rely on automatic detection and leave peripheral config unchanged. Rejected because the former JS content list and the execution handoff identify these references as live contracts whose silent loss would create coverage and CLI gaps.

## Risks / Trade-offs

- **Risk:** Tailwind 4 utility generation differs from the Tailwind 3 config. → **Mitigation:** Port every semantic alias and live animation contract, then run targeted token/config assertions plus frontend typecheck and lint.
- **Risk:** Moving theme values changes theme precedence. → **Mitigation:** Preserve selector order for root, light, high-contrast, and system-light overrides.
- **Risk:** Dirty package and lockfiles contain unrelated user work. → **Mitigation:** Use package-manager dependency operations and review scoped diffs; never replace the files wholesale.
- **Trade-off:** Compatibility aliases temporarily preserve two vocabularies. → This is intentional staging; C-05 and C-14a remove the old consumer vocabulary.

## Migration Plan

1. Update dependencies and converge both maintained lockfiles without discarding existing changes.
2. Register the Tailwind Vite plugin.
3. Create `tokens.css`, move theme ownership into it, and import it from `index.css`.
4. Remove the legacy configs and repair Storybook and shadcn references.
5. Verify exact pins, token/utility invariants, zero dangling references, typecheck, lint, and strict OpenSpec validity.

Rollback restores the two configuration files, Tailwind 3 dependency entries, the previous CSS directives, and the old Vite plugin list. No runtime data migration exists.

## Open Questions

None. The phase plan and official Tailwind/shadcn documentation resolve the integration and config formats.
