import {
  ActionBarPrimitive,
  AuiIf,
  BranchPickerPrimitive,
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  type ThreadMessageLike,
  type ToolCallMessagePartProps,
  useMessage,
  useMessagePartText,
} from "@assistant-ui/react";
import {
  AlertTriangleIcon,
  ArrowDownIcon,
  ArrowUpIcon,
  BookTextIcon,
  BotIcon,
  BrainIcon,
  CheckIcon,
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ClipboardIcon,
  CopyIcon,
  Loader2Icon,
  PaperclipIcon,
  PencilIcon,
  SparklesIcon,
  SquareIcon,
  UserIcon,
  ZapIcon,
} from "lucide-react";
import { type FC, type ReactNode, useCallback, useMemo, useRef, useState } from "react";
import { EnhancedMarkdownText } from "@/components/assistant-ui/enhanced-markdown-text";
import { TooltipIconButton } from "@/components/assistant-ui/tooltip-icon-button";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Separator } from "@/components/ui/separator";
import { ContextUpdateBlock } from "@/features/chat/components/context-update-block";
import { SkillActivationBlock } from "@/features/chat/components/skill-activation-block";
import { MemoryMutationBlock, MemoryRecallBlock, MemoryUpdateBlock } from "@/features/chat/components/memory-chunk-block";
import { A2uiDisplayBlock, A2uiInputBlock } from "@/features/chat/components/a2ui-artifact-block";
import { ToolCallBlockWrapper } from "@/features/chat/components/tool-call-block";
import { AttachmentPreviewStrip } from "@/features/chat/components/attachment-preview";
import { useAttachmentContext } from "@/features/chat/attachment-context";
import { useMemoryContext } from "@/features/chat/memory-context";
import { cn } from "@/lib/utils";
import { CapabilityToggles } from "@/features/chat/capability-toggles";
import { useAgentConfig } from "@/features/chat/agent-config-context";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { useChatMessageStore, selectMessageById } from "@/stores/chat-message-store";
import { useAgent } from "@/entities/hooks/use-agents";
import { useAgentStatusStore } from "@/stores/agent-status-store";
import { AgentStatusIndicator } from "@/features/chat/components/AgentStatusIndicator";

// ─── Stable AuiIf condition predicates ────────────────────────────────────────
// Defined at module level so their references are stable across renders.
// Passing inline arrow functions to AuiIf creates a new function reference on
// every render, causing assistant-ui's Zustand subscriptions to re-fire and
// triggering the React error #185 infinite update loop.
const condThreadEmpty = (s: { thread: { isEmpty: boolean } }) => s.thread.isEmpty;
const condThreadRunning = (s: { thread: { isRunning: boolean } }) => s.thread.isRunning;
const condThreadNotRunning = (s: { thread: { isRunning: boolean } }) => !s.thread.isRunning;
const condMessageCopied = (s: { message: { isCopied: boolean } }) => s.message.isCopied;
const condMessageNotCopied = (s: { message: { isCopied: boolean } }) => !s.message.isCopied;

// ─── Stable useMessage selectors ──────────────────────────────────────────────
// Same reason as above: useMessage subscribes to per-message state using the
// provided selector function. Inline selectors produce new references each
// render, re-triggering subscriptions.
function selectIsEmptyAndRunning(m: ThreadMessageLike): boolean {
  if (m.status?.type !== "running") return false;
  const parts = (m as unknown as { content?: unknown[] }).content ?? [];
  return (
    parts.length === 0 ||
    parts.every(
      (p) =>
        typeof p === "object" &&
        p !== null &&
        (p as { type?: string; text?: string }).type === "text" &&
        !(p as { text?: string }).text,
    )
  );
}

function selectErrorText(m: ThreadMessageLike): string | null {
  if (m.status?.type !== "incomplete") return null;
  const maybe = m as ThreadMessageLike & {
    metadata?: { custom?: { errorText?: string } };
  };
  return maybe.metadata?.custom?.errorText ?? null;
}

