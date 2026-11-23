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
import { ModelPicker } from "@/features/models/pickers";
import { SessionPresentationSelection } from "@/features/presentations";
import { ClearOverridePreview, SessionEffectiveModelView } from "./session-model-view";
import {
  agentSessionDraftId,
  useAgentSessionDraftActions,
  useAgentSessionDraftError,
  useAgentSessionDraftField,
  useAgentSessionDraftStatus,
  useAgentSessionDraftUncertain,
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
  sessionId,
  disabled,
  defaultLabel,
  effectiveFailed,
  onRetryEffective,
  onSave,
}: DraftControlProps & {
  sessionId: string;
  defaultLabel: string;
  effectiveFailed: boolean;
  onRetryEffective: () => void;
  /** Saves the whole draft; true when the runtime accepted it. */
  onSave: () => Promise<boolean>;
}) {
  const model = useAgentSessionDraftField(draftId, "model");
  const actions = useAgentSessionDraftActions();
  const [previewing, setPreviewing] = useState(false);
  const [clearing, setClearing] = useState(false);

  // Clearing never happens silently: the preview names what the conversation will inherit first.
  // Cancel only closes the preview; the draft and the saved settings are untouched.
  const confirmClear = useCallback(async () => {
    setClearing(true);
    actions.setField(draftId, "model", null);
    const saved = await onSave();
    setClearing(false);
    if (!saved) setPreviewing(false);
  }, [actions, draftId, onSave]);

  return (
    <div className="flex flex-col gap-2">
      <Label htmlFor="model-override" className="font-mono text-xs">
        Model Override
      </Label>
      <ModelPicker
        id="model-override"
        testId="session-config-model-trigger"
        ariaLabel="Model override"
        value={model ?? ""}
        onChange={(value) => {
          if (value) actions.setField(draftId, "model", value);
          else if (model) setPreviewing(true);
        }}
        defaultLabel={defaultLabel}
        placeholder="Select model override..."
        disabled={disabled || previewing}
      />
      <p className="font-body text-xs text-muted-foreground">
        Clear it to follow the agent, or the system default.
      </p>
      <SessionEffectiveModelView
        sessionId={sessionId}
        failed={effectiveFailed}
        onRetry={onRetryEffective}
      />
      {previewing ? (
        <ClearOverridePreview
          sessionId={sessionId}
          failed={effectiveFailed}
          busy={clearing}
          onConfirm={() => void confirmClear()}
          onCancel={() => setPreviewing(false)}
        />
      ) : (
        model && (
          <Button
            type="button"
            variant="secondary"
            size="sm"
            data-testid="session-config-clear"
            disabled={disabled}
            onClick={() => setPreviewing(true)}
            className="self-start font-mono text-xs"
          >
            Use the inherited model
          </Button>
        )
      )}
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
  const uncertain = useAgentSessionDraftUncertain(draftId);
  const effectivePromptCaching = useSessionPromptCaching(threadId);
  const [retryNonce, setRetryNonce] = useState(0);
  const [effectiveLoading, setEffectiveLoading] = useState(false);
  const [effectiveUnavailable, setEffectiveUnavailable] = useState(false);
  const [modelViewLoading, setModelViewLoading] = useState(false);
  const [modelViewFailed, setModelViewFailed] = useState(false);
  const draftOpened = useRef(false);
  const saveErrorRef = useRef<HTMLParagraphElement>(null);

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

  // Which model answers is read separately from the draft: a failure here is shown in its own
  // block and never blocks the other settings. Cleared first so a previous value is never
  // presented as current while the runtime is being asked again.
  useEffect(() => {
    if (!open) return;
    const controller = new AbortController();
    actions.clearEffectiveModel(threadId);
    setModelViewLoading(true);
    setModelViewFailed(false);
    void actions
      .loadEffectiveModel(threadId, controller.signal)
      .catch((loadError: unknown) => {
        if (controller.signal.aborted || isAbortError(loadError)) return;
        setModelViewFailed(true);
      })
      .finally(() => {
        if (!controller.signal.aborted) setModelViewLoading(false);
      });
    return () => {
      controller.abort();
    };
  }, [actions, open, retryNonce, threadId]);

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

  const handleSave = useCallback(async (): Promise<boolean> => {
    const saved = await actions.save(draftId);
    if (saved) onOpenChange(false);
    else requestAnimationFrame(() => saveErrorRef.current?.focus());
    return saved;
  }, [actions, draftId, onOpenChange]);

  const saving = saveStatus === "saving";
  const unavailable =
    uncertain ||
    saveStatus === null ||
    effectivePromptCaching === null ||
    effectiveLoading ||
    modelViewLoading ||
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
            sessionId={threadId}
            disabled={saving || unavailable}
            defaultLabel={
              agentConfig?.model
                ? `Agent: ${agentConfig.model}`
                : "Agent default"
            }
            effectiveFailed={modelViewFailed}
            onRetryEffective={() => setRetryNonce((value) => value + 1)}
            onSave={handleSave}
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

          <SessionPresentationSelection draftId={draftId} disabled={saving || unavailable} />

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
              ref={saveErrorRef}
              tabIndex={-1}
              role="alert"
              aria-live="assertive"
              className="font-mono text-xs text-destructive"
            >
              {error}
            </p>
          )}
          {uncertain && <Button type="button" variant="secondary" className="min-h-11 w-full" disabled={saving}
            onClick={() => { void actions.reconcileSaved(draftId); }}>
            {saving ? "Checking saved configuration..." : "Check saved configuration"}
          </Button>}
        </div>
      </SheetContent>
    </Sheet>
  );
}
