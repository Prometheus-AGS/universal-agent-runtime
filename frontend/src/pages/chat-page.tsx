import { AssistantRuntimeProvider } from "@assistant-ui/react";
import { EnhancedThread } from "@/components/assistant-ui/enhanced-thread";
import { LeftSidebar } from "@/components/layout/left-sidebar";
import { useUiStore } from "@/stores/ui-store";
import { useChatRuntime } from "@/features/chat/use-chat-runtime";
import { AttachmentContext } from "@/features/chat/attachment-context";
import { MemoryContextProvider } from "@/features/chat/memory-context";
import { AgentSelector, type AgentConfig } from "@/features/chat/agent-selector";
import { AgentConfigContext } from "@/features/chat/agent-config-context";
import { SessionConfigPanel } from "@/features/chat/session-config-panel";
import { useEffect, useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { AlertTriangle, Settings2 } from "lucide-react";
import { fetchConfiguredProviders } from "@/services/providers-api";

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
  const registerThread = useThreadRegistryStore((s) => s.registerThread);
  const setActive = useThreadRegistryStore((s) => s.setActive);

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
  const activeThreadId = useThreadRegistryStore((s) => s.activeThreadId);
  const mobileSidebarOpen = useUiStore((s) => s.mobileSidebarOpen);
  const setMobileSidebarOpen = useUiStore((s) => s.setMobileSidebarOpen);
  const [configOpen, setConfigOpen] = useState(false);
  const [agentConfig, setAgentConfig] = useState<AgentConfig | null>(null);
  const [providerCheck, setProviderCheck] = useState<{ loading: boolean; hasConfigured: boolean }>({ loading: true, hasConfigured: true });
  const navigate = useNavigate();

  // Check if at least one provider is configured
  useEffect(() => {
    let cancelled = false;
    fetchConfiguredProviders()
      .then((data) => {
        if (!cancelled) {
          const hasConfigured = (data.providers ?? []).length > 0;
          setProviderCheck({ loading: false, hasConfigured });
        }
      })
      .catch(() => {
        if (!cancelled) setProviderCheck({ loading: false, hasConfigured: false });
      });
    return () => { cancelled = true; };
  }, []);

  // Close mobile sidebar when active thread changes
  useEffect(() => { setMobileSidebarOpen(false); }, [activeThreadId, setMobileSidebarOpen]);

  // Reset agent config when thread changes
  useEffect(() => { setAgentConfig(null); }, [activeThreadId]);

  const handleAgentConfigChange = useCallback((config: AgentConfig | null) => {
    setAgentConfig(config);
  }, []);

  // No-default guard: block chat if no provider is configured
  if (!providerCheck.loading && !providerCheck.hasConfigured) {
    return (
      <div className="flex flex-1 items-center justify-center p-6">
        <div className="max-w-md text-center">
          <div className="mx-auto mb-4 flex size-14 items-center justify-center rounded-xl bg-amber-500/15">
            <AlertTriangle size={24} className="text-amber-400" />
          </div>
          <h2 className="font-display text-lg font-semibold text-foreground">
            No LLM Provider Configured
          </h2>
          <p className="mt-2 font-body text-sm text-muted-foreground">
            You need to configure at least one LLM provider before you can chat.
            Add your API key for OpenAI, Anthropic, or another provider.
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
          "shrink-0 border-r border-border transition-transform duration-200 ease-in-out",
          "fixed inset-y-0 left-0 top-12 z-40 md:relative md:top-auto md:z-auto",
          mobileSidebarOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0",
          "w-[var(--sidebar-w)]",
        )}
      />

      {/* Main chat area */}
      <main className="flex flex-1 flex-col overflow-hidden min-w-0">
        {activeThreadId ? (
          <>
            <div className="flex items-center gap-2 border-b border-border px-4 py-2">
              <AgentSelector threadId={activeThreadId} onAgentConfigChange={handleAgentConfigChange} className="flex-1" />
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7"
                onClick={() => setConfigOpen(true)}
                aria-label="Session configuration"
              >
                <Settings2 size={14} />
              </Button>
            </div>
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