const ThreadAgentStatus: FC = () => {
  const status = useAgentStatusStore((s) => s.status);
  const toolName = useAgentStatusStore((s) => s.toolName);
  return (
    <div className="mx-auto w-full max-w-(--thread-max-width) px-4">
      <AgentStatusIndicator status={{ type: status, toolName }} />
    </div>
  );
};

export const EnhancedThread: FC = () => {
  const activeThreadId = useThreadRegistryStore((s) => s.activeThreadId);
  const agentConfig = useAgentConfig();

  return (
    <ThreadPrimitive.Root
      className="aui-root aui-thread-root @container flex min-h-0 flex-1 flex-col bg-background"
      style={{ ["--thread-max-width" as string]: "48rem" }}
    >
      <ThreadPrimitive.Viewport
        turnAnchor="top"
        className="aui-thread-viewport relative flex min-h-0 flex-1 flex-col overflow-x-auto overflow-y-scroll scroll-smooth px-4 pt-4"
      >
        <AuiIf condition={condThreadEmpty}>
          <UarWelcome />
        </AuiIf>

        <ThreadPrimitive.Messages components={THREAD_COMPONENTS} />

        <ThreadAgentStatus />

        <ThreadPrimitive.ViewportFooter className="aui-thread-viewport-footer sticky bottom-0 mx-auto mt-auto flex w-full max-w-(--thread-max-width) flex-col gap-4 overflow-visible rounded-t-3xl bg-background pb-4 md:pb-6">
          <ThreadScrollToBottom />
          <EnhancedComposer />
          <CapabilityToggles threadId={activeThreadId} agentConfig={agentConfig} className="mx-2" />
        </ThreadPrimitive.ViewportFooter>
      </ThreadPrimitive.Viewport>
    </ThreadPrimitive.Root>
  );
};

// ─── Welcome Screen ───────────────────────────────────────────────────────────

const UarWelcome: FC = () => (
  <div className="mx-auto my-auto flex w-full max-w-(--thread-max-width) grow flex-col">
    <div className="flex w-full grow flex-col items-center justify-center">
      <div className="flex size-full flex-col items-center justify-center gap-4 px-4 text-center">
        <div className="flex items-center justify-center rounded-2xl border border-border/50 bg-muted/30 p-4">
          <SparklesIcon size={28} className="text-primary" />
        </div>
        <div className="space-y-1">
          <h1 className="font-display font-semibold text-2xl tracking-tight text-foreground">Universal Agent Runtime</h1>
          <p className="font-mono text-[11px] text-primary">{"// Ready to assist"}</p>
        </div>
        <p className="max-w-sm font-body text-sm text-muted-foreground leading-relaxed">
          Send a message to start a new conversation. Your agent is configured and ready.
        </p>
        <p className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground/50">Start typing below</p>
      </div>
    </div>
  </div>
);

// ─── Scroll to bottom ─────────────────────────────────────────────────────────

const ThreadScrollToBottom: FC = () => (
  <ThreadPrimitive.ScrollToBottom asChild>
    <TooltipIconButton
      tooltip="Scroll to bottom"
      variant="outline"
      className="absolute -top-12 z-10 self-center rounded-full p-4 disabled:invisible dark:bg-background dark:hover:bg-accent"
    >
      <ArrowDownIcon />
    </TooltipIconButton>
  </ThreadPrimitive.ScrollToBottom>
);

// ─── Composer ────────────────────────────────────────────────────────────────

