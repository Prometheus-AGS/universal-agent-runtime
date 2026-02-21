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
  BrainIcon,
  CheckIcon,
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ClipboardIcon,
  CopyIcon,
  PaperclipIcon,
  PencilIcon,
  RefreshCwIcon,
  SparklesIcon,
  SquareIcon,
  UserIcon,
} from "lucide-react";
import { type FC, useCallback, useRef, useState } from "react";
import { EnhancedMarkdownText } from "@/components/assistant-ui/enhanced-markdown-text";
import { TooltipIconButton } from "@/components/assistant-ui/tooltip-icon-button";
import { Button } from "@/components/ui/button";
import { ContextUpdateBlock } from "@/features/chat/components/context-update-block";
import { SkillActivationBlock } from "@/features/chat/components/skill-activation-block";
import { ToolCallBlockWrapper } from "@/features/chat/components/tool-call-block";
import { AttachmentPreviewStrip } from "@/features/chat/components/attachment-preview";
import { useAttachmentContext } from "@/features/chat/attachment-context";
import { useMemoryContext } from "@/features/chat/memory-context";
import { cn } from "@/lib/utils";

export const EnhancedThread: FC = () => (
  <ThreadPrimitive.Root
    className="aui-root aui-thread-root @container flex h-full flex-col bg-background"
    style={{ ["--thread-max-width" as string]: "48rem" }}
  >
    <ThreadPrimitive.Viewport
      turnAnchor="top"
      className="aui-thread-viewport relative flex flex-1 flex-col overflow-x-auto overflow-y-scroll scroll-smooth px-4 pt-4"
    >
      <AuiIf condition={(s) => s.thread.isEmpty}>
        <UarWelcome />
      </AuiIf>

      <ThreadPrimitive.Messages components={{ UserMessage, EditComposer, AssistantMessage }} />

      <ThreadPrimitive.ViewportFooter className="aui-thread-viewport-footer sticky bottom-0 mx-auto mt-auto flex w-full max-w-(--thread-max-width) flex-col gap-4 overflow-visible rounded-t-3xl bg-background pb-4 md:pb-6">
        <ThreadScrollToBottom />
        <EnhancedComposer />
      </ThreadPrimitive.ViewportFooter>
    </ThreadPrimitive.Viewport>
  </ThreadPrimitive.Root>
);

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

const EnhancedComposer: FC = () => {
  const attachmentManager = useAttachmentContext();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { memoryEnabled, setMemoryEnabled } = useMemoryContext();

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && attachmentManager) {
        attachmentManager.add(e.target.files);
      }
      // Reset so same file can be re-attached if removed
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
        onChange={handleFileChange}
        aria-hidden
      />

      <ComposerPrimitive.AttachmentDropzone className="relative flex w-full flex-col rounded-2xl border border-input bg-background/80 px-1 pt-2 backdrop-blur-sm outline-none transition-shadow has-[textarea:focus-visible]:border-ring has-[textarea:focus-visible]:ring-2 has-[textarea:focus-visible]:ring-ring/20 overflow-hidden">

        {/* Animated progress bar — only visible while request is in-flight */}
        <AuiIf condition={(s) => s.thread.isRunning}>
          <div className="absolute inset-x-0 top-0 h-[2px] overflow-hidden rounded-t-2xl">
            <div className="h-full w-1/2 animate-[shimmer_1.4s_ease-in-out_infinite] bg-gradient-to-r from-transparent via-primary to-transparent" />
          </div>
        </AuiIf>

        {/* Attachment preview strip – shown above textarea when files are pending */}
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

          {/* Thinking indicator — shown while running */}
          <AuiIf condition={(s) => s.thread.isRunning}>
            <span className="flex items-center gap-1.5 ml-auto font-mono text-[11px] text-muted-foreground/70">
              <span className="inline-flex gap-0.5 items-center">
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.2s_infinite]" />
                <span className="h-1 w-1 rounded-full bg-primary/70 animate-[pulse_1.2s_ease-in-out_0.4s_infinite]" />
              </span>
              Agent is thinking…
            </span>
          </AuiIf>

          {/* Send button — hidden while running */}
          <AuiIf condition={(s) => !s.thread.isRunning}>
            <ComposerPrimitive.Send asChild>
              <TooltipIconButton tooltip="Send message" side="bottom" type="submit" variant="default" size="icon" className="size-8 rounded-full bg-primary text-primary-foreground hover:bg-primary/90 ml-auto" aria-label="Send message">
                <ArrowUpIcon className="size-4" />
              </TooltipIconButton>
            </ComposerPrimitive.Send>
          </AuiIf>

          {/* Cancel button — shown while running */}
          <AuiIf condition={(s) => s.thread.isRunning}>
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

const UserAvatar: FC = () => (
  <div className="flex flex-col items-center gap-1 pt-0.5">
    <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-zinc-700 text-zinc-200 ring-1 ring-zinc-600">
      <UserIcon size={14} />
    </div>
    <span className="font-mono text-[9px] uppercase tracking-wider text-muted-foreground/60">You</span>
  </div>
);

