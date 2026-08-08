import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

export function ChunkSurface({ children, className, label, live = false }: { children: ReactNode; className?: string; label?: string; live?: boolean }) {
  return (
    <div
      aria-label={label}
      aria-live={live ? "polite" : undefined}
      className={cn("my-2 rounded-xl bg-surface px-3 py-3 text-foreground", className)}
    >
      {children}
    </div>
  );
}

export function ChunkMeta({ children }: { children: ReactNode }) {
  return <span className="font-mono text-[10px] text-fg-faint">{children}</span>;
}

export function JsonSource({ value, label = "JSON" }: { value: unknown; label?: string }) {
  const source = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  return (
    <pre aria-label={label} className="max-h-80 overflow-auto rounded-lg bg-card px-3 py-2 font-mono text-xs text-fg-sub">
      <code>{source}</code>
    </pre>
  );
}