const EnhancedComposer: FC = () => {
  const attachmentManager = useAttachmentContext();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { memoryEnabled, setMemoryEnabled } = useMemoryContext();
  const activeThreadId = useThreadRegistryStore((s) => s.activeThreadId);

  // Stabilize curried selector references with useCallback so Zustand v5 sees
  // the same selector object across renders when threadKey hasn't changed.
  // The selectors are written inline so threadKey is genuinely in the closure.
  const threadKey = activeThreadId ?? "__none__";
  type StoreState = ReturnType<typeof useChatMessageStore.getState>;
  const selectIsAwaiting = useCallback(
    (s: StoreState) => s.streamingByThread[threadKey]?.awaitingFirstToken ?? false,
    [threadKey],
  );
  const selectRetryAttemptCb = useCallback(
    (s: StoreState) => s.streamingByThread[threadKey]?.retryAttempt ?? 0,
    [threadKey],
  );
  const selectRetryMaxAttemptsCb = useCallback(
    (s: StoreState) => s.streamingByThread[threadKey]?.retryMaxAttempts ?? 0,
    [threadKey],
  );
  const selectRetryDelayMsCb = useCallback(
    (s: StoreState) => s.streamingByThread[threadKey]?.retryDelayMs ?? 0,
    [threadKey],
  );

  const isAwaitingFirstToken = useChatMessageStore(selectIsAwaiting);
  const retryAttempt = useChatMessageStore(selectRetryAttemptCb);
  const retryMaxAttempts = useChatMessageStore(selectRetryMaxAttemptsCb);
  const retryDelayMs = useChatMessageStore(selectRetryDelayMsCb);

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && attachmentManager) {
        attachmentManager.add(e.target.files);
      }
      if (e.target) e.target.value = "";
    },
    [attachmentManager],
  );

  return (
    <ComposerPrimitive.Root className="relative flex w-full flex-col">
      {/* Hidden file input */}
      <input
        ref={fileInputRef}
        type="file"
        multiple
        accept="image/*,.pdf,.doc,.docx,.txt,.md,.json,.csv"
        className="sr-only"
        title="Attach files"
        onChange={handleFileChange}
        aria-hidden
      />

      <ComposerPrimitive.AttachmentDropzone className="relative flex w-full flex-col rounded-2xl border border-input bg-background/80 px-1 pt-2 backdrop-blur-sm outline-none transition-shadow has-[textarea:focus-visible]:border-ring has-[textarea:focus-visible]:ring-2 has-[textarea:focus-visible]:ring-ring/20 overflow-hidden">

        {/* Animated progress bar — only visible while waiting for first streamed token */}
        {isAwaitingFirstToken && (
          <div className="absolute inset-x-0 top-0 h-[2px] overflow-hidden rounded-t-2xl">
            <div className="h-full w-1/2 animate-[shimmer_1.4s_ease-in-out_infinite] bg-gradient-to-r from-transparent via-primary to-transparent" />
          </div>
        )}

        {/* Attachment preview strip */}
        {attachmentManager && attachmentManager.pending.length > 0 && (
          <AttachmentPreviewStrip
            attachments={attachmentManager.pending}
            onRemove={attachmentManager.remove}
          />
        )}

        <ComposerPrimitive.Input
          placeholder="Send a message to the agent…"
          className="mb-1 max-h-48 min-h-[3.5rem] w-full resize-none bg-transparent px-4 pt-3 pb-3 font-body text-sm text-foreground outline-none placeholder:text-muted-foreground/50 focus-visible:ring-0 disabled:cursor-not-allowed disabled:opacity-40 transition-opacity"
          rows={1}
          autoFocus
          aria-label="Message input"
        />

        <div className="relative mx-2 mb-2 flex items-center gap-2">
          {/* Attach file button */}
          {attachmentManager && (
            <TooltipIconButton
              tooltip="Attach file"
              side="top"
              type="button"
              variant="ghost"
              size="icon"
              className="size-7 rounded-full text-muted-foreground hover:text-foreground"
              aria-label="Attach file"
              onClick={() => fileInputRef.current?.click()}
            >
              <PaperclipIcon className="size-4" />
            </TooltipIconButton>
          )}

          {/* Memory toggle button */}
          <TooltipIconButton
            tooltip={memoryEnabled ? "Memory ON – click to disable" : "Memory OFF – click to enable"}
            side="top"
            type="button"
            variant="ghost"
            size="icon"
            className={cn(
              "size-7 rounded-full transition-colors",
              memoryEnabled
                ? "text-primary hover:text-primary/80"
                : "text-muted-foreground/40 hover:text-muted-foreground"
            )}
            aria-label={memoryEnabled ? "Disable memory for this message" : "Enable memory for this message"}
            onClick={() => setMemoryEnabled(!memoryEnabled)}
          >
            <BrainIcon className="size-4" />
          </TooltipIconButton>

          {/* Request in-flight indicator (pre-stream) */}
          {isAwaitingFirstToken && (
            <span className="ml-auto flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground/70">
              <Loader2Icon size={11} className="animate-spin text-primary" />
              {retryAttempt > 0
                ? `Retrying (${retryAttempt}/${Math.max(retryMaxAttempts, retryAttempt)}) in ${(retryDelayMs / 1000).toFixed(1)}s…`
                : "Waiting for model…"}
            </span>
          )}

          {/* Running indicator (after stream begins) */}
          <AuiIf condition={condThreadRunning}>
            <span className={cn("ml-auto flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground/70", isAwaitingFirstToken && "hidden")}>
              <span className="inline-flex items-center gap-0.5">
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.2s_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.4s_infinite]" />
              </span>
              Generating…
            </span>
          </AuiIf>

          {/* Send button — hidden while running */}
          <AuiIf condition={condThreadNotRunning}>
            <ComposerPrimitive.Send asChild>
              <TooltipIconButton tooltip="Send message" side="bottom" type="submit" variant="default" size="icon" className="size-8 rounded-full bg-primary text-primary-foreground hover:bg-primary/90 ml-auto" aria-label="Send message">
                <ArrowUpIcon className="size-4" />
              </TooltipIconButton>
            </ComposerPrimitive.Send>
          </AuiIf>

          {/* Cancel button — shown while running */}
          <AuiIf condition={condThreadRunning}>
            <ComposerPrimitive.Cancel asChild>
              <Button type="button" variant="default" size="icon" className="size-8 rounded-full" aria-label="Stop generating">
                <SquareIcon className="size-3 fill-current" />
              </Button>
            </ComposerPrimitive.Cancel>
          </AuiIf>
        </div>
      </ComposerPrimitive.AttachmentDropzone>
    </ComposerPrimitive.Root>
  );
};

