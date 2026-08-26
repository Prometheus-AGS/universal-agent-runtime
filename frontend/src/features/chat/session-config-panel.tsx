import { useCallback, useEffect, useId, useRef, useState } from "react";
import { AlertCircle, RefreshCw } from "lucide-react";

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
  useSessionPromptCaching,
} from "@/platform/entities";
import type {
  AgentSessionConfig,
  PromptCachingSource,
  ToolApproval,
} from "@/platform/entities";

interface SessionConfigPanelProps {
  threadId: string;
  agentConfig?: AgentConfig | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

function fallbackSessionConfig(
  agentId: string | undefined,
): AgentSessionConfig {
  return {
    agent_id: agentId ?? "default-agent",
    model: null,
    tools: null,
    skills: null,
    knowledge_bases: null,
    mcp_servers: null,
    tool_approval: null,
    prompt_caching_enabled: null,
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

const PROMPT_CACHING_SOURCE_LABELS: Record<PromptCachingSource, string> = {
  request: "request override",
  session: "session override",
  user: "user override",
  global: "global default",
};

const PROMPT_CACHING_OPTIONS = [
  { value: "inherit", label: "Inherit" },
  { value: "on", label: "On" },
  { value: "off", label: "Off" },
];

function PromptCachingOverrideControl({
  draftId,
  sessionId,
  disabled,
  unavailable,
  onRetry,
}: DraftControlProps & {
  sessionId: string;
  unavailable: boolean;
  onRetry: () => void;
}) {
  const promptCaching = useAgentSessionDraftField(
    draftId,
    "prompt_caching_enabled",
  );
  const effective = useSessionPromptCaching(sessionId);
  const actions = useAgentSessionDraftActions();
  const selectId = useId();
  const descriptionId = useId();
  const statusId = useId();

  if (unavailable || !effective) {
    return (
      <div
        role="alert"
        className="flex items-start justify-between gap-3 rounded-lg bg-destructive/10 p-3"
      >
        <div className="flex items-start gap-2">
          <AlertCircle size={14} className="mt-0.5 shrink-0 text-destructive" />
          <div>
            <p className="font-mono text-xs font-medium text-destructive">
              Effective prompt-caching status is unavailable.
            </p>
            <p className="font-body text-xs text-destructive">
              Retry before changing this session override.
            </p>
          </div>
        </div>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          onClick={onRetry}
          className="shrink-0 gap-1.5 font-mono text-xs"
        >
          <RefreshCw size={12} /> Retry
        </Button>
      </div>
    );
  }

  const inheritedEnabled = effective.user_override ?? effective.global_default;
  const inheritedSource =
    effective.user_override === null ? "global default" : "user override";

  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor={selectId} className="font-mono text-xs">
        Prompt Caching
      </Label>
      <Select
        items={PROMPT_CACHING_OPTIONS}
        value={
          promptCaching === null || promptCaching === undefined
            ? "inherit"
            : promptCaching
              ? "on"
              : "off"
        }
        disabled={disabled}
        onValueChange={(value) => {
          if (value === null) return;
          actions.setField(
            draftId,
            "prompt_caching_enabled",
            value === "inherit" ? null : value === "on",
          );
        }}
      >
        <SelectTrigger
          id={selectId}
          aria-describedby={`${descriptionId} ${statusId}`}
          className="w-40 font-mono text-xs"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="inherit">Inherit</SelectItem>
          <SelectItem value="on">On</SelectItem>
          <SelectItem value="off">Off</SelectItem>
        </SelectContent>
      </Select>
      <p id={descriptionId} className="font-body text-xs text-muted-foreground">
        Applies to every request in this session. Inherit follows the user
        override, then the global default.
      </p>
      <p
        id={statusId}
        role="status"
        aria-live="polite"
        className="font-mono text-xs text-muted-foreground"
      >
        Effective now: {effective.enabled ? "On" : "Off"} from{" "}
        {PROMPT_CACHING_SOURCE_LABELS[effective.source]}. Inherited value:{" "}
        {inheritedEnabled ? "On" : "Off"} from {inheritedSource}.
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
  const effectivePromptCaching = useSessionPromptCaching(threadId);
  const [retryNonce, setRetryNonce] = useState(0);
  const [effectiveLoading, setEffectiveLoading] = useState(false);
  const [effectiveUnavailable, setEffectiveUnavailable] = useState(false);
  const draftOpened = useRef(false);

  useEffect(() => {
    if (!open) return;
    const controller = new AbortController();
    const fallback = fallbackSessionConfig(agentConfig?.agent_id);
    setEffectiveLoading(true);
    setEffectiveUnavailable(false);
    void actions
      .loadAndOpen(threadId, editorId, fallback, controller.signal)
      .then(() => {
        if (controller.signal.aborted) return;
        draftOpened.current = true;
      })
      .catch((loadError: unknown) => {
        if (controller.signal.aborted || isAbortError(loadError)) return;
        setEffectiveUnavailable(true);
        if (!draftOpened.current) {
          actions.open(threadId, editorId, fallback);
          draftOpened.current = true;
        }
        actions.markError(draftId, (loadError as Error).message);
      })
      .finally(() => {
        if (!controller.signal.aborted) setEffectiveLoading(false);
      });
    return () => {
      controller.abort();
    };
  }, [
    actions,
    agentConfig?.agent_id,
    draftId,
    editorId,
    open,
    retryNonce,
    threadId,
  ]);

  useEffect(
    () => () => {
      actions.cancel(draftId);
      draftOpened.current = false;
    },
    [actions, draftId],
  );

  const handleOpenChange = useCallback(
    (nextOpen: boolean) => {
      if (!nextOpen) {
        actions.cancel(draftId);
        draftOpened.current = false;
      }
      onOpenChange(nextOpen);
    },
    [actions, draftId, onOpenChange],
  );

  const handleSave = useCallback(async () => {
    if (await actions.save(draftId)) onOpenChange(false);
  }, [actions, draftId, onOpenChange]);

  const saving = saveStatus === "saving";
  const unavailable =
    saveStatus === null ||
    effectivePromptCaching === null ||
    effectiveLoading ||
    effectiveUnavailable;

  return (
    <Sheet open={open} onOpenChange={handleOpenChange}>
      <SheetContent
        side="right"
        className="w-full overflow-y-auto sm:max-w-[400px]"
      >
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
            disabled={saving || unavailable}
            defaultLabel={
              agentConfig?.model
                ? `Agent: ${agentConfig.model}`
                : "Agent default"
            }
          />

          <Separator />

          <ToolApprovalControl
            draftId={draftId}
            disabled={saving || unavailable}
          />

          <Separator />

          <PromptCachingOverrideControl
            draftId={draftId}
            sessionId={threadId}
            disabled={saving || unavailable}
            unavailable={
              effectiveUnavailable || effectivePromptCaching === null
            }
            onRetry={() => setRetryNonce((value) => value + 1)}
          />

          <Separator />

          <Button
            onClick={handleSave}
            disabled={saving || unavailable}
            className="w-full font-mono text-xs"
          >
            {saving ? "Saving..." : "Save Configuration"}
          </Button>
          {error && (
            <p
              role="alert"
              aria-live="assertive"
              className="font-mono text-xs text-destructive"
            >
              {error}
            </p>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
