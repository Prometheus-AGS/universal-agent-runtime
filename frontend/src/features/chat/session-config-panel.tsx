import { useCallback, useEffect, useId } from "react";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import type { AgentConfig } from "@/features/chat/agent-selector";
import { ModelSelector } from "@/features/models/model-selector";
import {
  agentSessionDraftId,
  useAgentSessionDraftActions,
  useAgentSessionDraftError,
  useAgentSessionDraftField,
  useAgentSessionDraftStatus,
} from "@/platform/entities";
import type { AgentSessionConfig, ToolApproval } from "@/platform/entities";

interface SessionConfigPanelProps {
  threadId: string;
  agentConfig?: AgentConfig | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function fallbackSessionConfig(agentId: string | undefined): AgentSessionConfig {
  return {
    agent_id: agentId ?? "default-agent",
    model: null,
    tools: null,
    skills: null,
    knowledge_bases: null,
    mcp_servers: null,
    tool_approval: null,
  };
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === "AbortError";
}

interface DraftControlProps {
  draftId: string;
  disabled: boolean;
}

function ModelOverrideControl({
  draftId,
  disabled,
  defaultLabel,
}: DraftControlProps & { defaultLabel: string }) {
  const model = useAgentSessionDraftField(draftId, "model");
  const actions = useAgentSessionDraftActions();
  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor="model-override" className="font-mono text-xs">
        Model Override
      </Label>
      <ModelSelector
        value={model ?? ""}
        onChange={(value) => actions.setField(draftId, "model", value || null)}
        defaultLabel={defaultLabel}
        placeholder="Select model override..."
        disabled={disabled}
      />
      <p className="font-body text-xs text-muted-foreground">
        Leave empty to use the agent default.
      </p>
    </div>
  );
}

function ToolApprovalControl({ draftId, disabled }: DraftControlProps) {
  const toolApproval = useAgentSessionDraftField(draftId, "tool_approval");
  const actions = useAgentSessionDraftActions();
  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor="tool-approval" className="font-mono text-xs">
        Tool Approval
      </Label>
      <Select
        value={toolApproval ?? "inherit"}
        disabled={disabled}
        onValueChange={(value) => {
          if (value === null) return;
          actions.setField(
            draftId,
            "tool_approval",
            value === "inherit" ? null : (value as ToolApproval),
          );
        }}
      >
        <SelectTrigger id="tool-approval" className="w-40 font-mono text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="inherit">Agent default</SelectItem>
          <SelectItem value="auto">Auto</SelectItem>
          <SelectItem value="ask">Ask</SelectItem>
          <SelectItem value="deny">Deny</SelectItem>
        </SelectContent>
      </Select>
      <p className="font-body text-xs text-muted-foreground">
        How tool calls are approved during this session.
      </p>
    </div>
  );
}

export function SessionConfigPanel({
  threadId,
  agentConfig,
  open,
  onOpenChange,
}: SessionConfigPanelProps) {
  const editorId = useId();
  const draftId = agentSessionDraftId(threadId, editorId);
  const actions = useAgentSessionDraftActions();
  const saveStatus = useAgentSessionDraftStatus(draftId);
  const error = useAgentSessionDraftError(draftId);

  useEffect(() => {
    if (!open) return;
    const controller = new AbortController();
    const fallback = fallbackSessionConfig(agentConfig?.agent_id);
    void actions
      .loadAndOpen(threadId, editorId, fallback, controller.signal)
      .catch((loadError: unknown) => {
        if (controller.signal.aborted || isAbortError(loadError)) return;
        actions.open(threadId, editorId, fallback);
        actions.markError(draftId, (loadError as Error).message);
      });
    return () => {
      controller.abort();
      actions.cancel(draftId);
    };
  }, [actions, agentConfig?.agent_id, draftId, editorId, open, threadId]);

  const handleOpenChange = useCallback(
    (nextOpen: boolean) => {
      if (!nextOpen) actions.cancel(draftId);
      onOpenChange(nextOpen);
    },
    [actions, draftId, onOpenChange],
  );

  const handleSave = useCallback(async () => {
    if (await actions.save(draftId)) onOpenChange(false);
  }, [actions, draftId, onOpenChange]);

  const saving = saveStatus === "saving";
  const unavailable = saveStatus === null || saveStatus === "error";

  return (
    <Sheet open={open} onOpenChange={handleOpenChange}>
      <SheetContent side="right" className="w-full overflow-y-auto sm:max-w-[400px]">
        <SheetHeader>
          <SheetTitle className="font-display text-lg font-semibold text-foreground">
            Session Configuration
          </SheetTitle>
          <SheetDescription className="text-xs text-muted-foreground">
            Override agent defaults for this session.
          </SheetDescription>
        </SheetHeader>

        <div className="flex flex-col gap-6 px-4 pb-4">
          <ModelOverrideControl
            draftId={draftId}
            disabled={saving}
            defaultLabel={agentConfig?.model ? `Agent: ${agentConfig.model}` : "Agent default"}
          />

          <Separator />

          <ToolApprovalControl draftId={draftId} disabled={saving} />

          <Separator />

          <Button
            onClick={handleSave}
            disabled={saving || unavailable}
            className="w-full font-mono text-xs"
          >
            {saving ? "Saving..." : "Save Configuration"}
          </Button>
          {error && <p className="font-mono text-xs text-destructive">{error}</p>}
        </div>
      </SheetContent>
    </Sheet>
  );
}