const AgentAvatar: FC = () => (
  <div className="flex flex-col items-center gap-1 pt-0.5">
    <div className="flex size-8 shrink-0 items-center justify-center rounded-full bg-primary/15 text-primary ring-1 ring-primary/30">
      <SparklesIcon size={14} />
    </div>
    <span className="font-mono text-[9px] uppercase tracking-wider text-primary/70">Agent</span>
  </div>
);

const UserMessage: FC = () => (
  <MessagePrimitive.Root className="fade-in slide-in-from-bottom-1 mx-auto flex w-full max-w-(--thread-max-width) animate-in flex-col gap-0.5 px-4 py-2 duration-150" data-role="user">
    <div className="flex w-full items-start gap-3">
      <UserActionBar />
      <div className="min-w-0 flex-1">
        <div className="wrap-break-word rounded-2xl rounded-tr-sm bg-zinc-800 px-4 py-3 font-body text-sm text-foreground leading-relaxed shadow-sm">
          <MessagePrimitive.Parts components={{ Text: EnhancedMarkdownText }} />
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

const AssistantMessage: FC = () => (
  <MessagePrimitive.Root className="fade-in slide-in-from-bottom-1 mx-auto flex w-full max-w-(--thread-max-width) animate-in flex-col gap-0.5 px-4 py-2 duration-150" data-role="assistant">
    <div className="flex w-full items-start gap-3">
      <AgentAvatar />
      <div className="min-w-0 flex-1">
        <div className="wrap-break-word rounded-2xl rounded-tl-sm bg-muted/60 px-4 py-3 font-body text-sm text-foreground leading-relaxed shadow-sm">
          <MessagePrimitive.Parts components={{ Text: EnhancedMarkdownText, Reasoning: ReasoningPart, tools: { Fallback: ToolCallPart } }} />
          <MessageError />
        </div>
      </div>
    </div>
    <div className="ml-11 flex"><BranchPicker /><AssistantActionBar /></div>
  </MessagePrimitive.Root>
);

const ReasoningPart: FC = () => {
  const { text, status } = useMessagePartText();
  const isStreaming = status.type === "running";
  const [isOpen, setIsOpen] = useState(isStreaming);
  return (
    <div className="my-2 overflow-hidden rounded-lg border border-border/50 bg-muted/20">
      <Button variant="ghost" onClick={() => setIsOpen((o) => !o)} className="flex h-auto w-full items-center justify-start gap-2 rounded-none px-3 py-2 hover:bg-muted/30" aria-expanded={isOpen}>
        <BrainIcon size={13} className="shrink-0 text-muted-foreground" />
        <span className="flex-1 font-mono text-[11px] text-muted-foreground">
          {isStreaming ? (
            <span className="flex items-center gap-2">{"// Reasoning"}<span className="inline-flex gap-0.5"><span className="h-1 w-1 animate-pulse rounded-full bg-primary/60" /><span className="h-1 w-1 animate-pulse rounded-full bg-primary/60 [animation-delay:0.2s]" /><span className="h-1 w-1 animate-pulse rounded-full bg-primary/60 [animation-delay:0.4s]" /></span></span>
          ) : "// Reasoning"}
        </span>
        <ChevronDownIcon size={13} className={cn("shrink-0 text-muted-foreground transition-transform duration-150", isOpen && "rotate-180")} />
      </Button>
      {isOpen && (
        <div className="border-t border-border/30 px-3 pb-3 pt-2">
          <p className="whitespace-pre-wrap font-body text-[13px] leading-relaxed text-muted-foreground">
            {text}
            {isStreaming && <span className="ml-0.5 inline-block h-3.5 w-0.5 animate-[pulse_1s_step-end_infinite] bg-primary" />}
          </p>
        </div>
      )}
    </div>
  );
};

const ToolCallPart: FC<ToolCallMessagePartProps> = ({ toolName, args, result, status }) => {
  if (toolName === "__skill__") {
    const a = args as { skillId: string; skillName: string; selectionMethod?: string; status: "active" | "complete" };
    return <SkillActivationBlock skillId={a.skillId} skillName={a.skillName} selectionMethod={a.selectionMethod} status={a.status} />;
  }
  if (toolName === "__context__") {
    const a = args as { strategy: string; messagesRemoved: number; tokensSaved: number; wasApplied: boolean; summaryGenerated: boolean };
    return <ContextUpdateBlock strategy={a.strategy} messagesRemoved={a.messagesRemoved} tokensSaved={a.tokensSaved} wasApplied={a.wasApplied} summaryGenerated={a.summaryGenerated} />;
  }
  return <ToolCallBlockWrapper toolName={toolName} args={args as Record<string, unknown>} result={result} status={status} />;
};

const MessageError: FC = () => {
  const [copied, setCopied] = useState(false);
  const [collapsed, setCollapsed] = useState(false);

  // Read error text from the message status — assistant-ui marks failed messages as
  // status: { type: "incomplete", reason: "error" }; the reason field is what we populated
  // via richMessageToThreadMessageLike which sets reason: "error". The actual text lives
  // in the message's metadata.custom.errorText which we set during conversion.
  const errorText = useMessage((m: ThreadMessageLike) => {
    if (m.status?.type !== "incomplete") return null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (m as any).metadata?.custom?.errorText as string | undefined ?? null;
  });

  const handleCopy = useCallback(() => {
    if (!errorText) return;
    void navigator.clipboard.writeText(errorText).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [errorText]);

  if (!errorText) return null;

  return (
    <div className="mt-3 w-full overflow-hidden rounded-xl border border-destructive/60 bg-destructive/5 shadow-sm">
      {/* Header */}
      <div className="flex items-center gap-2 border-b border-destructive/30 bg-destructive/10 px-3 py-2">
        <AlertTriangleIcon size={14} className="shrink-0 text-destructive" />
        <span className="flex-1 font-mono text-[11px] font-semibold uppercase tracking-widest text-destructive">
          Agent Error
        </span>
        <button
          onClick={() => setCollapsed((c) => !c)}
          className="flex items-center gap-1 rounded px-1.5 py-0.5 font-mono text-[10px] text-destructive/70 hover:bg-destructive/10 transition-colors"
          aria-label={collapsed ? "Expand error" : "Collapse error"}
        >
          {collapsed ? "expand" : "collapse"}
          <ChevronDownIcon size={11} className={cn("transition-transform", !collapsed && "rotate-180")} />
        </button>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1 rounded px-1.5 py-0.5 font-mono text-[10px] text-destructive/70 hover:bg-destructive/10 transition-colors"
          aria-label="Copy error details"
        >
          {copied ? <CheckIcon size={11} /> : <ClipboardIcon size={11} />}
          {copied ? "copied" : "copy"}
        </button>
      </div>
      {/* Body — full error text, no clipping */}
      {!collapsed && (
        <div className="px-3 py-3">
          <pre className="whitespace-pre-wrap break-all font-mono text-[11px] leading-relaxed text-destructive/90">
            {errorText}
          </pre>
        </div>
      )}
    </div>
  );
};

const AssistantActionBar: FC = () => (
  <ActionBarPrimitive.Root hideWhenRunning autohide="not-last" autohideFloat="single-branch" className="col-start-3 row-start-2 -ml-1 flex gap-1 text-muted-foreground">
    <ActionBarPrimitive.Copy asChild>
      <TooltipIconButton tooltip="Copy">
        <AuiIf condition={(s) => s.message.isCopied}><CheckIcon /></AuiIf>
        <AuiIf condition={(s) => !s.message.isCopied}><CopyIcon /></AuiIf>
      </TooltipIconButton>
    </ActionBarPrimitive.Copy>
    <ActionBarPrimitive.Reload asChild>
      <TooltipIconButton tooltip="Regenerate"><RefreshCwIcon /></TooltipIconButton>
    </ActionBarPrimitive.Reload>
  </ActionBarPrimitive.Root>
);

const EditComposer: FC = () => (
  <MessagePrimitive.Root className="mx-auto flex w-full max-w-(--thread-max-width) flex-col px-2 py-3">
    <ComposerPrimitive.Root className="ml-auto flex w-full max-w-[85%] flex-col rounded-2xl bg-muted">
      <ComposerPrimitive.Input className="min-h-14 w-full resize-none bg-transparent p-4 font-body text-foreground text-sm outline-none" autoFocus />
      <div className="mx-3 mb-3 flex items-center gap-2 self-end">
        <ComposerPrimitive.Cancel asChild><Button variant="ghost" size="sm">Cancel</Button></ComposerPrimitive.Cancel>
        <ComposerPrimitive.Send asChild><Button size="sm">Update</Button></ComposerPrimitive.Send>
      </div>
    </ComposerPrimitive.Root>
  </MessagePrimitive.Root>
);

const BranchPicker: FC<BranchPickerPrimitive.Root.Props> = ({ className, ...rest }) => (
  <BranchPickerPrimitive.Root hideWhenSingleBranch className={cn("mr-2 -ml-2 inline-flex items-center text-muted-foreground text-xs", className)} {...rest}>
    <BranchPickerPrimitive.Previous asChild><TooltipIconButton tooltip="Previous branch"><ChevronLeftIcon /></TooltipIconButton></BranchPickerPrimitive.Previous>
    <span className="font-mono font-medium"><BranchPickerPrimitive.Number /> / <BranchPickerPrimitive.Count /></span>
    <BranchPickerPrimitive.Next asChild><TooltipIconButton tooltip="Next branch"><ChevronRightIcon /></TooltipIconButton></BranchPickerPrimitive.Next>
  </BranchPickerPrimitive.Root>
);