// ─── Avatars ──────────────────────────────────────────────────────────────────

const UserAvatar: FC = () => (
  <div className="flex flex-col items-center gap-1 pt-0.5">
    <Avatar className="size-8 ring-1 ring-zinc-600">
      <AvatarFallback className="bg-zinc-700 text-zinc-200">
        <UserIcon size={14} />
      </AvatarFallback>
    </Avatar>
    <span className="font-mono text-[9px] uppercase tracking-wider text-muted-foreground/60">You</span>
  </div>
);

const AgentAvatar: FC = () => (
  <div className="flex flex-col items-center gap-1 pt-0.5">
    <Avatar className="size-8 ring-1 ring-primary/30">
      <AvatarFallback className="bg-primary/15 text-primary">
        <SparklesIcon size={14} />
      </AvatarFallback>
    </Avatar>
    <span className="font-mono text-[9px] uppercase tracking-wider text-primary/70">Agent</span>
  </div>
);

// ─── User Message ─────────────────────────────────────────────────────────────

const UserMessage: FC = () => (
  <MessagePrimitive.Root className="fade-in slide-in-from-bottom-1 mx-auto flex w-full max-w-(--thread-max-width) animate-in flex-col gap-0.5 px-4 py-2 duration-150" data-role="user">
    <div className="flex w-full items-start gap-3">
      <UserActionBar />
      <div className="min-w-0 flex-1">
        <div className="wrap-break-word rounded-2xl rounded-tr-sm bg-zinc-800 px-4 py-3 font-body text-sm text-foreground leading-relaxed shadow-sm">
          <MessagePrimitive.Parts components={USER_MESSAGE_PARTS_COMPONENTS} />
        </div>
      </div>
      <UserAvatar />
    </div>
    <div className="pr-11"><BranchPicker /></div>
  </MessagePrimitive.Root>
);

