import type { FC, ReactNode } from "react";
import { cn } from "@/lib/utils";

/**
 * ASCII-frame empty-state component. Used everywhere a list/detail view
 * has no content yet. See `docs/admin-aesthetic-spec.md` §4.
 *
 * The frame draws box-drawing characters as a static SVG-free decoration;
 * the title goes inside the frame, the hint + optional action below.
 */
export const EmptyFrame: FC<{
  title: string;
  hint?: ReactNode;
  action?: ReactNode;
  className?: string;
}> = ({ title, hint, action, className }) => {
  // Pad/truncate the title to fit the 28-char-wide frame.
  const inner = title.length > 26 ? `${title.slice(0, 25)}…` : title;
  const padded = inner.padEnd(26, " ");

  return (
    <div className={cn("mx-auto max-w-md py-12 text-center", className)}>
      <p className="sr-only">{title}</p>
      <pre
        aria-hidden
        className="select-none text-xs leading-[1.2] text-[var(--color-fg-sub)] opacity-60"
      >
{`┌────────────────────────────┐
│                            │
│ ${padded} │
│                            │
└────────────────────────────┘`}
      </pre>
      {hint && (
        <p className="mt-3 text-xs leading-relaxed text-[var(--color-fg-sub)]">
          {hint}
        </p>
      )}
      {action && <div className="mt-4 flex justify-center">{action}</div>}
    </div>
  );
};
