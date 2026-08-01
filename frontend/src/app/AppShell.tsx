import { Link, useLocation } from "react-router-dom";
import type { ReactNode } from "react";
import { KnowMeLogo, KnowMeWordmark } from "@/components/KnowMeLogo";
import { ThemeToggle } from "@/components/ThemeToggle";
import { useHealthz } from "@/hooks/use-healthz";
import { cn } from "@/lib/utils";
import { DESTINATIONS, isDestinationActive } from "./navigation";

function useIsActive(path: string): boolean {
  const { pathname } = useLocation();
  return isDestinationActive(path, pathname);
}

function RailDestination({ path, label, icon: Icon }: (typeof DESTINATIONS)[number]) {
  const active = useIsActive(path);
  return (
    <Link
      to={path}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex h-12 items-center gap-3 rounded-xl px-4 text-sm font-semibold transition-colors",
        active
          ? "bg-ember-soft text-ember"
          : "text-fg-sub hover:bg-card-hov hover:text-foreground",
      )}
    >
      <Icon aria-hidden width={18} height={18} />
      <span>{label}</span>
    </Link>
  );
}

function BottomBarDestination({ path, label, icon: Icon }: (typeof DESTINATIONS)[number]) {
  const active = useIsActive(path);
  return (
    <Link
      to={path}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex min-h-14 flex-1 flex-col items-center justify-center gap-1 rounded-xl px-1 text-[11px] font-semibold transition-colors",
        active
          ? "bg-ember-soft text-ember"
          : "text-fg-sub hover:bg-card-hov",
      )}
    >
      <Icon aria-hidden width={20} height={20} />
      <span>{label}</span>
    </Link>
  );
}

/** Readiness lane — status dot + text (never color-alone) and deployment mode. */
function ReadinessCard() {
  const { health } = useHealthz();
  const statusText = health
    ? (health.status === "ok" || health.status === "healthy" ? "Ready" : `Status: ${health.status ?? "unknown"}`)
    : "Unreachable";
  const ok = health != null;
  const host = window.location.hostname;
  const embedded = host === "localhost" || host === "127.0.0.1" || host === "[::1]" || host === "";
  const versionSuffix = health?.version ? ` · v${health.version}` : "";

  return (
    <div className="m-4 mt-auto rounded-xl bg-surface p-4 text-xs text-fg-sub">
      <div className="flex items-center gap-2 font-semibold text-foreground">
        <span className={cn("size-2 rounded-full", ok ? "bg-success" : "bg-destructive")} aria-hidden />
        {statusText}
      </div>
      <p className="mt-1">
        {embedded ? "Embedded · local" : `Remote · ${host}`}{versionSuffix}
      </p>
    </div>
  );
}

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="flex min-h-0 w-full flex-1">
        {/* Wide: left rail */}
        <nav aria-label="Main (rail)" className="hidden w-60 shrink-0 flex-col bg-chrome md:flex">
          <div className="flex items-center gap-3 px-5 pb-2 pt-6 text-foreground">
            <KnowMeLogo size={28} />
            <KnowMeWordmark className="text-xl" />
            <div className="ml-auto">
              <ThemeToggle />
            </div>
          </div>
          <p className="pb-4 pl-12 font-mono text-[10px] uppercase tracking-widest text-fg-faint">
            Universal Agent Runtime
          </p>
          <div className="flex flex-col gap-1 px-3">
            {DESTINATIONS.map((destination) => (
              <RailDestination key={destination.path} {...destination} />
            ))}
          </div>
          <ReadinessCard />
        </nav>

        <main className="min-h-0 min-w-0 flex-1 overflow-hidden bg-background">{children}</main>
      </div>

      {/* Narrow: bottom bar */}
      <nav
        aria-label="Main (bottom bar)"
        className="flex shrink-0 gap-1 bg-chrome p-2 pb-[calc(.5rem+env(safe-area-inset-bottom))] md:hidden"
      >
        {DESTINATIONS.map((destination) => (
          <BottomBarDestination key={destination.path} {...destination} />
        ))}
      </nav>
    </div>
  );
}
