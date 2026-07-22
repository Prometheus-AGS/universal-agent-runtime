import { AssistantRuntimeProvider } from "@assistant-ui/react";
import { EnhancedThread } from "@/components/assistant-ui/enhanced-thread";
import { LeftSidebar } from "@/components/layout/left-sidebar";
import { useUiState } from "@/hooks/use-ui-state";
import { useChatRuntime } from "@/features/chat/use-chat-runtime";
import { AttachmentContext } from "@/features/chat/attachment-context";
import { MemoryContextProvider } from "@/features/chat/memory-context";
import { AgentSelector, type AgentConfig } from "@/features/chat/agent-selector";
import { AgentConfigContext } from "@/features/chat/agent-config-context";
import { SessionConfigPanel } from "@/features/chat/session-config-panel";
import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useThreadUi } from "@/hooks/use-thread-ui";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { AlertTriangle, Menu, Settings2, X } from "lucide-react";
import { useChatPage } from "@/hooks/use-chat-page";

/** Thread detail view — wraps chat runtime for the store-selected thread. */
function ThreadView({ threadId }: { threadId: string }) {
  const { runtime, attachmentManager } = useChatRuntime(threadId);
  return (
    <MemoryContextProvider>
      <AssistantRuntimeProvider runtime={runtime}>
        <AttachmentContext.Provider value={attachmentManager}>
          <EnhancedThread />
        </AttachmentContext.Provider>
      </AssistantRuntimeProvider>
    </MemoryContextProvider>
  );
}

/** Empty state shown when no thread is selected. */
function NoThreadSelected() {
  const { registerThread, setActive } = useThreadUi();

  const handleNew = () => {
    const id = crypto.randomUUID();
    registerThread(id);
    setActive(id);
  };

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 p-8 text-center">
      <p className="font-mono text-[11px] text-primary">{"Select a thread or start a new one"}</p>
      <Button
        onClick={handleNew}
        type="button"
        variant="outline"
        className="rounded-lg border border-primary/30 bg-primary/10 px-6 py-3 font-display text-sm font-semibold text-primary transition-colors hover:bg-primary/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        New conversation
      </Button>
    </div>
  );
}

export function ChatPage() {
  const { activeThreadId } = useThreadUi();
  const { mobileSidebarOpen, setMobileSidebarOpen, toggleMobileSidebar } = useUiState();
  const [configOpen, setConfigOpen] = useState(false);
  const [agentConfigState, setAgentConfigState] = useState<{
    threadId: string | null;
    config: AgentConfig | null;
  }>({ threadId: null, config: null });
  const { modelCheck } = useChatPage();
  const navigate = useNavigate();
  const agentConfig = agentConfigState.threadId === activeThreadId
    ? agentConfigState.config
    : null;

  // Close mobile sidebar when active thread changes
  useEffect(() => { setMobileSidebarOpen(false); }, [activeThreadId, setMobileSidebarOpen]);

  const handleAgentConfigChange = useCallback((config: AgentConfig | null) => {
    setAgentConfigState({ threadId: activeThreadId, config });
  }, [activeThreadId]);

  // No-model guard: block chat if no model is resolvable
  if (!modelCheck.loading && !modelCheck.ok) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <div className="max-w-md text-center">
          <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-xl bg-amber-500/15">
            <AlertTriangle size={24} className="text-amber-400" />
          </div>
          <h2 className="font-display text-lg font-semibold text-foreground">
            No Model Configured
          </h2>
          <p className="mt-2 font-body text-sm text-muted-foreground">
            {modelCheck.error ?? "No default model is configured. Add a provider and set a default model before chatting."}
          </p>
          <Button
            className="mt-4"
            onClick={() => navigate("/admin")}
          >
            Configure Provider
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Mobile sidebar overlay */}
      {mobileSidebarOpen && (
        <div
          className="fixed inset-0 z-40 cursor-pointer bg-black/60 md:hidden"
          data-clickable="true"
          onClick={() => setMobileSidebarOpen(false)}
          onKeyDown={(e) => { if (e.key === "Escape") setMobileSidebarOpen(false); }}
          role="button"
          tabIndex={0}
          aria-label="Close sidebar"
        />
      )}

      {/* Left sidebar */}
      <LeftSidebar
        className={cn(
          "shrink-0 bg-chrome transition-transform duration-200 ease-in-out",
          "fixed inset-y-0 left-0 z-40 md:relative md:z-auto",
          mobileSidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0",
          "w-[var(--sidebar-w)]",
        )}
      />

      {/* Main chat area */}
      <main className="flex flex-1 flex-col overflow-hidden min-w-0">
        {/* Agent selector toolbar — always visible so users can pick an agent
            before starting a thread (threadId null is handled inside the component). */}
        <div className="flex items-center gap-2 bg-background px-4 py-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 shrink-0 md:hidden"
            onClick={toggleMobileSidebar}
            aria-label={mobileSidebarOpen ? "Close sidebar" : "Open sidebar"}
          >
            {mobileSidebarOpen ? <X size={18} /> : <Menu size={18} />}
          </Button>
          <AgentSelector threadId={activeThreadId} onAgentConfigChange={handleAgentConfigChange} className="flex-1" />
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={() => setConfigOpen(true)}
            aria-label="Session configuration"
            disabled={!activeThreadId}
          >
            <Settings2 size={14} />
          </Button>
        </div>
        {activeThreadId ? (
          <>
            <SessionConfigPanel
              threadId={activeThreadId}
              agentConfig={agentConfig}
              open={configOpen}
              onOpenChange={setConfigOpen}
            />
            <AgentConfigContext.Provider value={agentConfig}>
              <ThreadView threadId={activeThreadId} />
            </AgentConfigContext.Provider>
          </>
        ) : (
          <NoThreadSelected />
        )}
      </main>
    </div>
  );
}
