import { MessageSquare, MoreHorizontal, Plus, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useThreadRegistryStore } from "@/stores/thread-registry-store";
import { useChatMessageStore } from "@/stores/chat-message-store";
import { useUiStore } from "@/stores/ui-store";
import { cn, formatRelativeTime } from "@/lib/utils";
import type { LocalThread } from "@/types";

interface LeftSidebarProps { className?: string }

export function LeftSidebar({ className }: LeftSidebarProps) {
  const [search, setSearch] = useState("");
  const setMobileSidebarOpen = useUiStore((s) => s.setMobileSidebarOpen);

  const threads = useThreadRegistryStore((s) => s.threads);
  const activeThreadId = useThreadRegistryStore((s) => s.activeThreadId);
  const registerThread = useThreadRegistryStore((s) => s.registerThread);
  const setActive = useThreadRegistryStore((s) => s.setActive);
  const removeThread = useThreadRegistryStore((s) => s.removeThread);

  const visibleThreads: LocalThread[] = Object.values(threads)
    .filter((t) => !t.isEphemeral)
    .filter((t) => t.title.toLowerCase().includes(search.toLowerCase()))
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime());

  const handleNewThread = () => {
    const id = crypto.randomUUID();
    registerThread(id);
    setActive(id);
    setMobileSidebarOpen(false);
  };

  const handleSelectThread = (thread: LocalThread) => {
    setActive(thread.id);
    setMobileSidebarOpen(false);
  };

  const handleDeleteThread = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    removeThread(id);
    useChatMessageStore.getState().clearThread(id);
    if (activeThreadId === id) setActive(null);
  };

  return (
    <aside className={cn("flex h-full flex-col bg-card", className)}>
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-3 py-3">
        <span className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Threads</span>
        <Button variant="outline" size="sm" onClick={handleNewThread} className="h-7 gap-1.5 px-2.5 font-ui text-xs font-semibold text-muted-foreground hover:border-primary hover:text-primary" aria-label="New thread">
          <Plus size={14} />New thread
        </Button>
      </div>

      {/* Search */}
      <div className="px-3 py-2">
        <div className="flex items-center gap-2 rounded-md border border-border bg-background px-2.5 py-1.5">
          <Search size={14} className="text-muted-foreground" />
          <input
            type="text"
            placeholder="Search threads…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent font-ui text-[13px] text-foreground placeholder:text-muted-foreground focus:outline-none"
            aria-label="Search threads"
          />
        </div>
      </div>

      {/* Thread list */}
      <div className="flex-1 overflow-y-auto px-1.5 py-1">
        {visibleThreads.length === 0 ? (
          <div className="px-3 py-8 text-center">
            <p className="font-mono text-[11px] text-primary">{"// No threads yet"}</p>
            <p className="mt-1 font-body text-xs text-muted-foreground">Start a new thread above</p>
          </div>
        ) : (
          visibleThreads.map((thread) => (
            <div key={thread.id} className="group relative">
              <Button
                variant="ghost"
                onClick={() => handleSelectThread(thread)}
                className={cn(
                  "h-auto w-full cursor-pointer justify-start gap-2 rounded-md px-2.5 py-2 pr-9 text-left",
                  activeThreadId === thread.id
                    ? "border-l-[3px] border-l-primary bg-accent hover:bg-accent"
                    : "hover:bg-muted/50",
                )}
                aria-current={activeThreadId === thread.id ? "page" : undefined}
              >
                <MessageSquare size={14} className="mt-0.5 shrink-0 text-muted-foreground" />
                <div className="min-w-0 flex-1">
                  <p className="truncate font-display text-[13px] font-semibold text-foreground">{thread.title}</p>
                  <span className="font-mono text-[10px] text-muted-foreground">{formatRelativeTime(thread.updatedAt)}</span>
                </div>
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon" className="absolute right-1 top-1/2 hidden size-6 -translate-y-1/2 text-muted-foreground hover:text-foreground group-hover:flex" aria-label={`Actions for ${thread.title}`}>
                    <MoreHorizontal size={14} />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={(e) => void handleDeleteThread(e, thread.id)}>
                    <Trash2 size={14} />Delete thread
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          ))
        )}
      </div>
    </aside>
  );
}
