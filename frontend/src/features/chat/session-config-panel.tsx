import { useState, useCallback } from "react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ModelSelector } from "@/components/model-selector";

interface SessionConfigPanelProps {
  threadId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SessionConfigPanel({
  threadId,
  open,
  onOpenChange,
}: SessionConfigPanelProps) {
  const [modelOverride, setModelOverride] = useState("");
  const [historyWindow, setHistoryWindow] = useState<number | "">(50);
  const [injectMemory, setInjectMemory] = useState(false);
  const [autoCapture, setAutoCapture] = useState(false);
  const [memoryScope, setMemoryScope] = useState("session");
  const [toolApproval, setToolApproval] = useState("auto");
  const [saving, setSaving] = useState(false);

  const handleSave = useCallback(async () => {
    setSaving(true);
    try {
      await fetch(
        `/api/uar/sessions/${encodeURIComponent(threadId)}/agent-config`,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            model_override: modelOverride || null,
            context_strategy: {
              history_window:
                historyWindow === "" ? null : Number(historyWindow),
              inject_memory: injectMemory,
              auto_capture: autoCapture,
              memory_scope: memoryScope,
            },
            tool_approval: toolApproval,
          }),
        },
      );
      onOpenChange(false);
    } catch {
      /* best-effort */
    } finally {
      setSaving(false);
    }
  }, [
    threadId,
    modelOverride,
    historyWindow,
    injectMemory,
    autoCapture,
    memoryScope,
    toolApproval,
    onOpenChange,
  ]);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-[400px] sm:max-w-[400px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="font-display text-lg font-semibold text-foreground">
            Session Configuration
          </SheetTitle>
          <SheetDescription className="text-xs text-muted-foreground">
            Override agent defaults for this session.
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 flex flex-col gap-6">
          {/* Model Override */}
          <div className="flex flex-col gap-2">
            <Label htmlFor="model-override" className="font-mono text-xs">
              Model Override
            </Label>
            <ModelSelector
              value={modelOverride}
              onChange={setModelOverride}
              defaultLabel="Agent default"
              placeholder="Select model override..."
            />
            <p className="font-body text-xs text-muted-foreground">
              Leave empty to use the agent default.
            </p>
          </div>

          <Separator />

          {/* Context Strategy */}
          <div className="flex flex-col gap-4">
            <h3 className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">Context Strategy</h3>

            <div className="flex flex-col gap-2">
              <Label htmlFor="history-window" className="font-mono text-xs">
                History Window
              </Label>
              <Input
                id="history-window"
                type="number"
                min={1}
                max={500}
                placeholder="50"
                value={historyWindow}
                onChange={(e) =>
                  setHistoryWindow(
                    e.target.value === "" ? "" : Number(e.target.value),
                  )
                }
                className="font-mono text-xs w-24"
              />
              <p className="font-body text-xs text-muted-foreground">
                Max messages to include in context.
              </p>
            </div>

            <div className="flex items-center justify-between">
              <Label htmlFor="inject-memory" className="font-mono text-xs">
                Inject Memory
              </Label>
              <Switch
                id="inject-memory"
                checked={injectMemory}
                onCheckedChange={setInjectMemory}
              />
            </div>

            <div className="flex items-center justify-between">
              <Label htmlFor="auto-capture" className="font-mono text-xs">
                Auto-capture
              </Label>
              <Switch
                id="auto-capture"
                checked={autoCapture}
                onCheckedChange={setAutoCapture}
              />
            </div>

            <div className="flex flex-col gap-2">
              <Label htmlFor="memory-scope" className="font-mono text-xs">
                Memory Scope
              </Label>
              <Select value={memoryScope} onValueChange={setMemoryScope}>
                <SelectTrigger id="memory-scope" className="font-mono text-xs w-40">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="session">Session</SelectItem>
                  <SelectItem value="agent">Agent</SelectItem>
                  <SelectItem value="global">Global</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <Separator />

          {/* Tool Approval */}
          <div className="flex flex-col gap-2">
            <Label htmlFor="tool-approval" className="font-mono text-xs">
              Tool Approval
            </Label>
            <Select value={toolApproval} onValueChange={setToolApproval}>
              <SelectTrigger id="tool-approval" className="font-mono text-xs w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">Auto</SelectItem>
                <SelectItem value="ask">Ask</SelectItem>
                <SelectItem value="deny">Deny</SelectItem>
              </SelectContent>
            </Select>
            <p className="font-body text-xs text-muted-foreground">
              How tool calls are approved during this session.
            </p>
          </div>

          <Separator />

          {/* Save */}
          <Button
            onClick={handleSave}
            disabled={saving}
            className="w-full font-mono text-xs"
          >
            {saving ? "Saving..." : "Save Configuration"}
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
