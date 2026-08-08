import { Settings2 } from "lucide-react";
import { Link } from "react-router";

import { useUiState } from "@/hooks/use-ui-state";
import { cn } from "@/lib/utils";

import {
  COMPACT_DESTINATIONS,
  isConfigurePath,
  isDestinationActive,
} from "./nav-destinations";

const targetClass =
  "flex min-h-11 flex-1 flex-col items-center justify-center gap-1 rounded-xl px-1 text-[11px] font-semibold transition-colors duration-200 focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember";

export function MobileTabBar({ pathname }: { pathname: string }) {
  const { shellSheet, setShellSheet } = useUiState();
  const configureActive = isConfigurePath(pathname);

  return (
    <nav
      aria-label="Compact navigation"
      className="flex shrink-0 gap-1 bg-chrome p-2 pb-[calc(.5rem+env(safe-area-inset-bottom))] min-[901px]:hidden"
    >
      {COMPACT_DESTINATIONS.map((destination) => {
        const active = isDestinationActive(destination, pathname);
        const Icon = destination.icon;
        return (
          <Link
            key={destination.id}
            to={destination.path}
            aria-current={active ? "page" : undefined}
            className={cn(
              targetClass,
              active ? "bg-ember-soft text-ember" : "text-fg-sub hover:bg-card-hov hover:text-foreground",
            )}
          >
            <Icon aria-hidden="true" className="size-5" />
            <span>{destination.label}</span>
          </Link>
        );
      })}
      <button
        type="button"
        onClick={() => setShellSheet("configure")}
        aria-haspopup="dialog"
        aria-expanded={shellSheet === "configure"}
        aria-current={configureActive ? "true" : undefined}
        className={cn(
          targetClass,
          configureActive ? "bg-ember-soft text-ember" : "text-fg-sub hover:bg-card-hov hover:text-foreground",
        )}
      >
        <Settings2 aria-hidden="true" className="size-5" />
        <span>Configure</span>
      </button>
    </nav>
  );
}
