// frontend/src/lib/theme-boot.ts
import { useThemeStore } from "@/stores/theme-store";

/**
 * Apply the user's stored theme at boot.
 *
 * `index.html` ships `class="dark"` so the first paint is never unstyled.
 * Without this call the theme store module is never evaluated, so a stored
 * `light` / `high-contrast` preference is never applied and the hardcoded
 * `dark` class always wins — light theme becomes unreachable (originally
 * caught by the `uar-ui-verify-gates` screenshot matrix).
 *
 * This lives in `lib/` rather than as a bare side-effect import in
 * `main.tsx` because the frontend boundary gate forbids components
 * (including the entry module) from importing stores directly. Boot-time
 * wiring belongs in a module, alongside `entities/bootstrap.ts`.
 *
 * Reading the store's initial state is enough: `theme-store` applies the
 * resolved theme to `document.documentElement` when the store is first
 * created, so evaluating it here performs the DOM update.
 *
 * @returns The theme that was applied.
 *
 * @example
 * ```ts
 * // main.tsx, before rendering
 * bootstrapTheme();
 * ```
 */
export function bootstrapTheme() {
  return useThemeStore.getState().theme;
}
