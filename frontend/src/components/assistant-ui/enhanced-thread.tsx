import {
  ActionBarPrimitive,
  AuiIf,
  BranchPickerPrimitive,
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  groupPartByType,
  type ThreadMessageLike,
  type ToolCallMessagePartProps,
  useAuiState,
  useMessage,
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
  LoaderIcon,
  PaperclipIcon,
  PencilIcon,
  SparklesIcon,
  SquareIcon,
  ZapIcon,
} from "lucide-react";
import { type FC, type ReactNode, useCallback, useMemo, useRef, useState } from "react";
import { MarkdownBubble } from "@/shared/markdown";
import { TooltipIconButton } from "@/components/assistant-ui/tooltip-icon-button";
import { UarWordmark } from "@/shared/ui/uar-logo";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { ToolCallBlockWrapper } from "@/features/chat/components/tool-call-block";
import { chatMessageAnchorId } from "@/features/chat/model/message-anchor";
import { AttachmentPreviewStrip } from "@/features/chat/components/attachment-preview";
import { useAttachmentContext } from "@/features/chat/attachment-context";
import { useMemoryContext } from "@/features/chat/memory-context";
import { cn } from "@/lib/utils";
import { CapabilityToggles } from "@/features/chat/capability-toggles";
import { useAgentConfig } from "@/features/chat/agent-config-context";
import { selectMessageById, type ChatMessageStoreState, useChatMessageSelector, useThreadUi } from "@/hooks/use-thread-ui";
import { MessageCitations } from "@/components/citations/citation-hover-panel";
import { useAgent } from "@/features/agents/model";
import { useAgentStatus } from "@/hooks/use-agent-status";
import { AgentStatusIndicator } from "@/features/chat/components/AgentStatusIndicator";
import { RichDataRenderers } from "@/features/chat/ui/chunks";

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
  const { status, toolName } = useAgentStatus();
  return (
    <div className="mx-auto w-full max-w-(--thread-max-width) px-4">
      <AgentStatusIndicator status={{ type: status, toolName }} />
    </div>
  );
};

export const EnhancedThread: FC = () => {
  const { activeThreadId } = useThreadUi();
  const agentConfig = useAgentConfig();
  const isEmpty = useAuiState(condThreadEmpty);

  return (
    <>
      <RichDataRenderers />
      <ThreadPrimitive.Root
      className="aui-root aui-thread-root @container flex min-h-0 flex-1 flex-col bg-background"
      style={{
        ["--thread-max-width" as string]: "44rem",
        ["--composer-bg" as string]: "var(--color-surface)",
        ["--composer-radius" as string]: "1.5rem",
        ["--composer-padding" as string]: "10px",
      }}
    >
      <ThreadPrimitive.Viewport
        turnAnchor="top"
        className="aui-thread-viewport relative flex min-h-0 flex-1 flex-col overflow-x-hidden overflow-y-auto scroll-smooth"
      >
        <div
          className={cn(
            "mx-auto flex w-full max-w-(--thread-max-width) flex-1 flex-col px-4 pt-4",
            isEmpty && "justify-center",
          )}
        >
          <AuiIf condition={condThreadEmpty}>
            <UarWelcome />
          </AuiIf>

          <div data-slot="aui_message-group" className="mb-14 flex flex-col gap-y-6 empty:hidden">
            <ThreadPrimitive.Messages components={THREAD_COMPONENTS} />
          </div>

          <ThreadAgentStatus />

          <ThreadPrimitive.ViewportFooter className="aui-thread-viewport-footer sticky bottom-0 mt-auto flex w-full flex-col gap-3 overflow-visible rounded-t-(--composer-radius) bg-background/95 pb-4 md:pb-6">
            <ThreadScrollToBottom />
            <EnhancedComposer />
            <CapabilityToggles threadId={activeThreadId} agentConfig={agentConfig} className="mx-2" />
          </ThreadPrimitive.ViewportFooter>
        </div>
      </ThreadPrimitive.Viewport>
      </ThreadPrimitive.Root>
    </>
  );
};

