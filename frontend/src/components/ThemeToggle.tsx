import { type FC } from "react";
import { Contrast, Moon, Monitor, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { type Theme, useTheme } from "@/hooks/use-theme";

const themeOrder: Theme[] = ["dark", "light", "high-contrast", "system"];

const themeIcons = {
  light: Sun,
  dark: Moon,
  "high-contrast": Contrast,
  system: Monitor,
} as const;

const themeLabels = {
  light: "Light mode",
  dark: "Dark mode",
  "high-contrast": "High contrast mode",
  system: "System theme",
} as const;

export const ThemeToggle: FC = () => {
  const { theme, resolved, setTheme } = useTheme();
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
