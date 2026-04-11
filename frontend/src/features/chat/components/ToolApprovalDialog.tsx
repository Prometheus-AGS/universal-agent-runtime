import { type FC, useCallback, useEffect, useRef, useState } from "react";
import { useToolApprovalActions } from "@/hooks/use-tool-approval";
import { AlertTriangle, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

interface ToolApprovalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  runId: string;
  toolName: string;
  args: Record<string, unknown>;
  riskReason?: string;
}

const TIMEOUT_SECONDS = 300; // 5 minutes

export const ToolApprovalDialog: FC<ToolApprovalDialogProps> = ({
  open,
  onOpenChange,
  runId,
  toolName,
  args,
  riskReason,
}) => {
  const [remaining, setRemaining] = useState(TIMEOUT_SECONDS);
  const [submitting, setSubmitting] = useState(false);
  const { submitApproval } = useToolApprovalActions();

  const respond = useCallback(
    async (approved: boolean) => {
      setSubmitting(true);
      try {
        await submitApproval(runId, approved);
      } catch {
        // best-effort
      } finally {
        setSubmitting(false);
        onOpenChange(false);
      }
    },
    [runId, onOpenChange, submitApproval],
  );

  const respondRef = useRef(respond);
  respondRef.current = respond;

  // Countdown timer (use ref so the interval does not close over a stale `respond` / skip deps)
  useEffect(() => {
    if (!open) {
      setRemaining(TIMEOUT_SECONDS);
      return;
    }
    const interval = setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          void respondRef.current(false);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [open]);

  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;
  const timerLabel = `${minutes}:${seconds.toString().padStart(2, "0")}`;
  const argsJson = JSON.stringify(args, null, 2);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle size={16} className="text-warning" />
            Tool Approval Required
          </DialogTitle>
          <DialogDescription>
            The agent wants to execute a tool that requires your approval.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div>
            <Label className="mb-1 block font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              Tool
            </Label>
            <p className="font-mono text-xs font-medium text-primary">
              {toolName}
            </p>
          </div>

          {riskReason && (
            <div>
              <Label className="mb-1 block font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
                Risk Reason
              </Label>
              <p className="text-sm text-warning">{riskReason}</p>
            </div>
          )}

          <div>
            <Label className="mb-1 block font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              Arguments
            </Label>
            <ScrollArea className="max-h-48 rounded-md border border-border">
              <pre className="hljs p-3 text-[11px]">
                <code>{argsJson}</code>
              </pre>
            </ScrollArea>
          </div>

          <div className="flex items-center justify-center">
            <span className="font-mono text-xs text-muted-foreground">
              Auto-reject in{" "}
              <span className="font-semibold text-foreground">{timerLabel}</span>
            </span>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => void respond(false)}
            disabled={submitting}
            className="gap-1.5"
          >
            <X size={14} />
            Reject
          </Button>
          <Button
            type="button"
            onClick={() => void respond(true)}
            disabled={submitting}
            className="gap-1.5"
          >
            <Check size={14} />
            Approve
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