// ─── Welcome Screen ───────────────────────────────────────────────────────────

const UarWelcome: FC = () => (
  <div className="flex w-full flex-col items-center justify-center">
    <div className="flex flex-col items-center justify-center gap-4 px-4 py-16 text-center">
      <UarWordmark className="h-16 w-full max-w-sm" />
      <div className="space-y-1">
        <h1 className="sr-only">Universal Agent Runtime</h1>
        <p className="font-mono text-[11px] text-primary">{"// Ready to assist"}</p>
      </div>
      <p className="max-w-sm font-body text-sm text-muted-foreground leading-relaxed">
        Send a message to start a new conversation. Your agent is configured and ready.
      </p>
      <p className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground/50">Start typing below</p>
    </div>
  </div>
);

// ─── Scroll to bottom ─────────────────────────────────────────────────────────

const ThreadScrollToBottom: FC = () => (
  <ThreadPrimitive.ScrollToBottom
    render={
      <TooltipIconButton
        tooltip="Scroll to bottom"
        variant="secondary"
        className="absolute -top-12 z-10 self-center rounded-full p-4 shadow-none disabled:invisible"
      />
    }
  >
    <ArrowDownIcon />
  </ThreadPrimitive.ScrollToBottom>
);

// ─── Composer ────────────────────────────────────────────────────────────────
// KnowMe idiom: a filled surface, never an outlined input. Flat 2.0 — no
// border, ring, blur, or elevation; focus shifts the fill one ladder step.

const EnhancedComposer: FC = () => {
  const attachmentManager = useAttachmentContext();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { memoryEnabled, setMemoryEnabled } = useMemoryContext();
  const { activeThreadId } = useThreadUi();

  // Stabilize curried selector references with useCallback so Zustand v5 sees
  // the same selector object across renders when threadKey hasn't changed.
  // The selectors are written inline so threadKey is genuinely in the closure.
  const threadKey = activeThreadId ?? "__none__";
  const selectIsAwaiting = useCallback(
    (s: ChatMessageStoreState) => s.streamingByThread[threadKey]?.awaitingFirstToken ?? false,
    [threadKey],
  );
  const selectRetryAttemptCb = useCallback(
    (s: ChatMessageStoreState) => s.streamingByThread[threadKey]?.retryAttempt ?? 0,
    [threadKey],
  );
  const selectRetryMaxAttemptsCb = useCallback(
    (s: ChatMessageStoreState) => s.streamingByThread[threadKey]?.retryMaxAttempts ?? 0,
    [threadKey],
  );
  const selectRetryDelayMsCb = useCallback(
    (s: ChatMessageStoreState) => s.streamingByThread[threadKey]?.retryDelayMs ?? 0,
    [threadKey],
  );

  const isAwaitingFirstToken = useChatMessageSelector(selectIsAwaiting);
  const retryAttempt = useChatMessageSelector(selectRetryAttemptCb);
  const retryMaxAttempts = useChatMessageSelector(selectRetryMaxAttemptsCb);
  const retryDelayMs = useChatMessageSelector(selectRetryDelayMsCb);

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

      <ComposerPrimitive.AttachmentDropzone className="relative flex w-full flex-col gap-2 overflow-hidden rounded-(--composer-radius) bg-(--composer-bg) p-(--composer-padding) shadow-none outline-none transition-colors focus-within:bg-card-hov data-[dragging=true]:bg-accent">

        {/* Attachment preview strip */}
        {attachmentManager && attachmentManager.pending.length > 0 && (
          <AttachmentPreviewStrip
            attachments={attachmentManager.pending}
            onRemove={attachmentManager.remove}
          />
        )}

        <ComposerPrimitive.Input
          placeholder="Send a message…"
          className="max-h-32 min-h-10 w-full resize-none bg-transparent px-2.5 py-1 font-body text-base text-foreground caret-primary outline-none placeholder:text-fg-faint focus-visible:ring-0 disabled:cursor-not-allowed disabled:opacity-40 transition-opacity"
          rows={1}
          autoFocus
          enterKeyHint="send"
          aria-label="Message input"
        />

        <div className="relative mx-1 mb-0.5 flex items-center gap-1.5">
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

          {/* Request in-flight indicator (pre-stream) — run-phase text, no spinner */}
          {isAwaitingFirstToken && (
            <span className="ml-auto flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground/70">
              <span className="inline-flex items-center gap-0.5">
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.2s_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.4s_infinite]" />
              </span>
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
            <ComposerPrimitive.Send
              render={
                <TooltipIconButton
                  tooltip="Send message"
                  side="bottom"
                  type="submit"
                  variant="default"
                  size="icon"
                  className="ml-auto size-8 rounded-full bg-primary text-primary-foreground hover:bg-primary/90"
                  aria-label="Send message"
                />
              }
            >
              <ArrowUpIcon className="size-4" />
            </ComposerPrimitive.Send>
          </AuiIf>

          {/* Cancel button — shown while running */}
          <AuiIf condition={condThreadRunning}>
            <ComposerPrimitive.Cancel
              render={
                <Button
                  type="button"
                  variant="default"
                  size="icon"
                  className="size-8 rounded-full"
                  aria-label="Stop generating"
                />
              }
            >
              <SquareIcon className="size-3 fill-current" />
            </ComposerPrimitive.Cancel>
          </AuiIf>
        </div>
      </ComposerPrimitive.AttachmentDropzone>
    </ComposerPrimitive.Root>
  );
};