const UserActionBar: FC = () => (
  <ActionBarPrimitive.Root hideWhenRunning autohide="not-last" className="flex shrink-0 flex-col items-end pt-2">
    <ActionBarPrimitive.Edit asChild>
      <TooltipIconButton tooltip="Edit" className="p-2"><PencilIcon /></TooltipIconButton>
    </ActionBarPrimitive.Edit>
  </ActionBarPrimitive.Root>
);

// ─── Assistant Message ────────────────────────────────────────────────────────

/** Shows a spinner when the assistant message is still empty (pre-first-token). */
const AssistantMessageBody: FC = () => {
  const isEmptyAndRunning = useMessage(selectIsEmptyAndRunning);

  return (
    <>
      {isEmptyAndRunning ? (
        <div className="flex items-center gap-2.5 py-1 text-muted-foreground/70">
          <Loader2Icon size={14} className="animate-spin text-primary" />
          <span className="font-mono text-[11px]">Agent is thinking…</span>
        </div>
      ) : (
        <MessagePrimitive.Parts components={ASSISTANT_MESSAGE_PARTS_COMPONENTS} />
      )}
      <MessageError />
    </>
  );
};

const AssistantMessage: FC = () => (
  <MessagePrimitive.Root className="fade-in slide-in-from-bottom-1 mx-auto flex w-full max-w-(--thread-max-width) animate-in flex-col gap-0.5 px-4 py-2 duration-150" data-role="assistant">
    <div className="flex w-full items-start gap-3">
      <AgentAvatar />
      <div className="min-w-0 flex-1">
        <div className="wrap-break-word rounded-2xl rounded-tl-sm bg-muted/60 px-4 py-3 font-body text-sm text-foreground leading-relaxed shadow-sm">
          <AssistantMessageBody />
        </div>
      </div>
    </div>
    <div className="ml-11 flex"><BranchPicker /><AssistantActionBar /></div>
  </MessagePrimitive.Root>
);

// ─── Reasoning Part ───────────────────────────────────────────────────────────

const ReasoningPart: FC = () => {
  const { text, status } = useMessagePartText();
  const isStreaming = status.type === "running";
  const [isOpen, setIsOpen] = useState(isStreaming);

  return (
    <Card className="my-2 overflow-hidden rounded-lg border-border/50 bg-muted/20 shadow-none">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger
          render={
            <Button
              variant="ghost"
              className="flex h-auto w-full items-center justify-start gap-2 rounded-none px-3 py-2 hover:bg-muted/30"
              aria-expanded={isOpen}
            />
          }
        >
          <BrainIcon size={13} className="shrink-0 text-muted-foreground" />
          <span className="flex-1 font-mono text-[11px] text-muted-foreground">
            {isStreaming ? (
              <span className="flex items-center gap-2">
                {"// Reasoning"}
                <span className="inline-flex gap-0.5">
                  <span className="h-1 w-1 animate-pulse rounded-full bg-primary/60" />
                  <span className="h-1 w-1 animate-pulse rounded-full bg-primary/60 [animation-delay:0.2s]" />
                  <span className="h-1 w-1 animate-pulse rounded-full bg-primary/60 [animation-delay:0.4s]" />
                </span>
              </span>
            ) : "// Reasoning"}
          </span>
          <ChevronDownIcon
            size={13}
            className={cn("shrink-0 text-muted-foreground transition-transform duration-150", isOpen && "rotate-180")}
          />
        </CollapsibleTrigger>
        <CollapsibleContent>
          <Separator className="opacity-30" />
          <CardContent className="px-3 pb-3 pt-2">
            <p className="whitespace-pre-wrap font-body text-sm leading-relaxed text-muted-foreground">
              {text}
              {isStreaming && <span className="ml-0.5 inline-block h-3.5 w-0.5 animate-[pulse_1s_step-end_infinite] bg-primary" />}
            </p>
          </CardContent>
        </CollapsibleContent>
      </Collapsible>
    </Card>
  );
};

