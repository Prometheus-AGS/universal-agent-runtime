import { type FC, useCallback, useEffect, useState } from "react";
import { AlertTriangle, Check, X } from "lucide-react";
import { Button } from "@/components/ui/button";
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

  // Countdown timer
  useEffect(() => {
    if (!open) {
      setRemaining(TIMEOUT_SECONDS);
      return;
    }
    const interval = setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          void respond(false);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const respond = useCallback(
    async (approved: boolean) => {
      setSubmitting(true);
      try {
        await fetch(`/api/uar/runs/${runId}/tool-approval`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ approved }),
        });
      } catch {
        // best-effort
      } finally {
        setSubmitting(false);
        onOpenChange(false);
      }
    },
    [runId, onOpenChange],
  );

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
            <p className="mb-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              Tool
            </p>
            <p className="font-mono text-[12px] font-medium text-primary">
              {toolName}
            </p>
          </div>

          {riskReason && (
            <div>
              <p className="mb-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
                Risk Reason
              </p>
              <p className="text-[13px] text-warning">{riskReason}</p>
            </div>
          )}

          <div>
            <p className="mb-1 font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
              Arguments
            </p>
            <pre className="hljs max-h-48 overflow-auto rounded-md p-3 text-[11px]">
              <code>{argsJson}</code>
            </pre>
          </div>

          <div className="flex items-center justify-center">
            <span className="font-mono text-[12px] text-muted-foreground">
              Auto-reject in{" "}
              <span className="font-semibold text-foreground">{timerLabel}</span>
            </span>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={() => void respond(false)}
            disabled={submitting}
            className="gap-1.5"
          >
            <X size={14} />
            Reject
          </Button>
          <Button
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
