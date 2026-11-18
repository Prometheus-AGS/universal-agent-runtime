import {
  AlertTriangle,
  ArrowRightLeft,
  Ban,
  CheckCircle2,
  Clock,
  HelpCircle,
  Hourglass,
  Loader2,
  OctagonX,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { VIEW_PRESENTATION, type AttemptView, type Tone } from "./run-model";

const TONE_CLASS: Record<Tone, string> = {
  danger: "text-destructive",
  warning: "text-warning",
  success: "text-success",
  progress: "text-foreground",
  neutral: "text-muted-foreground",
};

function ViewIcon({ view, className }: { view: AttemptView; className: string }) {
  switch (view) {
    case "queued":
      return <Clock aria-hidden="true" className={className} />;
    case "running":
      return <Loader2 aria-hidden="true" className={cn(className, "motion-safe:animate-spin")} />;
    case "cancelling":
      return <Ban aria-hidden="true" className={className} />;
    case "waiting":
      return <Hourglass aria-hidden="true" className={className} />;
    case "handed_off":
      return <ArrowRightLeft aria-hidden="true" className={className} />;
    case "cancelled_waiting":
      return <Ban aria-hidden="true" className={className} />;
    case "wait_invalidated":
      return <AlertTriangle aria-hidden="true" className={className} />;
    case "yielded":
      return <HelpCircle aria-hidden="true" className={className} />;
    case "succeeded":
      return <CheckCircle2 aria-hidden="true" className={className} />;
    case "failed":
      return <OctagonX aria-hidden="true" className={className} />;
    case "cancelled":
      return <Ban aria-hidden="true" className={className} />;
    case "uncertain":
      return <HelpCircle aria-hidden="true" className={className} />;
    case "unknown":
      return <AlertTriangle aria-hidden="true" className={className} />;
  }
}

/** Icon and text, never colour alone. `raw` names the runtime status when the view is not recognised. */
export function StatusLabel({ view, raw, className }: { view: AttemptView; raw?: string; className?: string }) {
  const presentation = VIEW_PRESENTATION[view];
  const text = view === "unknown" && raw ? `${presentation.label}: ${raw}` : presentation.label;
  return (
    <span className={cn("inline-flex items-center gap-1.5 text-sm font-medium", TONE_CLASS[presentation.tone], className)}>
      <ViewIcon view={view} className="size-4 shrink-0" />
      {text}
    </span>
  );
}