// ─── Tool Call Part ───────────────────────────────────────────────────────────

const ToolCallPart: FC<ToolCallMessagePartProps> = ({ toolName, args, result, status }) => {
  if (toolName === "__skill__") {
    const a = args as { skillId: string; skillName: string; selectionMethod?: string; status: "active" | "complete" };
    return <SkillActivationBlock skillId={a.skillId} skillName={a.skillName} selectionMethod={a.selectionMethod} status={a.status} />;
  }
  if (toolName === "__context__") {
    const a = args as { strategy: string; messagesRemoved: number; tokensSaved: number; wasApplied: boolean; summaryGenerated: boolean };
    return <ContextUpdateBlock strategy={a.strategy} messagesRemoved={a.messagesRemoved} tokensSaved={a.tokensSaved} wasApplied={a.wasApplied} summaryGenerated={a.summaryGenerated} />;
  }
  if (toolName === "__memory_recall__") {
    const a = args as {
      items: Array<{
        key: string;
        value: string;
        source: string;
        scope?: string;
        memory_type?: string;
        importance?: number;
      }>;
      count?: number;
    };
    return <MemoryRecallBlock items={a.items ?? []} count={a.count} />;
  }
  if (toolName === "__memory_mutation__") {
    const a = args as {
      operation: string;
      memoryId: string;
      content: string;
      scope: string;
      memoryType: string;
    };
    return (
      <MemoryMutationBlock
        operation={a.operation}
        memoryId={a.memoryId}
        content={a.content}
        scope={a.scope}
        memoryType={a.memoryType}
      />
    );
  }
  if (toolName === "__memory_update__") {
    const a = args as { key: string; operation: string; value: string };
    return <MemoryUpdateBlock memoryKey={a.key} operation={a.operation} value={a.value} />;
  }
  if (toolName === "__a2ui_input__") {
    const a = args as {
      runId: string;
      artifactId: string;
      artifactType: string;
      title: string;
      content: string;
      metadata: Record<string, unknown>;
    };
    const artifactStatus: "running" | "complete" | "failed" =
      status.type === "running" ? "running" : status.type === "incomplete" ? "failed" : "complete";
    return (
      <A2uiInputBlock
        runId={a.runId}
        artifactId={a.artifactId}
        artifactType={a.artifactType}
        title={a.title}
        content={a.content}
        metadata={a.metadata}
        status={artifactStatus}
        result={typeof result === "string" ? result : result ? JSON.stringify(result) : undefined}
      />
    );
  }
  if (toolName === "__a2ui_display__") {
    const a = args as { artifactType: string; title: string; language?: string };
    return (
      <A2uiDisplayBlock
        artifactType={a.artifactType}
        title={a.title}
        language={a.language}
        content={typeof result === "string" ? result : result ? JSON.stringify(result, null, 2) : ""}
      />
    );
  }
  return <ToolCallBlockWrapper toolName={toolName} args={args as Record<string, unknown>} result={result} status={status} />;
};

// ─── Message Error ────────────────────────────────────────────────────────────

