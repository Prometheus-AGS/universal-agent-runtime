import type { FC } from "react";
import { cn } from "@/lib/utils";

/**
 * Terminal loading indicator — a blinking ▍ phosphor block.
 *
 * Replaces generic spinners and skeletons on every admin page. See
 * `docs/admin-aesthetic-spec.md` §4 for the contract.
 */
export const LoadingCursor: FC<{ className?: string; label?: string }> = ({
  className,
  label,
}) => {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 text-[var(--color-fg-sub)]",
        className,
      )}
    >
      <span
        aria-hidden
        className="inline-block text-[var(--color-ember)]"
        style={{ animation: "terminal-cursor-blink 600ms steps(1, end) infinite" }}
      >
        ▍
      </span>
      {label && <span className="text-xs">{label}</span>}
    </span>
  );
};
