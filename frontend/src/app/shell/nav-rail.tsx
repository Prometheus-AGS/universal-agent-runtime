import { ChevronLeft, ChevronRight } from "lucide-react";
import { Link } from "react-router";

import { useUiState } from "@/hooks/use-ui-state";
import { cn } from "@/lib/utils";
import { UarAppIcon, UarWordmark } from "@/shared/ui/uar-logo";

import {
  CONFIGURE_DESTINATIONS,
  isDestinationActive,
  SYSTEM_DESTINATIONS,
  WORK_DESTINATIONS,
  type NavigationDestination,
} from "./nav-destinations";
import { ReadinessStatus } from "./readiness-status";

interface NavRailProps {
  pathname: string;
  health: { status?: string; version?: string } | null;
}

const focusClass =
  "focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember";

function RailLink({
  destination,
  pathname,
  collapsed,
}: {
  destination: NavigationDestination;
  pathname: string;
  collapsed: boolean;
}) {
  const active = isDestinationActive(destination, pathname);
  const Icon = destination.icon;

  return (
    <Link
      to={destination.path}
      aria-current={active ? "page" : undefined}
      aria-label={collapsed ? destination.label : undefined}
      title={collapsed ? destination.label : undefined}
      className={cn(
        "flex h-10 items-center rounded-xl text-sm font-semibold transition-colors duration-200",
        collapsed ? "justify-center px-2" : "gap-3 px-3",
        active
          ? "bg-ember-soft text-ember"
          : "text-fg-sub hover:bg-card-hov hover:text-foreground",
        focusClass,
      )}
    >
      <Icon aria-hidden="true" className="size-[18px] shrink-0" />
      {!collapsed && <span className="truncate">{destination.label}</span>}
    </Link>
  );
}

function RailGroup({
  label,
  destinations,
  pathname,
  collapsed,
}: {
  label: string;
  destinations: readonly NavigationDestination[];
  pathname: string;
  collapsed: boolean;
}) {
  return (
    <div className="space-y-1">
      {!collapsed && (
        <p className="px-3 pt-3 font-mono text-[10px] font-semibold uppercase tracking-[0.15em] text-fg-faint">
          {label}
        </p>
      )}
      {destinations.map((destination) => (
        <RailLink
          key={destination.id}
          destination={destination}
          pathname={pathname}
          collapsed={collapsed}
        />
      ))}
    </div>
  );
}

export function NavRail({ pathname, health }: NavRailProps) {
  const { navRailCollapsed, toggleNavRail } = useUiState();

  return (
    <nav
      aria-label="Primary navigation"
      data-collapsed={navRailCollapsed || undefined}
      className={cn(
        "hidden shrink-0 flex-col bg-chrome transition-[width] duration-200 min-[901px]:flex",
        navRailCollapsed ? "w-[60px]" : "w-60",
      )}
    >
      <div className={cn("flex min-h-20 items-center", navRailCollapsed ? "justify-center" : "px-4")}>
        {navRailCollapsed ? (
          <UarAppIcon className="size-8" />
        ) : (
          <UarWordmark className="h-12 w-full" />
        )}
      </div>

      <div className={cn("flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto", navRailCollapsed ? "px-2" : "px-3")}>
        <RailGroup label="Work" destinations={WORK_DESTINATIONS} pathname={pathname} collapsed={navRailCollapsed} />
        <RailGroup label="Configure" destinations={CONFIGURE_DESTINATIONS} pathname={pathname} collapsed={navRailCollapsed} />
        <RailGroup label="System" destinations={SYSTEM_DESTINATIONS} pathname={pathname} collapsed={navRailCollapsed} />
      </div>

      {navRailCollapsed ? (
        <ReadinessStatus health={health} collapsed />
      ) : (
        <ReadinessStatus health={health} />
      )}

      <button
        type="button"
        onClick={toggleNavRail}
        aria-label={navRailCollapsed ? "Expand navigation" : "Collapse navigation"}
        aria-expanded={!navRailCollapsed}
        className={cn(
          "m-2 flex h-10 items-center justify-center rounded-xl bg-surface text-fg-sub transition-colors duration-200 hover:bg-card-hov hover:text-foreground",
          focusClass,
        )}
      >
        {navRailCollapsed ? <ChevronRight aria-hidden="true" className="size-4" /> : <ChevronLeft aria-hidden="true" className="size-4" />}
      </button>
    </nav>
  );
}
