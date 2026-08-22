import { AssistantRuntimeProvider } from "@assistant-ui/react";

import { EnhancedThread } from "@/components/assistant-ui/enhanced-thread";
import { AgentConfigContext } from "@/features/chat/agent-config-context";
import type { AgentConfig } from "@/features/chat/agent-selector";
import { AttachmentContext } from "@/features/chat/attachment-context";
import { MemoryContextProvider } from "@/features/chat/memory-context";
import { useChatRuntime } from "@/features/chat/use-chat-runtime";

interface ChatThreadViewProps {
  threadId: string;
  agentConfig: AgentConfig | null;
}

function ChatThreadRuntime({ threadId }: Pick<ChatThreadViewProps, "threadId">) {
  const { runtime, attachmentManager } = useChatRuntime(threadId);

  return (
    <AssistantRuntimeProvider runtime={runtime}>
      <AttachmentContext.Provider value={attachmentManager}>
        <EnhancedThread />
      </AttachmentContext.Provider>
    </AssistantRuntimeProvider>
  );
}

/** Load the assistant runtime only after the user selects or creates a thread. */
export function ChatThreadView({ threadId, agentConfig }: ChatThreadViewProps) {
  return (
    <AgentConfigContext.Provider value={agentConfig}>
      <MemoryContextProvider>
        <ChatThreadRuntime threadId={threadId} />
      </MemoryContextProvider>
    </AgentConfigContext.Provider>
  );
}
