import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { SheetFooter } from "@/components/ui/sheet";

interface SheetActionsProps {
  /** Why the primary action is unavailable, or null. Rendered above the button row so it never wraps the buttons. */
  reason: string | null;
  /** The id the reason is rendered with, so a gated primary button can be described by it. */
  reasonId: string;
  primary: ReactNode;
  secondary?: ReactNode;
  onCancel: () => void;
  cancelDisabled?: boolean;
}

/** One footer for every Teams sheet: the reason, then primary action, secondary action, Cancel. */
export function SheetActions({ reason, reasonId, primary, secondary, onCancel, cancelDisabled = false }: SheetActionsProps) {
  return (
    <SheetFooter className="gap-3">
      {reason !== null && <p id={reasonId} className="max-w-prose break-words text-xs text-muted-foreground">{reason}</p>}
      <div className="flex flex-row flex-wrap items-center gap-2">
        {primary}
        {secondary}
        <Button type="button" variant="ghost" className="min-h-11" disabled={cancelDisabled} onClick={onCancel}>Cancel</Button>
      </div>
    </SheetFooter>
  );
}
