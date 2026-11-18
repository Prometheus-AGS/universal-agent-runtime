import { useId, useState, type ReactNode } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

interface DisclosureProps {
  /** Visible label of the toggle. */
  label: string;
  /** A count shown after the label, for example the number of earlier attempts. */
  count?: number;
  children: ReactNode;
  className?: string;
}

/**
 * A collapsed-by-default section. The content mounts only while open, so a closed disclosure holds no
 * subscriptions. Its open state lives with the component, so it survives every refresh that keeps the
 * component mounted (all keys in the run view are stable ids).
 */
export function Disclosure({ label, count, children, className }: DisclosureProps) {
  const [open, setOpen] = useState(false);
  const regionId = useId();
  const Chevron = open ? ChevronDown : ChevronRight;
  return (
    <div className={cn("min-w-0", className)}>
      <button
        type="button"
        aria-expanded={open}
        aria-controls={regionId}
        onClick={() => setOpen((current) => !current)}
        className="flex min-h-11 w-full items-center gap-1.5 rounded-md px-1 text-left text-sm font-medium hover:bg-muted focus-visible:outline-2 focus-visible:outline-ring"
      >
        <Chevron aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
        <span className="min-w-0 break-words">{label}</span>
        {count !== undefined && <span className="text-xs font-normal text-muted-foreground">({count})</span>}
      </button>
      <div id={regionId} hidden={!open} className="space-y-2 pt-1 pl-5">
        {open ? children : null}
      </div>
    </div>
  );
}