// ─── User Message ─────────────────────────────────────────────────────────────
// KnowMe anatomy: trailing ember-soft bubble, no avatar, hover action bar
// beside the bubble, branch picker on the row below.

const UserMessage: FC = () => {
  const messageId = useMessage((message) => message.id);
  return (
    <MessagePrimitive.Root
      id={chatMessageAnchorId(messageId)}
      data-message-id={messageId}
      data-slot="aui_user-message-root"
      data-role="user"
      tabIndex={-1}
      className="fade-in slide-in-from-bottom-1 animate-in grid auto-rows-auto grid-cols-[minmax(72px,1fr)_auto] content-start gap-y-2 px-2 duration-150 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember [&:where(>*)]:col-start-2"
    >
      <div className="relative col-start-2 min-w-0">
        <div className="peer max-w-[80%] rounded-2xl rounded-ee-md bg-ember-soft px-4 py-3 font-body text-sm leading-relaxed text-foreground wrap-break-word empty:hidden">
          <MessagePrimitive.Parts components={USER_MESSAGE_PARTS_COMPONENTS} />
        </div>
        <div className="absolute start-0 top-1/2 -translate-x-full -translate-y-1/2 pe-2 peer-empty:hidden rtl:translate-x-full">
          <UserActionBar />
        </div>
      </div>

      <BranchPicker className="col-span-full col-start-1 row-start-3 -me-1 justify-end" />
    </MessagePrimitive.Root>
  );
};

const UserActionBar: FC = () => (
  <ActionBarPrimitive.Root hideWhenRunning autohide="not-last" className="flex flex-col items-end text-muted-foreground">
    <ActionBarPrimitive.Copy render={<TooltipIconButton tooltip="Copy" />}>
      <CopyIcon />
    </ActionBarPrimitive.Copy>
    <ActionBarPrimitive.Edit render={<TooltipIconButton tooltip="Edit" />}>
      <PencilIcon />
    </ActionBarPrimitive.Edit>
  </ActionBarPrimitive.Root>
);

// ─── Assistant Message ────────────────────────────────────────────────────────

