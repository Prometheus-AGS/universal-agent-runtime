import { resolveTheme, useThemeStore } from "@/stores/theme-store";
export type { Theme } from "@/stores/theme-store";

/** Expose the current theme and mutation intent. */
export function useTheme() {
  const theme = useThemeStore((state) => state.theme);
  return { theme, resolved: resolveTheme(theme), setTheme: useThemeStore((state) => state.setTheme) };
}
