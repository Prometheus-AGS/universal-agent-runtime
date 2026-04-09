import { type FC } from "react";
import { cn } from "@/lib/utils";

interface ContextUsageBarProps {
  used: number;
  total: number;
  strategy?: string;
}

export const ContextUsageBar: FC<ContextUsageBarProps> = ({
  used,
  total,
  strategy,
}) => {
  const pct = total > 0 ? Math.min((used / total) * 100, 100) : 0;

  const colorClass =
    pct > 85
      ? "bg-destructive"
      : pct > 70
        ? "bg-warning"
        : "bg-success";

  const textColorClass =
    pct > 85
      ? "text-destructive"
      : pct > 70
        ? "text-warning"
        : "text-success";

  const formatTokens = (n: number): string => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
    return String(n);
  };

  return (
    <div className="flex items-center gap-2">
      <div className="relative h-1.5 w-24 overflow-hidden rounded-full bg-muted">
        <div
          className={cn("h-full rounded-full transition-all duration-300", colorClass)}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className={cn("font-mono text-[10px]", textColorClass)}>
        {formatTokens(used)}/{formatTokens(total)}
      </span>
      {strategy && (
        <span className="font-mono text-[9px] text-muted-foreground/60">
          {strategy}
        </span>
      )}
    </div>
  );
};