/** Shows a run-phase line when the assistant message is still empty (pre-first-token). */
const AssistantMessageBody: FC = () => {
  const isEmptyAndRunning = useMessage(selectIsEmptyAndRunning);

  if (isEmptyAndRunning) {
    return (
      <div className="flex items-center gap-2.5 py-1 text-muted-foreground/70">
        <span className="inline-flex items-center gap-0.5">
          <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_infinite]" />
          <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.2s_infinite]" />
          <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.4s_infinite]" />
        </span>
        <span className="font-mono text-[11px]">Agent is thinking…</span>
      </div>
    );
  }

  return (
    <>
      <MessagePrimitive.GroupedParts groupBy={GROUP_BY}>
        {({ part, children }) => {
          switch (part.type) {
            case "group-chainOfThought":
              return <div data-slot="aui_chain-of-thought">{children}</div>;
            case "group-tool":
              return (
                <ToolGroup
                  count={part.indices.length}
                  active={part.status.type === "running"}
                >
                  {children}
                </ToolGroup>
              );
            case "group-reasoning":
              return (
                <ReasoningGroup streaming={part.status.type === "running"}>
                  {children}
                </ReasoningGroup>
              );
            case "text":
              return (
                <div className="my-2 w-fit max-w-[80%] rounded-2xl rounded-es-md bg-card px-4 py-3 text-foreground">
                  <MarkdownBubble />
                </div>
              );
            case "reasoning":
              return (
                <pre className="m-0 whitespace-pre-wrap break-words font-body text-sm leading-relaxed text-muted-foreground select-text">
                  {(part as { text?: string }).text}
                </pre>
              );
            case "tool-call":
              return (
                (part as { toolUI?: ReactNode }).toolUI ?? (
                  <ToolCallPart {...(part as unknown as ToolCallMessagePartProps)} />
                )
              );
            case "data":
              return (part as { dataRendererUI?: ReactNode }).dataRendererUI ?? null;
            case "indicator":
              return (
                <span data-slot="aui_assistant-message-indicator" className="animate-pulse" aria-label="Assistant is working">
                  {"●"}
                </span>
              );
            default:
              return null;
          }
        }}
      </MessagePrimitive.GroupedParts>
      <MessageError />
    </>
  );
};

const AssistantMessage: FC = () => {
  const messageId = useMessage((message) => message.id);
  return (
    <MessagePrimitive.Root
      id={chatMessageAnchorId(messageId)}
      data-message-id={messageId}
      data-slot="aui_assistant-message-root"
      data-role="assistant"
      tabIndex={-1}
      className="fade-in slide-in-from-bottom-1 animate-in relative duration-150 focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ember [contain-intrinsic-size:auto_200px] [content-visibility:auto]"
    >
      <div data-slot="aui_assistant-message-content" className="px-2 leading-relaxed text-foreground wrap-break-word">
        <AssistantMessageBody />
        <AssistantMessageCitations />
      </div>
      <div data-slot="aui_assistant-message-footer" className="ms-2 flex min-h-7.5 items-center pt-1.5">
        <BranchPicker />
        <AssistantActionBar />
      </div>
    </MessagePrimitive.Root>
  );
};

// ─── Reasoning group ──────────────────────────────────────────────────────────
// KnowMe idiom: collapsed by default on a cyan-tinted surface; auto-opens
// while streaming and auto-collapses when streaming ends, until the user
// takes over manually.

