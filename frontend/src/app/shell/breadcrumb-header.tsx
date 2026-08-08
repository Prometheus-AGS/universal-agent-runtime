import { Search } from "lucide-react";

import { ThemeToggle } from "@/components/ThemeToggle";
import { UarLogo } from "@/shared/ui/uar-logo";

import {
  findDestinationForPath,
  type NavigationGroup,
} from "./nav-destinations";
import { ReadinessStatus } from "./readiness-status";

interface BreadcrumbHeaderProps {
  pathname: string;
  health: { status?: string; version?: string } | null;
  onOpenCommandPalette: () => void;
}

const sectionLabels: Record<NavigationGroup, string> = {
  work: "Workspace",
  configure: "Configure",
  system: "System",
};

export function BreadcrumbHeader({ pathname, health, onOpenCommandPalette }: BreadcrumbHeaderProps) {
  const destination = findDestinationForPath(pathname);
  const section = destination ? sectionLabels[destination.group] : "Application";
  const commandKey = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent)
    ? "⌘K"
    : "Ctrl K";

  return (
    <header className="flex min-h-14 shrink-0 items-center gap-3 bg-background px-3 min-[901px]:px-5">
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <UarLogo size={22} className="shrink-0 text-ember min-[901px]:hidden" />
        <nav aria-label="Breadcrumb" className="min-w-0">
          <ol className="flex min-w-0 items-center gap-2 text-sm">
            <li className="hidden text-fg-faint sm:block">{section}</li>
            <li aria-hidden="true" className="hidden text-fg-faint sm:block">/</li>
            <li aria-current="page" className="min-w-0">
              <h1 className="truncate font-semibold text-foreground">
                {destination?.label ?? "Current view"}
              </h1>
            </li>
          </ol>
        </nav>
      </div>

      <div className="hidden max-[900px]:block">
        <ReadinessStatus health={health} compact />
      </div>

      <button
        type="button"
        onClick={onOpenCommandPalette}
        className="flex h-10 items-center gap-2 rounded-xl bg-surface px-3 text-sm font-semibold text-fg-sub transition-colors duration-200 hover:bg-card-hov hover:text-foreground focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember max-[900px]:h-11"
        aria-label="Open command palette"
        aria-haspopup="dialog"
      >
        <Search aria-hidden="true" className="size-4" />
        <span className="hidden sm:inline">Command</span>
        <kbd className="hidden rounded-md bg-card px-1.5 py-0.5 font-mono text-[10px] text-fg-faint lg:inline">
          {commandKey}
        </kbd>
      </button>
      <ThemeToggle className="max-[900px]:size-11" />
    </header>
  );
}