const MessageError: FC = () => {
  const [copied, setCopied] = useState(false);
  const [isOpen, setIsOpen] = useState(true);

  const errorText = useMessage(selectErrorText);

  const handleCopy = useCallback(() => {
    if (!errorText) return;
    void navigator.clipboard.writeText(errorText).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [errorText]);

  if (!errorText) return null;

  return (
    <Card className="mt-3 overflow-hidden rounded-xl border-destructive/60 bg-destructive/5 shadow-sm">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        {/* Error header */}
        <div className="flex items-center gap-2 border-b border-destructive/30 bg-destructive/10 px-3 py-2">
          <AlertTriangleIcon size={14} className="shrink-0 text-destructive" />
          <span className="flex-1 font-mono text-[11px] font-semibold uppercase tracking-widest text-destructive">
            Agent Error
          </span>
          <CollapsibleTrigger
            render={
              <Button
                variant="ghost"
                size="sm"
                className="flex h-auto items-center gap-1 rounded px-1.5 py-0.5 font-mono text-[10px] text-destructive/70 hover:bg-destructive/10 hover:text-destructive"
                aria-label={isOpen ? "Collapse error" : "Expand error"}
              />
            }
          >
            {isOpen ? "collapse" : "expand"}
            <ChevronDownIcon size={11} className={cn("transition-transform", isOpen && "rotate-180")} />
          </CollapsibleTrigger>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleCopy}
            className="flex h-auto items-center gap-1 rounded px-1.5 py-0.5 font-mono text-[10px] text-destructive/70 hover:bg-destructive/10 hover:text-destructive"
            aria-label="Copy error details"
          >
            {copied ? <CheckIcon size={11} /> : <ClipboardIcon size={11} />}
            {copied ? "copied" : "copy"}
          </Button>
        </div>
        {/* Error body */}
        <CollapsibleContent>
          <CardContent className="px-3 py-3">
            <pre className="whitespace-pre-wrap break-all font-mono text-[11px] leading-relaxed text-destructive/90">
              {errorText}
            </pre>
          </CardContent>
        </CollapsibleContent>
      </Collapsible>
    </Card>
  );
};

// ─── Per-message metadata chips ───────────────────────────────────────────────
// Rendered to the right of the Copy button: which agent answered, the model used,
// token usage (↑in ↓out), and counts of skills activated / citations for this
// message. Each chip hides when its datum is absent (older messages, no usage).

const MetaChip: FC<{ icon: ReactNode; label: string; title: string }> = ({ icon, label, title }) => (
  <span title={title} className="inline-flex items-center gap-1 rounded-md bg-muted/60 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
    {icon}
    {label}
  </span>
);

const MessageMetaChips: FC = () => {
  const messageId = useMessage((m) => m.id);
  const activeThreadId = useThreadRegistryStore((s) => s.activeThreadId);
  // Memoize the selector so its reference is stable across renders — an inline
  // selector re-fires assistant-ui's Zustand subscription (React error #185).
  const selector = useMemo(
    () => (activeThreadId ? selectMessageById(activeThreadId, messageId) : () => null),
    [activeThreadId, messageId],
  );
  const message = useChatMessageStore(selector);
  const answeringAgent = useAgent(message?.agentId);

  if (!message) return null;

  const agentLabel = message.agentId
    ? (answeringAgent?.metadata?.title ?? message.agentId)
    : null;
  const skillCount = message.content.filter((b) => b.type === "skill-activation").length;
  const citationCount = message.content.filter((b) => b.type === "citation").length;

  const hasAny =
    agentLabel || message.model || message.usage || skillCount > 0 || citationCount > 0;
  if (!hasAny) return null;

  return (
    <div className="ml-1 flex flex-wrap items-center gap-1">
      {agentLabel && (
        <MetaChip icon={<BotIcon className="size-3" />} label={agentLabel} title={`Answered by agent: ${agentLabel}`} />
      )}
      {message.model && (
        <MetaChip icon={<SparklesIcon className="size-3" />} label={message.model} title={`Model: ${message.model}`} />
      )}
      {message.usage && (
        <span
          title={`Tokens — in: ${message.usage.inputTokens}, out: ${message.usage.outputTokens}, total: ${message.usage.totalTokens}`}
          className="inline-flex items-center gap-0.5 rounded-md bg-muted/60 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground"
        >
          <ArrowUpIcon className="size-3" />
          {message.usage.inputTokens}
          <ArrowDownIcon className="ml-1 size-3" />
          {message.usage.outputTokens}
        </span>
      )}
      {skillCount > 0 && (
        <MetaChip icon={<ZapIcon className="size-3" />} label={String(skillCount)} title={`${skillCount} skill${skillCount === 1 ? "" : "s"} activated`} />
      )}
      {citationCount > 0 && (
        <MetaChip icon={<BookTextIcon className="size-3" />} label={String(citationCount)} title={`${citationCount} citation${citationCount === 1 ? "" : "s"}`} />
      )}
    </div>
  );
};

