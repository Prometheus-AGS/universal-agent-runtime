import { type FC } from "react";
import { Moon, Monitor, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useThemeStore, resolveTheme, type Theme } from "@/stores/theme-store";

const themeOrder: Theme[] = ["dark", "light", "system"];

const themeIcons = {
  light: Sun,
  dark: Moon,
  system: Monitor,
} as const;

const themeLabels = {
  light: "Light mode",
  dark: "Dark mode",
  system: "System theme",
} as const;

export const ThemeToggle: FC = () => {
  const { theme, setTheme } = useThemeStore();
  const resolved = resolveTheme(theme);
  const Icon = themeIcons[theme];

  const cycle = () => {
    const idx = themeOrder.indexOf(theme);
    const next = themeOrder[(idx + 1) % themeOrder.length];
    setTheme(next);
  };

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={cycle}
      aria-label={themeLabels[theme]}
      title={`${themeLabels[theme]}${theme === "system" ? ` (${resolved})` : ""}`}
      className="h-8 w-8"
    >
      <Icon size={15} />
    </Button>
  );
};
