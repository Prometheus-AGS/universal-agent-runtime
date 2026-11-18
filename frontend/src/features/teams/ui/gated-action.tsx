import { useId, type ComponentProps, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface GatedActionProps extends Omit<ComponentProps<typeof Button>, "disabled" | "aria-disabled"> {
  /** Why the action is unavailable, or null when it is allowed. Shown as visible text beside the control. */
  reason: string | null;
  /**
   * Set when the caller renders the reason itself (for example above a sheet footer row): the button is then
   * described by the element with this id and no reason text is rendered here.
   */
  externalReasonId?: string;
  children: ReactNode;
}

/**
 * A button that stays focusable while unavailable (aria-disabled), with the reason as adjacent visible text
 * that the button is described by. Disabled buttons drop out of the tab order and hide why; this one does not.
 */
export function GatedAction({ reason, externalReasonId, onClick, className, children, ...rest }: GatedActionProps) {
  const ownReasonId = useId();
  const reasonId = externalReasonId ?? ownReasonId;
  const blocked = reason !== null;
  const handleClick: NonNullable<ComponentProps<typeof Button>["onClick"]> = (event) => {
    if (blocked) {
      event.preventDefault();
      return;
    }
    onClick?.(event);
  };
  return (
    <div className="flex min-w-0 flex-col items-start gap-1.5">
      <Button
        type="button"
        {...rest}
        aria-disabled={blocked || undefined}
        aria-describedby={blocked ? reasonId : undefined}
        onClick={handleClick}
        className={cn("min-h-11 aria-disabled:cursor-not-allowed aria-disabled:opacity-50", className)}
      >
        {children}
      </Button>
      {blocked && externalReasonId === undefined && (
        <p id={reasonId} className="max-w-prose break-words text-xs text-muted-foreground">
          {reason}
        </p>
      )}
    </div>
  );
}
