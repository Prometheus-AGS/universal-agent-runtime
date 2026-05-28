import type { FC } from "react";
import { cn } from "@/lib/utils";

/**
 * Top-pinned error bar with a monospaced error-code prefix.
 *
 * Replaces inconsistent error banners across admin pages. See
 * `docs/admin-aesthetic-spec.md` §4 for the contract.
 *
 * `code` becomes the `ERR-<CODE>` prefix; conventionally an uppercase
 * short token like `ERR-MODELS`, `ERR-PROVIDERS`, etc.
 */
export const ErrorBar: FC<{
  code: string;
  message: string;
  className?: string;
  onDismiss?: () => void;
}> = ({ code, message, className, onDismiss }) => {
  const prefix = code.startsWith("ERR-") ? code : `ERR-${code.toUpperCase()}`;
  return (
    <div
      role="alert"
      className={cn(
        "flex items-center gap-3 border-b border-[hsl(var(--signal-red))] bg-[hsl(var(--signal-red)/0.08)] px-6 py-2 text-xs text-[hsl(var(--signal-red))]",
        className,
      )}
    >
      <span className="font-semibold tracking-tight">{prefix}</span>
      <span className="flex-1 truncate">{message}</span>
      {onDismiss && (
        <button
          type="button"
          onClick={onDismiss}
          className="rounded border border-[hsl(var(--signal-red)/0.4)] px-2 py-0.5 text-xs hover:bg-[hsl(var(--signal-red)/0.15)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[hsl(var(--phosphor-glow))]"
        >
          dismiss
        </button>
      )}
    </div>
  );
};
