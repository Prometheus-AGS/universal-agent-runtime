import type { ComponentPropsWithRef, ReactNode } from "react";
import { cn } from "../lib/cn";

export type StatusTone = "neutral" | "ok" | "warn" | "err" | "live" | "accent";

const TONE_VAR: Record<StatusTone, string> = {
  neutral: "var(--color-fg-sub)",
  ok: "var(--color-green)",
  warn: "var(--color-amber)",
  err: "var(--color-red)",
  live: "var(--color-cyan)",
  accent: "var(--color-ember)",
};

export interface StatusChipProps extends Omit<ComponentPropsWithRef<"span">, "children"> {
  /** Semantic state. Colour is never the only signal: `children` must name the state. */
  tone?: StatusTone;
  /** The visible state label, e.g. "Connected" or "Reconnecting · attempt 3". */
  children: ReactNode;
  /** Hide the leading dot when the chip sits next to another status indicator. */
  dot?: boolean;
}

/**
 * A small filled status label (Flat 2.0: tinted fill, no outline).
 *
 * The text colour mixes the tone with the theme foreground so the label keeps
 * at least 4.5:1 contrast in dark, light and high-contrast themes; the tinted
 * fill and the dot carry the hue.
 */
export function StatusChip({ tone = "neutral", dot = true, className, style, children, ...rest }: StatusChipProps) {
  const toneColor = TONE_VAR[tone];
  return (
    <span
      data-tone={tone}
      className={cn(
        "inline-flex h-6 shrink-0 items-center gap-1.5 rounded-full px-2.5 text-xs font-medium whitespace-nowrap",
        className,
      )}
      style={{
        backgroundColor: `color-mix(in oklch, ${toneColor} 16%, transparent)`,
        color: `color-mix(in oklch, ${toneColor} 72%, var(--color-fg))`,
        ...style,
      }}
      {...rest}
    >
      {dot ? (
        <span aria-hidden="true" className="size-1.5 rounded-full" style={{ backgroundColor: toneColor }} />
      ) : null}
      {children}
    </span>
  );
}