const ReasoningGroup: FC<{ streaming: boolean; children: ReactNode }> = ({ streaming, children }) => {
  const [userOpen, setUserOpen] = useState<boolean | null>(null);
  const isOpen = userOpen ?? streaming;

  return (
    <div className="my-2 w-full max-w-[80%] rounded-xl bg-[color-mix(in_srgb,var(--color-cyan)_7%,transparent)] px-3 py-2">
      <Collapsible open={isOpen} onOpenChange={(open) => setUserOpen(open)}>
        <CollapsibleTrigger
          className="group/trigger flex max-w-[75%] origin-left items-center gap-2 py-1.5 text-sm text-[var(--color-cyan)] transition-colors hover:text-foreground"
          aria-expanded={isOpen}
        >
          <BrainIcon className="size-4 shrink-0" />
          <span className="leading-none tabular-nums">
            {streaming ? "Reasoning…" : "Reasoning"}
          </span>
          <ChevronDownIcon
            className={cn(
              "mt-0.5 size-4 shrink-0 transition-transform duration-200 motion-reduce:transition-none",
              isOpen ? "rotate-0" : "-rotate-90",
            )}
          />
        </CollapsibleTrigger>
        <CollapsibleContent
          aria-busy={streaming}
          className="relative overflow-hidden text-sm text-muted-foreground outline-none data-[state=closed]:animate-none"
        >
          <div className="max-h-64 overflow-y-auto ps-6 pt-2 pb-2 leading-relaxed">{children}</div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
};

// ─── Tool group ───────────────────────────────────────────────────────────────
// Consecutive tool calls collapse into one disclosure with a count trigger.

const ToolGroup: FC<{ count: number; active: boolean; children: ReactNode }> = ({ count, active, children }) => {
  const [isOpen, setIsOpen] = useState(false);
  const label = `${count} tool ${count === 1 ? "call" : "calls"}`;

  return (
    <div className="my-1 w-full">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger
          className="group/trigger flex origin-left items-center gap-2 py-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
          aria-expanded={isOpen}
        >
          {active && <LoaderIcon className="size-3 shrink-0 animate-spin [animation-duration:0.6s]" />}
          <span className="text-xs leading-none">{label}</span>
          <ChevronDownIcon
            className={cn(
              "size-3 shrink-0 transition-transform duration-200 motion-reduce:transition-none",
              isOpen ? "rotate-0" : "-rotate-90",
            )}
          />
        </CollapsibleTrigger>
        <CollapsibleContent className="relative overflow-hidden text-sm outline-none">
          <div className="mt-1 flex flex-col gap-1">{children}</div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
};

// ─── Tool Call Part ───────────────────────────────────────────────────────────

const ToolCallPart: FC<ToolCallMessagePartProps> = ({ toolName, args, result, status }) => {
  return <ToolCallBlockWrapper toolName={toolName} args={args as Record<string, unknown>} result={result} status={status} />;
};

// ─── Message Error ────────────────────────────────────────────────────────────
// Flat 2.0: destructive-tinted surface, no border or Card chrome.

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
    <div className="mt-2 overflow-hidden rounded-md bg-destructive/10">
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        {/* Error header */}
        <div className="flex items-center gap-2 px-3 py-2">
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
          <div className="px-3 pb-3">
            <pre className="whitespace-pre-wrap break-all font-mono text-[11px] leading-relaxed text-destructive/90">
              {errorText}
            </pre>
          </div>
        </CollapsibleContent>
      </Collapsible>
    </div>
  );
};

// ─── Per-message metadata chips ───────────────────────────────────────────────
// Rendered to the right of the Copy button: which agent answered, the model used,
// token usage (↑in ↓out), and counts of skills activated / citations for this
// message. Each chip hides when its datum is absent (older messages, no usage).

const MetaChip: FC<{ icon: ReactNode; label: string; title: string }> = ({ icon, label, title }) => (
  <span title={title} className="inline-flex items-center gap-1 rounded-md bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
    {icon}
    {label}
  </span>
);

const MessageMetaChips: FC = () => {
  const messageId = useMessage((m) => m.id);
  const { activeThreadId } = useThreadUi();
  // Memoize the selector so its reference is stable across renders — an inline
  // selector re-fires assistant-ui's Zustand subscription (React error #185).
  const selector = useMemo(
    () => (activeThreadId ? selectMessageById(activeThreadId, messageId) : () => null),
    [activeThreadId, messageId],
  );
  const message = useChatMessageSelector(selector);
  const answeringAgent = useAgent(message?.agentId);

  if (!message) return null;

  const agentLabel = message.agentId
    ? (answeringAgent?.metadata?.title ?? message.agentId)
    : null;
  const skillCount = message.chunks?.filter((chunk) => chunk.kind === "skill-activation").length ?? 0;
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
          className="inline-flex items-center gap-0.5 rounded-md bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground"
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
  <ActionBarPrimitive.Root hideWhenRunning autohide="not-last" autohideFloat="single-branch" className="-ml-1 flex items-center gap-1 text-muted-foreground">
    <ActionBarPrimitive.Copy render={<TooltipIconButton tooltip="Copy" />}>
      <AuiIf condition={condMessageCopied}><CheckIcon /></AuiIf>
      <AuiIf condition={condMessageNotCopied}><CopyIcon /></AuiIf>
    </ActionBarPrimitive.Copy>
    <MessageMetaChips />
  </ActionBarPrimitive.Root>
);

