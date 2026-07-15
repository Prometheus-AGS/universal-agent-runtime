import { create } from "zustand";

export type Theme = "light" | "dark" | "high-contrast" | "system";

interface ThemeState {
  theme: Theme;
}

interface ThemeActions {
  setTheme: (theme: Theme) => void;
}

type ThemeStore = ThemeState & ThemeActions;

const STORAGE_KEY = "uar-theme";

function getStoredTheme(): Theme {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "high-contrast" || stored === "system") {
      return stored;
    }
  } catch {
    // localStorage unavailable
  }
  return "dark";
}

function getSystemPreference(): "light" | "dark" {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

export function resolveTheme(theme: Theme): "light" | "dark" | "high-contrast" {
  return theme === "system" ? getSystemPreference() : theme;
}

function applyTheme(theme: Theme) {
  const resolved = resolveTheme(theme);
  const root = document.documentElement;
  root.classList.remove("light", "dark", "high-contrast");
  root.classList.add(resolved);
}

export const useThemeStore = create<ThemeStore>((set) => {
  const initial = getStoredTheme();

  // Apply on init
  if (typeof document !== "undefined") {
    applyTheme(initial);
  }

  // Listen for system preference changes
  if (typeof window !== "undefined") {
    window
      .matchMedia("(prefers-color-scheme: light)")
      .addEventListener("change", () => {
        const { theme } = useThemeStore.getState();
        if (theme === "system") {
          applyTheme("system");
        }
      });
  }

  return {
    theme: initial,
    setTheme: (theme) => {
      try {
        localStorage.setItem(STORAGE_KEY, theme);
      } catch {
        // localStorage unavailable
      }
      applyTheme(theme);
      set({ theme });
    },
  };
});