// ─── Assistant Action Bar ─────────────────────────────────────────────────────

const AssistantActionBar: FC = () => (
  <ActionBarPrimitive.Root hideWhenRunning autohide="not-last" autohideFloat="single-branch" className="col-start-3 row-start-2 -ml-1 flex items-center gap-1 text-muted-foreground">
    <ActionBarPrimitive.Copy asChild>
      <TooltipIconButton tooltip="Copy">
        <AuiIf condition={condMessageCopied}><CheckIcon /></AuiIf>
        <AuiIf condition={condMessageNotCopied}><CopyIcon /></AuiIf>
      </TooltipIconButton>
    </ActionBarPrimitive.Copy>
    <MessageMetaChips />
  </ActionBarPrimitive.Root>
);

// ─── Edit Composer ────────────────────────────────────────────────────────────

const EditComposer: FC = () => (
  <MessagePrimitive.Root className="mx-auto flex w-full max-w-(--thread-max-width) flex-col px-2 py-3">
    <ComposerPrimitive.Root className="ml-auto flex w-full max-w-[85%] flex-col rounded-2xl bg-muted">
      <ComposerPrimitive.Input className="min-h-14 w-full resize-none bg-transparent p-4 font-body text-foreground text-sm outline-none" autoFocus />
      <CardFooter className="mx-3 mb-3 flex items-center gap-2 self-end p-0">
        <ComposerPrimitive.Cancel asChild><Button variant="ghost" size="sm">Cancel</Button></ComposerPrimitive.Cancel>
        <ComposerPrimitive.Send asChild><Button size="sm">Update</Button></ComposerPrimitive.Send>
      </CardFooter>
    </ComposerPrimitive.Root>
  </MessagePrimitive.Root>
);

// ─── Branch Picker ────────────────────────────────────────────────────────────

const BranchPicker: FC<BranchPickerPrimitive.Root.Props> = ({ className, ...rest }) => (
  <BranchPickerPrimitive.Root hideWhenSingleBranch className={cn("mr-2 -ml-2 inline-flex items-center text-muted-foreground text-xs", className)} {...rest}>
    <BranchPickerPrimitive.Previous asChild><TooltipIconButton tooltip="Previous branch"><ChevronLeftIcon /></TooltipIconButton></BranchPickerPrimitive.Previous>
    <span className="font-mono font-medium"><BranchPickerPrimitive.Number /> / <BranchPickerPrimitive.Count /></span>
    <BranchPickerPrimitive.Next asChild><TooltipIconButton tooltip="Next branch"><ChevronRightIcon /></TooltipIconButton></BranchPickerPrimitive.Next>
  </BranchPickerPrimitive.Root>
);

// ─── Stable components objects ─────────────────────────────────────────────────
// Defined after all component declarations so they can reference them.
// Object literals created inline inside JSX are new references on every render,
// which causes assistant-ui's context/subscription machinery to re-render
// indefinitely (React error #185). Module-level constants are created once.
const USER_MESSAGE_PARTS_COMPONENTS = { Text: EnhancedMarkdownText };
const ASSISTANT_MESSAGE_PARTS_COMPONENTS = {
  Text: EnhancedMarkdownText,
  Reasoning: ReasoningPart,
  tools: { Fallback: ToolCallPart },
};
const THREAD_COMPONENTS = { UserMessage, EditComposer, AssistantMessage };