// ─── RAG citation sources row ────────────────────────────────────────────────
// Renders the [1], [2], ... hover-to-source badges for this message's RAG
// citation stream (see `CitationStream` / `NormalizedEvent::RagCitations` on
// the backend). Sits alongside the action bar so citations stay visible
// without cluttering the message body itself.

const AssistantMessageCitations: FC = () => {
  const messageId = useMessage((m) => m.id);
  const { activeThreadId } = useThreadUi();
  return <MessageCitations threadId={activeThreadId} messageId={messageId} />;
};

// ─── Edit Composer ────────────────────────────────────────────────────────────

const EditComposer: FC = () => (
  <MessagePrimitive.Root className="flex flex-col px-2 [contain-intrinsic-size:auto_200px] [content-visibility:auto]">
    <ComposerPrimitive.Root className="ms-auto flex w-full max-w-[85%] flex-col rounded-(--composer-radius) bg-(--composer-bg) shadow-none">
      <ComposerPrimitive.Input className="min-h-14 w-full resize-none bg-transparent px-4 pt-3 pb-1 font-body text-base text-foreground outline-none" autoFocus />
      <div className="mx-2.5 mb-2.5 flex items-center gap-1.5 self-end">
        <ComposerPrimitive.Cancel render={<Button variant="ghost" size="sm" className="h-8 rounded-full px-3.5" />}>
          Cancel
        </ComposerPrimitive.Cancel>
        <ComposerPrimitive.Send render={<Button size="sm" className="h-8 rounded-full px-3.5" />}>
          Update
        </ComposerPrimitive.Send>
      </div>
    </ComposerPrimitive.Root>
  </MessagePrimitive.Root>
);

// ─── Branch Picker ────────────────────────────────────────────────────────────

const BranchPicker: FC<BranchPickerPrimitive.Root.Props> = ({ className, ...rest }) => (
  <BranchPickerPrimitive.Root hideWhenSingleBranch className={cn("mr-2 -ml-2 inline-flex items-center text-muted-foreground text-xs", className)} {...rest}>
    <BranchPickerPrimitive.Previous render={<TooltipIconButton tooltip="Previous branch" />}>
      <ChevronLeftIcon />
    </BranchPickerPrimitive.Previous>
    <span className="font-mono font-medium"><BranchPickerPrimitive.Number /> / <BranchPickerPrimitive.Count /></span>
    <BranchPickerPrimitive.Next render={<TooltipIconButton tooltip="Next branch" />}>
      <ChevronRightIcon />
    </BranchPickerPrimitive.Next>
  </BranchPickerPrimitive.Root>
);

// ─── Stable components objects ─────────────────────────────────────────────────
// Defined after all component declarations so they can reference them.
// Object literals created inline inside JSX are new references on every render,
// which causes assistant-ui's context/subscription machinery to re-render
// indefinitely (React error #185). Module-level constants are created once.
const USER_MESSAGE_PARTS_COMPONENTS = { Text: MarkdownBubble };
const GROUP_BY = groupPartByType({
  reasoning: ["group-chainOfThought", "group-reasoning"],
  "tool-call": ["group-chainOfThought", "group-tool"],
  "standalone-tool-call": [],
});
const THREAD_COMPONENTS = { UserMessage, EditComposer, AssistantMessage };
