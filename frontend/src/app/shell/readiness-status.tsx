import { cn } from "@/lib/utils";

interface ReadinessHealth {
  status?: string;
  version?: string;
}

interface ReadinessStatusProps {
  health: ReadinessHealth | null;
  collapsed?: boolean;
  compact?: boolean;
}

export function ReadinessStatus({
  health,
  collapsed = false,
  compact = false,
}: ReadinessStatusProps) {
  const ready = health?.status === "ok" || health?.status === "healthy";
  const statusText = health
    ? ready
      ? "Ready"
      : `Status: ${health.status ?? "unknown"}`
    : "Unreachable";
  const host = window.location.hostname;
  const embedded = host === "localhost" || host === "127.0.0.1" || host === "[::1]" || host === "";
  const modeText = embedded ? "Embedded · local" : `Remote · ${host}`;
  const versionText = health?.version ? ` · v${health.version}` : "";

  if (collapsed) {
    return (
      <div
        className="mx-auto mb-4 flex size-10 items-center justify-center rounded-xl bg-surface"
        role="status"
        aria-label={`${statusText}. ${modeText}${versionText}`}
        title={`${statusText} · ${modeText}${versionText}`}
      >
        <span
          className={cn("size-2 rounded-full", ready ? "bg-success" : "bg-destructive")}
          aria-hidden="true"
        />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-xl bg-surface text-fg-sub",
        compact ? "px-2.5 py-2 text-[11px]" : "m-4 mt-auto p-4 text-xs",
      )}
    >
      <span
        className={cn("size-2 shrink-0 rounded-full", ready ? "bg-success" : "bg-destructive")}
        aria-hidden="true"
      />
      <span className="min-w-0">
        <span className="block font-semibold text-foreground">{statusText}</span>
        {!compact && <span className="mt-1 block truncate">{modeText}{versionText}</span>}
      </span>
    </div>
  );
}
